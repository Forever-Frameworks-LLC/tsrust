//! Record literals suspend before fields and publish concrete layout afterward.
use super::super::scoped_record_construction::ScopedFieldFrame;
use super::super::*;
use super::literals::StoredValueFrame;

pub(in crate::analysis) enum RecordFrame {
    Inferred {
        module_literal: bool,
        inferred: Vec<RecordField>,
        types: HashMap<String, Type>,
    },
    Named {
        index: usize,
        seen: HashSet<String>,
        concrete: Vec<Type>,
        specialized: bool,
    },
}

pub(in crate::analysis) enum FieldFrame {
    Inferred,
    Named {
        nominal: usize,
        field_type: Type,
        adaptation: Adaptation,
    },
}

pub(in crate::analysis) enum Adaptation {
    Stored(StoredValueFrame),
    Scoped(ScopedFieldFrame),
}

impl Analyzer {
    pub(in crate::analysis) fn prepare_record_work(
        &mut self,
        fields: &mut Vec<(Token, Expr)>,
        expected: Option<Type>,
        span: Span,
    ) -> Result<RecordFrame, Diagnostic> {
        let module_literal = self
            .module_literal_scope
            .is_some_and(|scope| scope.start <= span.start && span.end <= scope.end);
        if expected.is_none() && (self.builtin_literals || module_literal) {
            return Ok(RecordFrame::Inferred {
                module_literal,
                inferred: Vec::new(),
                types: HashMap::new(),
            });
        }
        let index = match expected {
            Some(Type::Record(index)) => index,
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "A record literal needs a named record type from its annotation or call/return context.",
                    span,
                ));
            }
        };
        let Type::Record(index) = self.scoped_records.recursive_parameter(Type::Record(index))
        else {
            unreachable!()
        };
        if let Some(id) = self.program.records[index].native_type
            && !crate::native_types::declaration(id).source_constructible()
        {
            return Err(Diagnostic::new(
                "native-opaque",
                "Opaque native values cannot be constructed with record literals; use their module's constructors.",
                span,
            ));
        }
        self.fill_omitted_optional_fields(index, fields, span);
        let seen = HashSet::new();
        let concrete = self.program.records[index]
            .fields
            .iter()
            .map(|field| field.ty.expect("checked field"))
            .collect::<Vec<_>>();
        let specialized = false;
        Ok(RecordFrame::Named {
            index,
            seen,
            concrete,
            specialized,
        })
    }

    pub(in crate::analysis) fn prepare_record_child(
        &mut self,
        context: &mut Context,
        key: &Token,
        value: &mut Expr,
        frame: &mut RecordFrame,
    ) -> Result<(Option<Type>, FieldFrame), Diagnostic> {
        let RecordFrame::Named { index, seen, .. } = frame else {
            let RecordFrame::Inferred { types, .. } = frame else {
                unreachable!()
            };
            if types.contains_key(&key.text) {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Duplicate field {:?}.", key.text),
                    key.span,
                ));
            }
            return Ok((None, FieldFrame::Inferred));
        };
        let index = *index;
        if !seen.insert(key.text.clone()) {
            return Err(Diagnostic::new(
                "duplicate",
                format!("Duplicate field {:?}.", key.text),
                key.span,
            ));
        }
        let field_type = self.fields[index].get(&key.text).copied().ok_or_else(|| {
            Diagnostic::new(
                "type",
                format!(
                    "Unknown field {:?} on {}.",
                    key.text,
                    self.type_name(Type::Record(index))
                ),
                key.span,
            )
        })?;
        let nominal = self.scoped_records.declaration(index);
        let scoped_value = matches!(self.unions.payload(field_type).unwrap_or(field_type), Type::StoredCallable(family) if self.callables.storage.scoped_native_family(family).is_some())
            && !matches!(
                self.expression_type_hint(context, value),
                Some(Type::Null | Type::Undefined)
            );
        let field_type =
            if scoped_value || matches!(field_type, Type::Closure(_) | Type::BorrowedCallable(_)) {
                self.program.records[nominal]
                    .fields
                    .iter()
                    .find(|field| field.name.text == key.text)
                    .and_then(|field| field.ty)
                    .unwrap_or(field_type)
            } else {
                field_type
            };
        let (expected, adaptation) = if let Type::StoredCallable(family) = field_type {
            let frame = self.prepare_stored_value(context, value, family)?;
            (frame.expected, Adaptation::Stored(frame))
        } else {
            let concrete_expected = self
                .absent_nullable_record_annotation(context, Some(field_type), value)?
                .unwrap_or(field_type);
            let frame = self.prepare_scoped_record_field(value, concrete_expected);
            (frame.expected, Adaptation::Scoped(frame))
        };
        Ok((
            expected,
            FieldFrame::Named {
                nominal,
                field_type,
                adaptation,
            },
        ))
    }

    pub(in crate::analysis) fn finish_record_child(
        &mut self,
        context: &mut Context,
        key: &Token,
        value: &mut Expr,
        frame: &mut RecordFrame,
        child: FieldFrame,
        actual: Type,
    ) -> Result<(), Diagnostic> {
        match child {
            FieldFrame::Inferred => {
                let RecordFrame::Inferred {
                    module_literal,
                    inferred,
                    types,
                } = frame
                else {
                    unreachable!("inferred record")
                };
                let module_literal = *module_literal;
                if !module_literal
                    && !matches!(
                        actual,
                        Type::Int
                            | Type::Number
                            | Type::Boolean
                            | Type::Null
                            | Type::Undefined
                            | Type::String
                            | Type::Array(_)
                            | Type::Record(_)
                    )
                {
                    return Err(Diagnostic::new(
                        "type",
                        "Builtin object fields must be printable values.",
                        value.span,
                    ));
                }
                types.insert(key.text.clone(), actual);
                inferred.push(RecordField {
                    name: key.clone(),
                    annotation: crate::type_syntax::ValueAnnotation {
                        kind: crate::type_syntax::ValueAnnotationKind::Named(
                            crate::type_syntax::TypeName {
                                text: key.text.clone(),
                                span: key.span,
                            },
                        ),
                        span: key.span,
                    },
                    ty: Some(actual),
                    callable_declaration: None,
                    optional: false,
                });
                Ok(())
            }
            FieldFrame::Named {
                nominal,
                field_type,
                adaptation,
            } => {
                let RecordFrame::Named {
                    index,
                    concrete,
                    specialized,
                    ..
                } = frame
                else {
                    unreachable!("named record")
                };
                let index = *index;
                let actual = match adaptation {
                    Adaptation::Stored(frame) => {
                        self.finish_stored_value(context, value, frame, actual)?
                    }
                    Adaptation::Scoped(frame) => self.finish_scoped_record_field(
                        value,
                        frame.field_type,
                        frame.payload,
                        actual,
                    )?,
                };
                let scoped_native = self.native_borrow_types
                    && actual != field_type
                    && self.native_shape_contains_view(actual);
                let contract_type = if let Type::BorrowedCallable(family) = field_type {
                    Type::Callback(
                        self.callables
                            .storage
                            .declaration(family)
                            .expect("borrowed field declaration"),
                    )
                } else if scoped_native && let Type::StoredCallable(family) = field_type {
                    Type::Callback(
                        self.callables
                            .storage
                            .declaration(family)
                            .expect("owned field declaration"),
                    )
                } else {
                    field_type
                };
                self.require(actual, Some(contract_type), value.span)?;
                let nested_instance = matches!((actual, field_type), (Type::Record(actual), Type::Record(expected))
                if actual != expected && self.scoped_records.declaration(actual) == expected);
                let nested_array_instance = matches!((actual, field_type), (Type::Array(ArrayElement::Record(actual, depth)), Type::Array(ArrayElement::Record(expected, expected_depth)))
                if depth == expected_depth && actual != expected && self.scoped_records.declaration(actual) == expected);
                if scoped_native
                    || nested_instance
                    || nested_array_instance
                    || self.scoped_record_nullable_contract(actual, field_type)
                    || self.scoped_map_contract(actual, field_type)
                    || matches!(
                        self.unions.payload(field_type).unwrap_or(field_type),
                        Type::Callback(_) | Type::BorrowedCallable(_)
                    )
                {
                    let position = self.program.records[index]
                        .fields
                        .iter()
                        .position(|field| field.name.text == key.text)
                        .expect("checked field name");
                    concrete[position] =
                        self.recursive_native_field_type(actual, nominal, position, value.span)?;
                    *specialized = true;
                }
                Ok(())
            }
        }
    }

    pub(in crate::analysis) fn finish_record_work(
        &mut self,
        context: &mut Context,
        fields: &mut Vec<(Token, Expr)>,
        frame: &mut RecordFrame,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        match frame {
            RecordFrame::Inferred {
                module_literal,
                inferred,
                types,
            } => {
                let module_literal = *module_literal;
                let inferred = std::mem::take(inferred);
                let types = std::mem::take(types);
                let id = self.program.records.len();
                let name = if module_literal {
                    format!("@module:object:{id}")
                } else {
                    format!("BuiltinObject{id}")
                };
                self.record_names.insert(name.clone(), id);
                self.program.records.push(Record {
                    native_type: None,
                    map_definition: None,
                    set_definition: None,
                    enum_definition: None,
                    literal_definition: None,
                    application: None,
                    name: Token {
                        text: name,
                        span,
                        kind: TokenKind::Name,
                    },
                    type_parameters: Vec::new(),
                    fields: inferred,
                });
                self.fields.push(types);
                return Ok(Type::Record(id));
            }
            RecordFrame::Named {
                index,
                seen,
                concrete,
                specialized,
            } => {
                let index = *index;
                let specialized = *specialized;
                let concrete = std::mem::take(concrete);
                let mut missing = self.fields[index]
                    .keys()
                    .filter(|name| !seen.contains(*name))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    missing.sort();
                    return Err(Diagnostic::new(
                        "type",
                        format!("Missing fields: {}.", missing.join(", ")),
                        span,
                    ));
                }
                if specialized {
                    let recursive = self.native_borrow_types
                        && concrete
                            .iter()
                            .any(|ty| self.native_shape_contains_view(*ty))
                        && crate::recursive_view_shapes::Index::new(&self.program.records, &|id| {
                            self.unions.members(id)
                        })
                        .recursive(self.scoped_records.declaration(index));
                    let instance = if recursive {
                        self.scoped_records.materialize_recursive(
                            index,
                            concrete,
                            &mut self.program.records,
                            &self.unions,
                        )?
                    } else {
                        self.scoped_records
                            .materialize(index, concrete, &mut self.program.records)
                    };
                    while self.fields.len() < self.program.records.len() {
                        let id = self.fields.len();
                        self.fields.push(
                            self.program.records[id]
                                .fields
                                .iter()
                                .map(|field| {
                                    (field.name.text.clone(), field.ty.expect("concrete field"))
                                })
                                .collect(),
                        );
                    }
                    if recursive {
                        for (name, value) in fields.iter_mut() {
                            let target = self.fields[instance][&name.text];
                            if value.ty != Some(target) {
                                if matches!(&value.kind, ExprKind::Array(values) | ExprKind::ArrayOf(values, _) if values.is_empty())
                                    && matches!(target, Type::Array(_))
                                {
                                    value.ty = Some(target);
                                } else if let ExprKind::Inject {
                                    value: payload,
                                    union,
                                } = &mut value.kind
                                    && matches!(payload.ty, Some(Type::Null | Type::Undefined))
                                    && let Type::Union(destination) = target
                                {
                                    *union = destination;
                                    value.ty = Some(target);
                                } else {
                                    value.wrap_record_carrier(target);
                                }
                            }
                        }
                        self.refresh_native_recursive_shapes(context);
                    }
                    return Ok(Type::Record(instance));
                }
                Ok(Type::Record(index))
            }
        }
    }
}
