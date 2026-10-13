//! Call continuations retain capture-loan boundaries through callee and arguments.
use super::super::*;

pub(in crate::analysis) enum CallDispatch { Ready(Type), Conditional { publish_type: bool } }

pub(super) struct CalleeFrame {
    capture_boundary: crate::capture_loan_identity::Boundary,
    pub(super) checked_callee: bool,
}

pub(super) struct CallCompletion {
    capture_boundary: crate::capture_loan_identity::Boundary,
    pub(super) checked_callee: bool,
    pub(super) hint: Type,
    span: Span,
}

impl Analyzer {
    pub(in crate::analysis) fn prepare_builtin_call_work(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<CallDispatch, Diagnostic> {
        let ExprKind::Call(callee, _) = &expr.kind else { unreachable!("call dispatch") };
        let stored_field = matches!(callee.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
            && self.expression_type_hint(context, callee).is_some_and(|ty| matches!(ty, Type::StoredCallable(_) | Type::BorrowedCallable(_)));
        if stored_field { return self.builtin_or_method_call(context, expr, loans, expected).map(CallDispatch::Ready); }
        let callable_field = matches!(callee.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
            && self.expression_type_hint(context, callee).is_some_and(|ty| matches!(ty, Type::Closure(_) | Type::Callback(_)));
        if callable_field || matches!(callee.kind,
            ExprKind::Selected { .. } | ExprKind::CoalesceAssign { .. } | ExprKind::SpreadValue(_)
                | ExprKind::Call(_, _) | ExprKind::Sequence(_) | ExprKind::Closure { .. } | ExprKind::Conditional { .. }
                | ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, .. }
                | ExprKind::Unary { op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull, .. }) {
            return Ok(CallDispatch::Conditional { publish_type: true });
        }
        self.builtin_or_method_call(context, expr, loans, expected).map(CallDispatch::Ready)
    }

    pub(super) fn prepare_conditional_callee(&mut self, context: &mut Context, callee: &mut Expr, args: &mut Vec<Expr>) -> Result<CalleeFrame, Diagnostic> {
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let distinct_conditional = if let ExprKind::Conditional { consequent, alternate, .. } = &callee.kind {
            let left = self.callback_value_hint(context, consequent)?;
            let right = self.callback_value_hint(context, alternate)?;
            left.is_none() || right.is_none() || left != right
        } else { false };
        let checked_callee = matches!(callee.kind, ExprKind::Closure { .. } | ExprKind::Call(_, _)) || distinct_conditional;
        if matches!(callee.kind, ExprKind::Closure { .. }) { self.contextual_expression_callee(context, callee, args)?; }
        Ok(CalleeFrame { capture_boundary, checked_callee })
    }

    pub(super) fn finish_conditional_callee(&mut self, callee: &Expr, arguments: usize, span: Span, frame: CalleeFrame, hint: Option<Type>) -> Result<CallCompletion, Diagnostic> {
        let hint = hint.map(|hint| if let Type::Callable(id) = hint { self.result_shape(id) } else { hint });
        let hint = hint.ok_or_else(|| Diagnostic::new("unsupported", "Conditional invocation requires a resolved callable origin.", callee.span))?;
        let hint = if let Type::Closure(origin) = hint
            && let Some(ClosureForward::Native(id)) = self.closures[origin].forward {
            self.native_function_shape(id, span, arguments, false)?
        } else { hint };
        Ok(CallCompletion { capture_boundary: frame.capture_boundary, checked_callee: frame.checked_callee, hint, span })
    }

    pub(super) fn finish_conditional_call(&mut self, context: &mut Context, callee: &mut Expr, frame: CallCompletion, result: (CallTarget, Type)) -> Result<(CallTarget, Type), Diagnostic> {
        let CallCompletion { capture_boundary, checked_callee, hint, span } = frame;
        self.remember_native_expression_call(context,result.0,callee,hint,span)?;
        if checked_callee && !self.expression_result_retains_captures(result.1){capture_boundary.unwind(&mut context.capture_loans);}
        Ok(result)
    }

}
