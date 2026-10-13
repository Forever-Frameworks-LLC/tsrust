//! Logical operands suspend with distinct boolean-flow and value-selection state.
use super::super::*;

pub(super) struct Left {
    boolean: bool,
    mode: Mode,
    expected: Option<Type>,
    pub(super) access: Mode,
    pub(super) operand_expected: Option<Type>,
}

pub(super) struct Right {
    pub(super) context: Option<Box<Context>>,
    pub(super) access: Mode,
    pub(super) expected: Option<Type>,
    pub(super) loans: Loans,
    finish: Finish,
}

enum Finish {
    Boolean { before: Vec<(usize, Availability)>, skipped: crate::flow::Facts },
    Value { mode: Mode, expected: Option<Type>, lhs: Type, selected: Vec<Type>, reaches_right: bool,
        before: Vec<(usize, Availability)>, skipped_flow: crate::flow::Facts,
        skipped_owners: crate::capture_loan_owners::Owners },
}

impl Analyzer {
    pub(super) fn prepare_logical_work(&mut self, context: &mut Context, expr: &Expr, mode: Mode, expected: Option<Type>) -> Left {
        let ExprKind::Binary { op, left, right, .. } = &expr.kind else { unreachable!("logical operands"); };
        if self.expression_type_hint(context, left) == Some(Type::Boolean)
            && self.expression_type_hint(context, right).is_none_or(|ty| ty == Type::Boolean) {
            return Left { boolean: true, mode, expected, access: Mode::Copy, operand_expected: Some(Type::Boolean) };
        }
        let hint = self.expression_type_hint(context, left);
        let (hint_selected, _) = hint.map(|ty| self.logical_selection(ty, *op)).unwrap_or_default();
        // For array/record && rhs, only absent sentinels can be returned from
        // the left. Observing the present owner must not transfer it.
        let left_mode = if matches!(hint, Some(Type::BorrowedUnion(_)))
            || (hint.is_some() && !hint_selected.iter().any(|ty| !ty.is_copy())) {
            Mode::Read
        } else { mode };
        let lhs_expected = if number_literal_context(left) { expected.filter(|ty| matches!(ty, Type::Int | Type::Number)) } else { None };
        Left { boolean: false, mode, expected, access: left_mode, operand_expected: lhs_expected }
    }

    pub(super) fn logical_right_work(&mut self, context: &mut Context, expr: &Expr, frame: Left, lhs: Type, loans: &Loans) -> Result<Right, Diagnostic> {
        let Left { boolean, mode, expected, access: left_mode, .. } = frame;
        let span = expr.span;
        let ExprKind::Binary { op, left, right, operator } = &expr.kind else { unreachable!("logical operands"); };
        if boolean {
            let (op, operator, actual) = (*op, *operator, lhs);
            self.require(actual, Some(Type::Boolean), operator)?;
            let (left_true, left_false) = Self::condition_outcomes(context, left);
            let before = context
                .names
                .values()
                .map(|&id| (id, Availability::of(&context.bindings[id])))
                .collect::<Vec<_>>();
            let skipped = if op == BinaryOp::And {
                left_false.clone()
            } else {
                left_true.clone()
            };
            context.flow = if op == BinaryOp::And {
                left_true
            } else {
                left_false
            };
            return Ok(Right { context: None, access: Mode::Copy, expected: Some(Type::Boolean), loans: loans.clone(),
                finish: Finish::Boolean { before, skipped } });
        }
        if lhs == Type::Void {
            return Err(Diagnostic::new("type", "A void expression cannot supply a logical value. Use a value expression or an explicit conditional statement.", left.span));
        }
        let (selected, reaches_right) = self.logical_selection(lhs, *op);
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
        let child_context = if reaches_right { None } else { Some(Box::new(context.clone())) };
        Ok(Right { context: child_context, access: if reaches_right { mode } else { Mode::Consume },
            expected: if reaches_right { right_expected } else { None }, loans: if reaches_right { active } else { loans.clone() },
            finish: Finish::Value { mode, expected, lhs, selected, reaches_right, before, skipped_flow, skipped_owners } })
    }

    pub(super) fn finish_logical_work(&mut self, context: &mut Context, expr: &Expr, frame: Right, rhs: Type) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Binary { op, right, operator, .. } = &expr.kind else { unreachable!("logical operands"); };
        let (mode, expected, lhs, mut selected, reaches_right, before, skipped_flow, skipped_owners) = match frame.finish {
            Finish::Boolean { before, skipped } => {
                let (op, operator, actual, expr_span) = (*op, *operator, rhs, span);
                self.require(actual, Some(Type::Boolean), operator)?;
                let (right_true, right_false) = Self::condition_outcomes(context, right);
                for (id, state) in before {
                    state
                        .join(Availability::of(&context.bindings[id]))
                        .restore(&mut context.bindings[id]);
                }
                let outcomes = if op == BinaryOp::And {
                    (right_true, crate::flow::join(&skipped, &right_false))
                } else {
                    (crate::flow::join(&skipped, &right_true), right_false)
                };
                context.flow = crate::flow::join(&outcomes.0, &outcomes.1);
                context
                    .condition_flows
                    .insert((expr_span.start, expr_span.end), outcomes);
                return Ok(Type::Boolean);
            }
            Finish::Value { mode, expected, lhs, selected, reaches_right, before, skipped_flow, skipped_owners } =>
                (mode, expected, lhs, selected, reaches_right, before, skipped_flow, skipped_owners),
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
