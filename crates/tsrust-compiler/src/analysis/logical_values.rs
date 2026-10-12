//! Value-returning logical expressions; boolean guards retain their existing flow checker.
use super::*;

impl Analyzer {
    fn logical_alternatives(&self, ty: Type) -> Vec<Type> {
        ty.union().map(|id| self.unions.members(id)).unwrap_or_else(|| vec![ty])
    }

    fn logical_truthiness(ty: Type) -> (bool, bool) {
        match ty {
            Type::Null | Type::Undefined => (false, true),
            Type::Boolean | Type::Int | Type::Number | Type::String => (true, true),
            Type::Void => (false, false),
            _ => (true, false),
        }
    }

    fn logical_selection(&self, lhs: Type, op: BinaryOp) -> (Vec<Type>, bool) {
        let members = self.logical_alternatives(lhs);
        let selected = members.iter().copied().filter(|ty| {
            let (truthy, falsy) = Self::logical_truthiness(*ty);
            if op == BinaryOp::Or { truthy } else { falsy }
        }).collect();
        let reaches_right = members.iter().any(|ty| {
            let (truthy, falsy) = Self::logical_truthiness(*ty);
            if op == BinaryOp::Or { falsy } else { truthy }
        });
        (selected, reaches_right)
    }

    pub(super) fn logical_value_hint(&self, context: &Context, left: &Expr, right: &Expr, op: BinaryOp, span: Span) -> Option<Type> {
        let lhs = self.expression_type_hint(context, left)?;
        let (mut selected, reaches_right) = self.logical_selection(lhs, op);
        if reaches_right { selected.push(self.expression_type_hint(context, right)?.owning()); }
        let result = self.unions.intern(selected, span).ok()?;
        Some(if matches!(lhs, Type::BorrowedUnion(_)) {
            result.union().map(Type::BorrowedUnion).unwrap_or(result)
        } else { result })
    }

    pub(super) fn logical_value_expression(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Binary { op, left, right, operator } = &mut expr.kind else { unreachable!() };
        let hint = self.expression_type_hint(context, left);
        let (hint_selected, _) = hint.map(|ty| self.logical_selection(ty, *op)).unwrap_or_default();
        // For array/record && rhs, only absent sentinels can be returned from
        // the left. Observing the present owner must not transfer it.
        let left_mode = if matches!(hint, Some(Type::BorrowedUnion(_)))
            || (hint.is_some() && !hint_selected.iter().any(|ty| !ty.is_copy())) {
            Mode::Read
        } else { mode };
        let lhs_expected = if number_literal_context(left) { expected.filter(|ty| matches!(ty, Type::Int | Type::Number)) } else { None };
        let lhs = self.expression(context, left, left_mode, loans, lhs_expected)?;
        if lhs == Type::Void {
            return Err(Diagnostic::new("type", "A void expression cannot supply a logical value. Use a value expression or an explicit conditional statement.", left.span));
        }
        let (mut selected, reaches_right) = self.logical_selection(lhs, *op);
        let left_selected_owned = selected.iter().any(|ty| !ty.is_copy());
        let before = context.names.values().map(|&id| (id, Availability::of(&context.bindings[id]))).collect::<Vec<_>>();
        let skipped_flow = context.flow.clone();
        let skipped_owners = context.capture_loan_owners.clone();
        let mut active = loans.clone();
        if left_mode == Mode::Read && left_selected_owned {
            for root in Self::loan_roots(left, Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, left.span));
            }
        }
        if let Some(place) = crate::flow::place(left, &context.names) {
            let members = self.logical_alternatives(lhs).into_iter().filter(|ty| {
                let (truthy, falsy) = Self::logical_truthiness(*ty);
                if *op == BinaryOp::Or { falsy } else { truthy }
            }).collect::<Vec<_>>();
            if !members.is_empty() && lhs.union().is_some() {
                context.flow.insert(place, crate::flow::Fact { members, origin: Some((span.start, span.end)) });
            }
        }
        let right_expected = expected.or_else(|| {
            if number_literal_context(right) { self.unions.payload(lhs.owning()).or(Some(lhs)).filter(|ty| matches!(ty, Type::Int | Type::Number)) } else { None }
        });
        let rhs = if reaches_right {
            self.expression(context, right, mode, &active, right_expected)?
        } else {
            let mut unreachable = context.clone();
            self.expression(&mut unreachable, right, Mode::Consume, loans, None)?
        };
        if reaches_right { selected.push(rhs.owning()); }
        let result = self.unions.intern(selected, *operator).map_err(|_| Diagnostic::new(
            "logical-result",
            format!("Logical operands can select incompatible value types: {} and {}. Callable alternatives require one compatible payload; use an explicit conditional with compatible results.", self.type_name(lhs), self.type_name(rhs)),
            *operator,
        ))?;
        self.require(result, expected, span)?;
        let borrowed = matches!(lhs, Type::BorrowedUnion(_)) || (reaches_right && matches!(rhs, Type::BorrowedUnion(_)));
        if borrowed && !result.is_copy() && mode != Mode::Read {
            return Err(Diagnostic::new("borrowed-view", "Logical selection does not make borrowed payloads owned. Use clone() or a readonly consumer.", span));
        }
        if reaches_right {
            context.capture_loan_owners.join(&skipped_owners);
            Self::restore_live_capture_loans(context);
            context.flow = crate::flow::join(&skipped_flow, &context.flow);
            for (id, state) in before {
                state.join(Availability::of(&context.bindings[id])).restore(&mut context.bindings[id]);
            }
        } else { context.flow = skipped_flow; }
        Ok(if borrowed { result.union().map(Type::BorrowedUnion).unwrap_or(result) } else { result })
    }
}
