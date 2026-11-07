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
    pub(in crate::analysis) fn inferred_object_field(key: &Token, actual: Type) -> RecordField {
        RecordField {
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
        }
    }
    pub(in crate::analysis) fn intern_inferred_object(
        &mut self,
        module_literal: bool,
        inferred: Vec<RecordField>,
        types: HashMap<String, Type>,
        span: Span,
    ) -> Type {
        let shape = module_literal.then(|| {
            format!(
                "@object:shape:{:?}",
                inferred
                    .iter()
                    .map(|field| (&field.name.text, field.ty))
                    .collect::<Vec<_>>()
            )
        });
        if let Some(id) = shape
            .as_ref()
            .and_then(|shape| self.record_names.get(shape))
        {
            return Type::Record(*id);
        }
        let id = self.program.records.len();
        let name = if module_literal {
            format!("@object:{id}")
        } else {
            format!("BuiltinObject{id}")
        };
        if let Some(shape) = shape {
            self.record_names.insert(shape, id);
        }
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
        Type::Record(id)
    }
    pub(in crate::analysis) fn prepare_record_work(
        &mut self,
        context: &Context,
        fields: &mut Vec<(Token, Expr)>,
        expected: Option<Type>,
        span: Span,
    ) -> Result<RecordFrame, Diagnostic> {
        let module_literal = self
            .module_literal_scope
            .is_some_and(|scope| scope.start <= span.start && span.end <= scope.end)
            || (!fields.is_empty() && fields.iter().all(|(name,_)|name.text.starts_with("future#task")));
        if expected.is_none() {
            return Ok(RecordFrame::Inferred {
                module_literal: module_literal || !self.builtin_literals,
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
            && !(id==crate::native_signatures::NativeTypeId::OwnedIterator && context.function_name.starts_with("@stdlib:iterator#create"))
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
        loans: &Loans,
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
            // As with a declaration initializer, a callable binding contributes
            // its concrete environment; the child still uses consuming access.
            let mut expected=None;
            if let ExprKind::Name(name)=&value.kind
                && let Some(&binding)=context.names.get(name)
            {
                let ty=context.bindings[binding].ty;
                if matches!(ty,Type::Closure(_)) || matches!(ty,Type::Callback(id) if self.callables.callbacks[id].kind.is_owned()) {
                    expected=Some(ty);
                }
                // Program storage cannot be transferred. A checked, capture-free
                // named function denotes a fresh empty environment, not a copy of
                // a captured/Once callback. Preserve the source binding's access
                // checks before constructing that declaration's ordinary value.
                if context.bindings[binding].declaration_span.is_some_and(|span|self.program.module_owner_bindings.contains(&span))
                    && !context.bindings[binding].rebindable
                    && !context.bindings[binding].parameter
                    && !context.bindings[binding].capture_reference
                    && let Type::Closure(origin)=ty
                {
                    let closure=&self.closures[origin];
                    let declaration=if closure.captures.is_empty() && closure.program_captures.is_empty()
                        && !closure.borrowed && closure.lifetime_count==0 && !closure.program_scoped
                        && !closure.program_async && closure.invocation==Mode::Read {
                        match closure.forward {
                            Some(ClosureForward::Function(function)) if self.checked[function].is_some() && !self.named_captures.needs_context(function)=>Some(self.program.functions[function].name.text.clone()),
                            _=>self.deferred_generic_values.get(&origin).filter(|source|source.context.is_none() && source.identities.is_empty()).map(|source|source.declaration.clone()),
                        }
                    }else{None};
                    if let Some(declaration)=declaration.filter(|name|!context.names.contains_key(name)) {
                        Self::access(context,binding,Mode::Read,value.span,loans)?;
                        value.kind=ExprKind::Name(declaration);
                        expected=None;
                    }
                }
            }
            return Ok((expected, FieldFrame::Inferred));
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
                            | Type::Closure(_)
                    ) && !matches!(actual,Type::Callback(id) if self.callables.callbacks[id].kind.is_owned())
                {
                    return Err(Diagnostic::new(
                        "type",
                        "Builtin object fields must be printable values.",
                        value.span,
                    ));
                }
                types.insert(key.text.clone(), actual);
                inferred.push(Self::inferred_object_field(key,actual));
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
                let scoped_native = actual != field_type
                    && (self.native_borrow_types && self.native_shape_contains_view(actual)
                        || matches!(actual,Type::StoredCallable(family) if self.callables.storage.scoped_native_family(family).is_some_and(|layout|layout.lifetimes()==0)));
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
                Ok(self.intern_inferred_object(module_literal,inferred,types,span))
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
                        let absent=&self.absent_callback_payloads;
                        self.scoped_records.materialize_recursive(
                            index,
                            concrete,
                            &mut self.program.records,
                            &self.unions,
                            &|id|absent.values().any(|&empty|empty==id),
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
