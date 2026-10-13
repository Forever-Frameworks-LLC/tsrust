//! Receiver analysis boundaries preserve projection and storage access state.
use super::super::*;

pub(in crate::analysis) enum FieldFrame {
    Length,
    Temporary { borrowed: bool },
    Named { top_projection: bool },
}
pub(in crate::analysis) enum FieldDispatch {
    Ready(Type),
    Receiver { mode: Mode, frame: FieldFrame },
}

impl Analyzer {
    pub(in crate::analysis) fn prepare_field_work(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        name: &str,
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<FieldDispatch, Diagnostic> {
        if matches!(&base.kind, ExprKind::Name(namespace) if NumericConstant::lookup(namespace, name).is_some())
        {
            return Self::numeric_constant_access(mode, span).map(FieldDispatch::Ready);
        }
        if self.ordered_set_receiver(context, base).is_some() {
            return self
                .ordered_set_size(context, base, name, mode, loans, span)
                .map(FieldDispatch::Ready);
        }
        if self.heap_receiver(context, base).is_some() {
            return self
                .heap_size(context, base, name, mode, loans, span)
                .map(FieldDispatch::Ready);
        }
        if self.deque_receiver(context, base).is_some() {
            return self
                .deque_size(context, base, name, mode, loans, span)
                .map(FieldDispatch::Ready);
        }
        if self.map_receiver_hint(context, base) {
            return self
                .map_size(context, base, name, mode, loans, span)
                .map(FieldDispatch::Ready);
        }
        if self.set_receiver_hint(context, base) {
            return self
                .set_size(context, base, name, mode, loans, span)
                .map(FieldDispatch::Ready);
        }
        if name == "length"
            && !matches!(
                base.kind,
                ExprKind::Name(_)
                    | ExprKind::Selected { .. }
                    | ExprKind::Index(_, _)
                    | ExprKind::Field(_, _)
                    | ExprKind::Project { .. }
            )
        {
            return Ok(FieldDispatch::Receiver {
                mode: Mode::Read,
                frame: FieldFrame::Length,
            });
        }
        if !Self::named_storage(base) {
            let borrowed = self.borrowed_place_hint(context, base)
                || self.borrowed_query_projection_hint(context, base);
            if borrowed && mode == Mode::Mutate {
                return Err(Diagnostic::new(
                    "borrowed-place",
                    "An assignment result is readonly.",
                    span,
                ));
            }
            return Ok(FieldDispatch::Receiver {
                mode: if borrowed { Mode::Read } else { Mode::Consume },
                frame: FieldFrame::Temporary { borrowed },
            });
        }
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Sequence(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(
                base.kind,
                ExprKind::Sequence(_)
                    | ExprKind::Unary {
                        op: UnaryOp::NonNull,
                        ..
                    }
            ) && Self::named_storage(base))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Field access requires named, indexed, or field storage.",
                span,
            ));
        }
        let top_projection = context.projection.is_none();
        // Ordinary fully available field reads need no owned path allocation.
        let partial = if top_projection && context.has_field_moves {
            let mut root: &Expr = base;
            loop {
                match &root.kind {
                    ExprKind::Field(parent, _) => root = parent,
                    ExprKind::Sequence(values) => root = values.last().expect("nonempty sequence"),
                    _ => break,
                }
            }
            matches!(&root.kind, ExprKind::Name(root) if context.names.get(root).is_some_and(|&id| !context.bindings[id].moved_fields.is_empty()))
        } else {
            false
        };
        let mut direct =
            if top_projection && (partial || !loans.is_empty() || context.native_view_calls) {
                Self::direct_fields(base)
            } else {
                None
            };
        if let Some((_, path)) = &mut direct {
            path.push(name.to_owned());
        }
        if top_projection && let Some((root, path)) = &direct {
            context.projection = Some(Projection {
                root: root.clone(),
                path: path.clone(),
                replacing: context.replacement == Some(span),
                span,
            });
        }
        let base_mode = if mode == Mode::Mutate {
            Mode::Mutate
        } else {
            Mode::Read
        };
        Ok(FieldDispatch::Receiver {
            mode: base_mode,
            frame: FieldFrame::Named { top_projection },
        })
    }

    pub(in crate::analysis) fn finish_field_work(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        name: &str,
        mode: Mode,
        loans: &Loans,
        span: Span,
        frame: FieldFrame,
        result: Result<Type, Diagnostic>,
    ) -> Result<Type, Diagnostic> {
        match frame {
            FieldFrame::Length => {
                let actual = result?;
                if !matches!(actual, Type::String | Type::Array(_))
                    && !matches!(actual, Type::Record(id) if self.program.records[id].is_tuple())
                {
                    return Err(Diagnostic::new(
                        "type",
                        "Length requires a string or array.",
                        base.span,
                    ));
                }
                if mode == Mode::Mutate {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "String and array length are read-only; use push() to append an array element.",
                        span,
                    ));
                }
                Ok(Type::Int)
            }
            FieldFrame::Temporary { borrowed } => {
                let ty = result?;
                if matches!(ty.scalar_base(), Type::String | Type::Array(_)) && name == "length" {
                    if mode == Mode::Mutate {
                        return Err(Diagnostic::new(
                            "unsupported",
                            "String and array length are read-only; use push() to append an array element.",
                            span,
                        ));
                    }
                    return Ok(Type::Int);
                }
                let field = match ty {
                    Type::Record(record) => self.fields[record].get(name).copied(),
                    _ => None,
                };
                if borrowed && mode == Mode::Consume && field.is_some_and(|ty| !ty.is_copy()) {
                    return Err(Diagnostic::new(
                        "borrowed-place",
                        "An owned field of an assignment result is borrowed; use clone().",
                        span,
                    ));
                }
                field.ok_or_else(|| {
                    Diagnostic::new(
                        "type",
                        format!("Type {:?} has no field {name:?}.", self.type_name(ty)),
                        span,
                    )
                })
            }
            FieldFrame::Named { top_projection } => {
                if top_projection {
                    context.projection = None;
                }
                let base_type = result?;
                let base_type = self.recursive_array_receiver(context, base, base_type);
                if (matches!(base_type.scalar_base(), Type::Array(_) | Type::String)
                    || matches!(base_type, Type::Record(id) if self.program.records[id].is_tuple()))
                    && name == "length"
                {
                    if mode == Mode::Mutate {
                        return Err(Diagnostic::new(
                            "unsupported",
                            "String and array length are read-only; use push() to append an array element.",
                            span,
                        ));
                    }
                    return Ok(Type::Int);
                }
                let field_type = match base_type {
                    Type::Record(index) => self.fields[index].get(name).copied(),
                    Type::Union(union) => self.record_union_field(union, name),
                    _ => None,
                }
                .ok_or_else(|| {
                    Diagnostic::new(
                        "type",
                        format!(
                            "Type {:?} has no field {name:?}.",
                            self.type_name(base_type)
                        ),
                        span,
                    )
                })?;
                if matches!(base_type, Type::Union(_))
                    && (mode == Mode::Mutate || mode == Mode::Consume && !field_type.is_copy())
                {
                    return Err(Diagnostic::new(
                        "union-field-ownership",
                        "Narrow the record alternative before mutating or moving an owned field.",
                        span,
                    ));
                }
                if mode == Mode::Consume && !field_type.is_copy() {
                    let mut parent: &Expr = base;
                    loop {
                        match &parent.kind {
                            ExprKind::Sequence(values) => {
                                parent = values.last().expect("nonempty sequence")
                            }
                            ExprKind::Field(value, _)
                                if matches!(value.ty, Some(Type::Union(_))) =>
                            {
                                return Err(Diagnostic::new(
                                    "union-field-ownership",
                                    "Narrow and move the whole record before consuming a nested owned field.",
                                    span,
                                ));
                            }
                            ExprKind::Project {
                                narrowed: Type::Record(_),
                                ..
                            } => {
                                return Err(Diagnostic::new(
                                    "unsupported",
                                    "Move the whole narrowed record payload to a local before consuming its owned fields.",
                                    span,
                                ));
                            }
                            ExprKind::Project { value, .. }
                            | ExprKind::Field(value, _)
                            | ExprKind::Index(value, _) => parent = value,
                            _ => break,
                        }
                    }
                    let Some((root, mut path)) = Self::direct_fields(base) else {
                        return Err(Diagnostic::new(
                            "borrowed-element",
                            "Moving a field through indexed storage is not allowed; remove the element first or clone its field.",
                            span,
                        ));
                    };
                    path.push(name.to_owned());
                    context.projection = Some(Projection {
                        root,
                        path,
                        replacing: false,
                        span,
                    });
                    let checked = Self::access(
                        context,
                        Self::root_binding(base).expect("direct field root"),
                        Mode::Consume,
                        span,
                        loans,
                    );
                    context.projection = None;
                    checked?;
                }
                Ok(field_type)
            }
        }
    }
}

