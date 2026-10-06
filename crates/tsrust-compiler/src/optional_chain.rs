//! Optional postfix chains keep each selected source subtree exactly once.
use super::*;
use crate::syntax::{NullishBranch, SelectedSource};

impl Analyzer {
    fn optional_static(&self, context: &Context, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Call(callee, _) => {
                if matches!(&callee.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, value, .. } if matches!(value.kind, ExprKind::Name(_) | ExprKind::Field(_, _)) && !self.expression_type_hint(context,value).is_some_and(|ty|ty.union().is_some() || matches!(ty,Type::Null|Type::Undefined)))
                {
                    let placeholder = Expr::checked(ExprKind::Null, callee.span, None);
                    let old = std::mem::replace(callee.as_mut(), placeholder);
                    let ExprKind::Unary { value, .. } = old.kind else {
                        unreachable!()
                    };
                    **callee = *value;
                }
                self.optional_static(context, callee);
            }
            ExprKind::Field(base, _) => {
                if matches!(&base.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, value, .. } if matches!(&value.kind, ExprKind::Name(name) if matches!(name.as_str(), "Array"|"Math"|"Number"|"console"|"Boolean"|"String")))
                {
                    let placeholder = Expr::checked(ExprKind::Null, base.span, None);
                    let old = std::mem::replace(base.as_mut(), placeholder);
                    let ExprKind::Unary { value, .. } = old.kind else {
                        unreachable!()
                    };
                    **base = *value;
                }
                self.optional_static(context, base);
            }
            ExprKind::Index(base, _)
            | ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value: base,
                ..
            } => self.optional_static(context, base),
            _ => {}
        }
    }

    /// Find only the receiver spine, never argument/index subexpressions or grouped chains.
    fn optional_extract(expr: &mut Expr) -> Option<(Expr, Span)> {
        match &mut expr.kind {
            ExprKind::Field(base, _) | ExprKind::Index(base, _) | ExprKind::Call(base, _) => {
                Self::optional_extract(base)
            }
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value,
                operator,
            } => {
                if let Some(first) = Self::optional_extract(value) {
                    return Some(first);
                }
                let span = *operator;
                let placeholder = Expr::checked(ExprKind::Null, value.span, None);
                let base = std::mem::replace(value.as_mut(), placeholder);
                let source_span = base.span;
                *expr = Expr::checked(
                    ExprKind::Selected {
                        source: Box::new(SelectedSource {
                            readonly: false,
                            roots: Vec::new(),
                            place: None,
                            span: source_span,
                        }),
                        slot: span.start,
                        owned: false,
                        access: Mode::Read,
                    },
                    source_span,
                    None,
                );
                Some((base, span))
            }
            _ => None,
        }
    }

    fn optional_selected(
        expr: &mut Expr,
        slot: usize,
        ty: Type,
        source: &SelectedSource,
        owned: bool,
    ) {
        match &mut expr.kind {
            ExprKind::Selected {
                source: current,
                slot: id,
                owned: own,
                ..
            } if *id == slot => {
                **current = source.clone();
                *own = owned;
                expr.ty = Some(ty);
            }
            ExprKind::Field(base, _)
            | ExprKind::Index(base, _)
            | ExprKind::Call(base, _)
            | ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value: base,
                ..
            } => Self::optional_selected(base, slot, ty, source, owned),
            _ => {}
        }
    }

    #[inline(never)]
    pub(super) fn optional_hint_boundary(mut value:&Expr)->bool {
        loop {match &value.kind {
            ExprKind::Unary{op:UnaryOp::OptionalBoundary,..}=>return true,
            ExprKind::Field(base,_)|ExprKind::Index(base,_)|ExprKind::Call(base,_)=>value=base,
            _=>return false,
        }}
    }

    pub(super) fn optional_hint(&self, context: &Context, body: &Expr) -> Option<Type> {
        let result = self.expression_type_hint(context, body)?;
        fn missing(analyzer: &Analyzer, context: &Context, expr: &Expr) -> bool {
            match &expr.kind {
                ExprKind::Unary {
                    op: UnaryOp::OptionalBoundary,
                    value,
                    ..
                } => {
                    analyzer
                        .expression_type_hint(context, value)
                        .is_some_and(|ty| {
                            ty.union().is_some_and(|id| {
                                analyzer
                                    .unions
                                    .members(id)
                                    .iter()
                                    .any(|ty| matches!(ty, Type::Null | Type::Undefined))
                            }) || matches!(ty, Type::Null | Type::Undefined)
                        })
                        || missing(analyzer, context, value)
                }
                ExprKind::Field(base, _) | ExprKind::Index(base, _) | ExprKind::Call(base, _) => {
                    missing(analyzer, context, base)
                }
                _ => false,
            }
        }
        if missing(self, context, body) {
            let result = if result == Type::Void {
                Type::Undefined
            } else {
                result.owning()
            };
            let joined = self
                .unions
                .intern([result, Type::Undefined], body.span)
                .ok()?;
            if self.optional_borrowed_hint(context, body) && !joined.is_copy() {
                joined.union().map(Type::BorrowedUnion)
            } else {
                Some(joined)
            }
        } else {
            Some(result)
        }
    }

    pub(super) fn optional_borrowed_hint(&self, context: &Context, body: &Expr) -> bool {
        if self
            .expression_type_hint(context, body)
            .is_some_and(Type::is_copy)
        {
            return false;
        }
        match &body.kind {
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => {
                self.optional_source_named(context, base)
            }
            _ => false,
        }
    }
    fn optional_source_named(&self, context: &Context, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::OptionalChain(value) => self.optional_borrowed_hint(context, value),
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                self.optional_source_named(context, consequent)
                    && self.optional_source_named(context, alternate)
            }
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value,
                ..
            } => self.optional_source_named(context, value),
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => {
                self.optional_source_named(context, base)
            }
            ExprKind::Selected { owned, .. } => !owned,
            ExprKind::Name(name) => context.names.contains_key(name),
            _ => false,
        }
    }

    pub(super) fn selected_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ty = expr.ty.expect("checked selected payload");
        let ExprKind::Selected {
            source,
            slot,
            owned,
            access,
        } = &mut expr.kind
        else {
            unreachable!()
        };
        if source.readonly && mode == Mode::Mutate {
            return Err(Diagnostic::new(
                "borrowed-view",
                "An optional readonly view cannot be mutated; use clone() for independent ownership.",
                expr.span,
            ));
        }
        if !*owned && mode == Mode::Consume && !ty.is_copy() {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An optional selection borrows its source; use clone() for independent ownership.",
                expr.span,
            ));
        }
        let mode = if ty.is_copy() && mode != Mode::Mutate {
            Mode::Read
        } else {
            mode
        };
        *access = (*access).max(mode);
        context
            .selected_access
            .entry(*slot)
            .and_modify(|prior| *prior = (*prior).max(mode))
            .or_insert(mode);
        if !*owned {
            for root in &source.roots {
                let prior = context.projection.take();
                if let Some((id, path)) = &source.place
                    && id == root
                    && !path.is_empty()
                {
                    context.projection = Some(Projection {
                        root: context.bindings[*root].name.clone(),
                        path: path.clone(),
                        replacing: false,
                        span: expr.span,
                    });
                }
                let keep = context.keep_flow.len();
                if let Some((root, fields)) = &source.place {
                    context.keep_flow.push(crate::flow::Place {
                        root: *root,
                        fields: fields.clone(),
                    });
                }
                let result = Self::access(context, *root, mode, expr.span, loans);
                context.keep_flow.truncate(keep);
                context.projection = prior;
                result?;
            }
        }
        self.require(ty, expected, expr.span)?;
        Ok(ty)
    }

    // A stable, physically non-null storage reference has no optional branch.
    // Keep the ordinary helper's argument evaluation and receiver reacquisition.
    fn optional_present_storage(&self, context: &Context, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Field(base, _) | ExprKind::Index(base, _) | ExprKind::Call(base, _) => {
                self.optional_present_storage(context, base);
            }
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value,
                ..
            } => {
                self.optional_present_storage(context, value);
                let physical = crate::flow::place(value, &context.names)
                    .and_then(|place| self.place_type(context, &place));
                if Self::named_storage(value)
                    && physical.is_some_and(|ty| {
                        ty.union().is_none() && !matches!(ty, Type::Null | Type::Undefined)
                    })
                {
                    let span = value.span;
                    let selected = std::mem::replace(
                        value.as_mut(),
                        Expr::checked(ExprKind::Null, span, None),
                    );
                    *expr = selected;
                }
            }
            _ => {}
        }
    }

    pub(super) fn optional_chain(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let old = std::mem::replace(expr, Expr::checked(ExprKind::Null, expr.span, None));
        let ExprKind::OptionalChain(mut body) = old.kind else {
            unreachable!()
        };
        self.optional_static(context, &mut body);
        self.optional_present_storage(context, &mut body);
        let result = self.optional_body(context, &mut body, mode, loans, expected)?;
        expr.kind = body.kind;
        expr.ty = Some(result);
        expr.binding = body.binding;
        expr.target = body.target;
        // Preserve the complete chain range even when static identities disappear.
        if !matches!(expr.kind, ExprKind::Optional { .. }) {
            let mut value = Expr::checked(
                std::mem::replace(&mut expr.kind, ExprKind::Null),
                old.span,
                Some(result),
            );
            value.binding = expr.binding;
            value.target = expr.target;
            expr.kind = ExprKind::Unary {
                op: UnaryOp::OptionalSelected,
                operator: old.span,
                value: Box::new(value),
            };
        }
        Ok(result)
    }

    fn optional_body(
        &mut self,
        context: &mut Context,
        body: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let Some((mut base, operator)) = Self::optional_extract(body) else {
            return self.expression(context, body, mode, loans, expected);
        };
        let place = crate::flow::place(&base, &context.names);
        let possible = place.as_ref().filter(|place|place.fields.is_empty()).and_then(|place|context.constant_absence.get(&place.root)).map(|ty|vec![*ty]).or_else(||place
            .as_ref()
            .and_then(|place| context.flow.get(place))
            .map(|fact| fact.members.clone()));
        if let Some(place) = &place {
            context.suppressed_flow.insert(place.clone());
        }
        let named = Self::named_storage(&base)
            || self.optional_source_named(context, &base)
            || self.borrowed_place_hint(context, &base);
        let actual = self.expression(
            context,
            &mut base,
            if named { Mode::Read } else { Mode::Consume },
            loans,
            None,
        );
        if let Some(place) = &place {
            context.suppressed_flow.remove(place);
        }
        let actual = actual?;
        let union = actual.union();
        let members = union
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![actual]);
        let possible = possible.unwrap_or_else(|| members.clone());
        let present_types = possible.iter().copied().filter(|ty| !matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
        let mut payloads = present_types.iter().copied();
        let payload = payloads.next().or_else(|| self.unions.payload(actual.owning())).ok_or_else(|| Diagnostic::new("type", "An optional member or call requires a declared non-null payload type.", operator))?;
        if payloads.next().is_some() { return Err(Diagnostic::new("unsupported", "Optional member access requires one proven payload type; narrow the union first.", operator)); }
        let absent = possible
            .iter()
            .any(|ty| matches!(ty, Type::Null | Type::Undefined));
        let present = possible
            .iter()
            .any(|ty| !matches!(ty, Type::Null | Type::Undefined));
        let branch = if !absent {
            NullishBranch::Present
        } else if !present {
            NullishBranch::Absent
        } else {
            NullishBranch::Dynamic
        };
        let owned = actual.is_copy() || (!named && !matches!(actual, Type::BorrowedUnion(_)));
        let mut roots = Self::loan_roots(&base, Mode::Read);
        for root in roots.clone() {
            for source in &context.bindings[root].view_roots {
                if !roots.contains(source) {
                    roots.push(*source);
                }
            }
        }
        let source = SelectedSource {
            readonly: matches!(actual, Type::BorrowedUnion(_))
                || self.borrowed_place_hint(context, &base),
            roots,
            place: place.as_ref().map(|p| (p.root, p.fields.clone())),
            span: base.span,
        };
        Self::optional_selected(body, operator.start, payload, &source, owned);
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let before_flow = context.flow.clone();
        if let Some(place) = &place {
            context.flow.insert(
                place.clone(),
                crate::flow::Fact {
                    members: vec![payload],
                    origin: Some((operator.start, operator.end)),
                },
            );
        }
        let first = context.capture_loans.len();
        let body_ty = self.optional_body(
            context,
            body,
            mode,
            loans,
            expected.map(|ty| self.unions.payload(ty).unwrap_or(ty)),
        )?;
        context.capture_loans.truncate(first);
        let access = context
            .selected_access
            .remove(&operator.start)
            .unwrap_or(Mode::Read);
        if branch != NullishBranch::Present {
            for (id, state) in before {
                let after = Availability::of(&context.bindings[id]);
                if branch == NullishBranch::Absent {
                    state.restore(&mut context.bindings[id]);
                } else {
                    state.join(after).restore(&mut context.bindings[id]);
                }
            }
            context.flow = if branch == NullishBranch::Absent {
                before_flow
            } else {
                crate::flow::join(&before_flow, &context.flow)
            };
        } else if let Some(place) = &place {
            // Selection facts are branch-local; preserve unrelated successful effects.
            if let Some(fact) = before_flow.get(place) {
                context.flow.insert(place.clone(), fact.clone());
            } else {
                context.flow.remove(place);
            }
        }
        let value_type = if body_ty == Type::Void {
            Type::Undefined
        } else {
            body_ty.owning()
        };
        let joined = if absent {
            self.unions
                .intern([value_type, Type::Undefined], body.span)?
        } else {
            body_ty
        };
        let borrowed = !joined.is_copy()
            && (matches!(body_ty, Type::BorrowedUnion(_))
                || self.borrowed_place_hint(context, body));
        let result = if borrowed {
            joined.union().map(Type::BorrowedUnion).unwrap_or(joined)
        } else {
            joined
        };
        self.require(result, expected, body.span)?;
        let checked = std::mem::replace(body, Expr::checked(ExprKind::Null, body.span, None));
        *body = Expr::checked(
            ExprKind::Optional {
                base: Box::new(base),
                body: Box::new(checked),
                slot: operator.start,
                union,
                access,
                branch,
                owned,
            },
            body.span,
            Some(result),
        );
        Ok(result)
    }
}
