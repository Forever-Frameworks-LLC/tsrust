//! Coalescing suspends both operands, including validation of unreachable fallbacks.
use super::super::*;

pub(super) struct Left {
    before: Vec<(usize, Availability)>,
    before_flow: crate::flow::Facts,
    mode: Mode,
    expected: Option<Type>,
    undefined_only: bool,
    view: bool,
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

enum Finish { Left(Type), Right, Joined(Box<Join>) }

struct Join {
    mode: Mode,
    expected: Option<Type>,
    payload: Type,
    view: bool,
    undefined_only: bool,
    before: Vec<(usize, Availability)>,
    left_state: Vec<Availability>,
    left_flow: crate::flow::Facts,
    left_owners: crate::capture_loan_owners::Owners,
}

impl Analyzer {
    pub(super) fn prepare_coalesce_work(&mut self, context: &mut Context, expr: &Expr, mode: Mode, expected: Option<Type>) -> Left {
        let undefined_only = matches!(expr.kind, ExprKind::Binary { op: BinaryOp::PatternDefault, .. });
        let ExprKind::Binary { left, .. } = &expr.kind else { unreachable!("coalescing operands"); };
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let before_flow = context.flow.clone();
        let hint = self.expression_type_hint(context, left);
        let left_expected = if hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Closure(_) | Type::Callback(_) | Type::Callable(_) | Type::StoredCallable(_)
            )
        }) {
            expected.or(hint)
        } else if matches!(left.kind, ExprKind::OptionalChain(_)) {
            // An optional chain yields its own nullable result; the fallback,
            // not the chain, satisfies the expected type.
            None
        } else if hint.is_none()
            || number_literal_context(left)
            || matches!(left.kind, ExprKind::Record(_) | ExprKind::Array(_))
        {
            expected.map(|ty| self.unions.payload(ty).unwrap_or(ty))
        } else {
            None
        };
        let view = matches!(hint, Some(Type::BorrowedUnion(_)));
        let lhs_mode = if view { Mode::Read } else { mode };
        Left { before, before_flow, mode, expected, undefined_only, view, access: lhs_mode, operand_expected: left_expected }
    }

    pub(super) fn coalesce_right_work(&mut self, context: &mut Context, expr: &Expr, frame: Left, lhs: Type, loans: &Loans) -> Result<Right, Diagnostic> {
        let Left { before, before_flow, mode, expected, undefined_only, view, access: lhs_mode, .. } = frame;
        let span = expr.span;
        let ExprKind::Binary { left, right, .. } = &expr.kind else { unreachable!("coalescing operands"); };
        let members = lhs
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![lhs]);
        let payload = if undefined_only {let present=members.iter().copied().filter(|ty|*ty!=Type::Undefined).collect::<Vec<_>>();if present.is_empty(){None}else{Some(self.unions.intern(present,left.span)?)} }else{self.unions.present(lhs.owning(), left.span)?.filter(|_| !self.absent_callable_type(lhs))};
        let absent = members
            .iter()
            .any(|ty| *ty==Type::Undefined || (!undefined_only && *ty==Type::Null));
        if lhs == Type::Void {
            return Err(Diagnostic::new(
                "type",
                "A void expression cannot be a coalescing value.",
                left.span,
            ));
        }
        if !absent {
            let mut unreachable = Box::new(context.clone());
            for (id, state) in &before { state.clone().restore(&mut unreachable.bindings[*id]); }
            unreachable.flow = before_flow;
            let rhs_hint = self.expression_type_hint(&unreachable, right).filter(|ty| matches!(ty,
                Type::Closure(_) | Type::Callback(_) | Type::Callable(_) | Type::StoredCallable(_)));
            return Ok(Right { context: Some(unreachable), access: Mode::Consume, expected: rhs_hint,
                loans: loans.clone(), finish: Finish::Left(lhs) });
        }
        let Some(payload) = payload else {
            return Ok(Right { context: None, access: mode, expected, loans: loans.clone(), finish: Finish::Right });
        };
        self.require(payload, expected, left.span)?;
        let fallback_context = expected
            .map(|ty| self.unions.payload(ty).unwrap_or(ty))
            .unwrap_or(payload);
        let rhs_hint = self.expression_type_hint(context, right);
        let mut contextual_members = vec![fallback_context];
        // An int array literal beside a number array payload is a number
        // array, so `numbers ?? [1, 2]` stays one array type.
        let literal_fallback = matches!(right.kind, ExprKind::Array(_))
            && matches!((payload, rhs_hint), (Type::Array(ArrayElement::Number(wanted)), Some(Type::Array(ArrayElement::Int(found)))) if wanted == found)
            || fallback_context.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![fallback_context]).iter().any(|ty| {
                if let Type::Literal(id,kind)=*ty {
                    crate::literal_types::LiteralValue::expression(right,kind).is_some_and(|value|value.is_ok_and(|value|self.program.records[id].literal_definition.as_ref()==Some(&value)))
                }else{false}
            });
        if expected.is_none() && !literal_fallback && !number_literal_context(right) && let Some(ty)=rhs_hint
            && !ty.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![ty]).iter().any(|member|matches!(member,Type::Closure(_)|Type::Callable(_)|Type::Callback(_)|Type::StoredCallable(_))) {contextual_members.push(ty.owning());}
        if let Some(ty) = rhs_hint {
            for member in ty
                .union()
                .map(|id| self.unions.members(id))
                .unwrap_or_else(|| vec![ty])
            {
                if matches!(member, Type::Null | Type::Undefined) {
                    contextual_members.push(member);
                }
            }
        }
        let rhs_context = self.unions.intern(contextual_members, right.span)?;
        let left_state = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let left_flow = context.flow.clone();
        let left_owners = context.capture_loan_owners.clone();
        // The absent arm contains no owned payload to transfer. A stable source
        // place is available in that arm even when the present arm moves it.
        if lhs_mode == Mode::Consume
            && !lhs.is_copy()
            && let Some(place) = crate::flow::place(left, &context.names)
        {
            if let Some((_, state)) = before.iter().find(|(id, _)| *id == place.root) {
                state.clone().restore(&mut context.bindings[place.root]);
            }
            for (place, fact) in &before_flow {
                if place.root == Self::root_binding(left).unwrap_or(usize::MAX) {
                    context.flow.insert(place.clone(), fact.clone());
                }
            }
        }
        if let Some(place) = crate::flow::place(left, &context.names) {
            context.flow.insert(
                place,
                crate::flow::Fact {
                    members: members
                        .iter()
                        .copied()
                        .filter(|ty| *ty==Type::Undefined || (!undefined_only && *ty==Type::Null))
                        .collect(),
                    origin: Some((span.start, span.end)),
                },
            );
        }
        let mut fallback_loans=loans.clone();
        if undefined_only && view {
            for root in Self::loan_roots(left,Mode::Read){fallback_loans.entry(root).or_default().push((Mode::Read,left.span));}
        }
        Ok(Right { context: None,
            access: if view || matches!(rhs_hint, Some(Type::BorrowedUnion(_))) { Mode::Read } else { mode },
            expected: Some(rhs_context), loans: fallback_loans,
            finish: Finish::Joined(Box::new(Join { mode, expected, payload, view, undefined_only,
                before, left_state, left_flow, left_owners })) })
    }

    pub(super) fn finish_coalesce_work(&mut self, context: &mut Context, expr: &mut Expr, frame: Right, rhs: Type) -> Result<Type, Diagnostic> {
        let joined = match frame.finish {
            Finish::Right => return Ok(rhs),
            Finish::Left(lhs) => {
                let ExprKind::Binary { left, operator, .. } = &mut expr.kind else { unreachable!("selected coalescing operand"); };
                let value = std::mem::replace(left, Box::new(Expr::new(ExprKind::Undefined, left.span)));
                expr.binding = value.binding;
                expr.kind = ExprKind::Unary { op: UnaryOp::CoalesceSelected, operator: *operator, value };
                return Ok(lhs);
            }
            Finish::Joined(joined) => joined,
        };
        let Join { mode, expected, payload, view, undefined_only, before, left_state, left_flow, left_owners } = *joined;
        let span = expr.span;
        let ExprKind::Binary { right, .. } = &expr.kind else { unreachable!("coalescing fallback"); };
        let joined = self.unions.intern([payload, rhs.owning()], right.span)?;
        self.require(joined, expected, span)?;
        let borrowed = view || matches!(rhs, Type::BorrowedUnion(_));
        let result = if borrowed {
            joined.union().map(Type::BorrowedUnion).unwrap_or(joined)
        } else {
            expected.unwrap_or(joined)
        };
        if borrowed && !result.is_copy() && mode == Mode::Consume {
            return Err(Diagnostic::new(
                "borrowed-view",
                "Coalescing a borrowed payload cannot transfer ownership; use clone() or a readonly consumer.",
                span,
            ));
        }
        if borrowed
            && !rhs.is_copy()
            && !matches!(rhs, Type::BorrowedUnion(_))
            && !undefined_only
            && !Self::stable_at_receiver(right)
            && !matches!(right.kind, ExprKind::String(_))
        {
            return Err(Diagnostic::new(
                "borrowed-view",
                "Join borrowed values only with stable readonly sources or string literals; clone the view before selecting a fresh owner.",
                right.span,
            ));
        }
        context.capture_loan_owners.join(&left_owners);
        Self::restore_live_capture_loans(context);
        context.flow = crate::flow::join(&left_flow, &context.flow);
        for ((id, _), state) in before.into_iter().zip(left_state) {
            state
                .join(Availability::of(&context.bindings[id]))
                .restore(&mut context.bindings[id]);
        }
        Ok(result)
    }
}