pub(super) enum IndexKind {
    String,
    Named,
    Temporary { borrowed: bool },
}
pub(super) struct IndexFrame {
    pub(super) kind: IndexKind,
    pub(super) mode: Mode,
}
pub(super) struct IndexOperand {
    pub(super) element: Type,
    pub(super) mode: Mode,
    pub(super) named: bool,
}
pub(super) enum IndexAdvance {
    Done(Type),
    Operand { frame: IndexOperand, loans: Loans },
}

impl Analyzer {
    pub(super) fn prepare_index_work(
        &mut self,
        context: &Context,
        base: &Expr,
        mode: Mode,
        expected: Option<Type>,
    ) -> Result<(IndexFrame, Mode, Option<Type>), Diagnostic> {
        if !Self::optional_hint_boundary(base)
            && self
                .expression_type_hint(context, base)
                .map(Type::scalar_base)
                == Some(Type::String)
        {
            return Ok((
                IndexFrame {
                    kind: IndexKind::String,
                    mode,
                },
                Mode::Read,
                None,
            ));
        }
        if Self::named_storage(base) {
            Self::prepare_named_index(base)?;
            Ok((
                IndexFrame {
                    kind: IndexKind::Named,
                    mode,
                },
                Mode::Read,
                None,
            ))
        } else {
            let borrowed = self.borrowed_place_hint(context, base)
                || self.borrowed_query_projection_hint(context, base);
            let expected = expected
                .and_then(ArrayElement::from_value_type)
                .map(Type::Array);
            Ok((
                IndexFrame {
                    kind: IndexKind::Temporary { borrowed },
                    mode,
                },
                if borrowed { Mode::Read } else { Mode::Consume },
                expected,
            ))
        }
    }

