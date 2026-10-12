//! Assignment invocation reuses destination storage and existing loan checks.
use super::*;

impl Analyzer {
    /// A named assignment callee replaces storage before borrowing that same slot.
    /// The discarded prefix uses ordinary assignment ownership and disposal checks.
    #[inline(never)]
    pub(super) fn stage_assignment_callee(&mut self, context: &Context, expression: &mut Expr) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, _) = &mut expression.kind else { return Ok(()); };
        let ExprKind::Assign { target, .. } = &callee.kind else { return Ok(()); };
        if !matches!(target.kind, ExprKind::Name(_))
            || !self.expression_type_hint(context, target).is_some_and(|ty| matches!(ty, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_)))
        { return Ok(()); }
        let selected = target.as_ref().clone();
        let prefix = callee.as_ref().clone();
        *callee = Box::new(Expr::new(ExprKind::Sequence(vec![prefix, selected]), callee.span));
        Ok(())
    }
}