    pub(super) fn finish_index_base_work(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        index: &mut Expr,
        span: Span,
        frame: IndexFrame,
        ty: Type,
        loans: &Loans,
    ) -> Result<IndexAdvance, Diagnostic> {
        let (element, active, named) = match frame.kind {
            IndexKind::String => {
                debug_assert_eq!(ty.scalar_base(), Type::String);
                let mut active = loans.clone();
                for root in Self::loan_roots(base, Mode::Read) {
                    active
                        .entry(root)
                        .or_default()
                        .push((Mode::Read, base.span));
                }
                (Type::String, active, false)
            }
            IndexKind::Named => {
                let element = match self.finish_named_index_receiver(
                    context, base, index, frame.mode, loans, span, ty,
                )? {
                    Ok(field) => return Ok(IndexAdvance::Done(field)),
                    Err(element) => element,
                };
                (element, Self::selection_loans(base, loans), true)
            }
            IndexKind::Temporary { borrowed } => {
                let element = match self.finish_temporary_index_receiver(
                    context, base, index, frame.mode, borrowed, ty,
                )? {
                    Ok(field) => return Ok(IndexAdvance::Done(field)),
                    Err(element) => element,
                };
                let mut active = loans.clone();
                if borrowed {
                    for root in Self::loan_roots(base, Mode::Read) {
                        active
                            .entry(root)
                            .or_default()
                            .push((Mode::Read, base.span));
                    }
                }
                (element, active, false)
            }
        };
        Ok(IndexAdvance::Operand {
            frame: IndexOperand {
                element,
                mode: frame.mode,
                named,
            },
            loans: active,
        })
    }

    pub(super) fn finish_index_operand_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        frame: IndexOperand,
        actual: Type,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Index(base, index) = &mut expr.kind else {
            unreachable!("index expression");
        };
        if frame.named {
            self.array_index_access(
                context,
                base,
                (actual, index.span),
                frame.mode,
                loans,
                expr.span,
            )?;
        } else {
            self.require(actual, Some(Type::Int), index.span)?;
        }
        self.finish_index_work(expr, frame.element)
    }

    pub(super) fn finish_index_work(
        &self,
        expr: &mut Expr,
        result: Type,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Index(base, index) = &mut expr.kind else {
            unreachable!("index expression");
        };
        if let Some((name, _)) =
            self.checked_tuple_index(base.ty.expect("checked index receiver"), index)?
        {
            let span = base.span;
            let receiver = std::mem::replace(base, Box::new(Expr::new(ExprKind::Undefined, span)));
            expr.kind = ExprKind::Field(receiver, name);
        }
        Ok(result)
    }
}
