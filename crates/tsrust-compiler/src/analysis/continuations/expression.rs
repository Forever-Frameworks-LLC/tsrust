//! Expression completion effects are separate from child evaluation.
use super::super::*;

impl Analyzer {
    pub(in crate::analysis) fn expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        super::driver::check_expression(self, context, expr, mode, loans, expected)
    }

    // Preserve finite-literal context when a readonly argument is adapted to a
    // borrowed union. Reuse exactly the ordinary contextual-literal lookup.
    pub(super) fn contextual_literal_member(&self,expr:&Expr,union:UnionId)->Option<Type>{
        self.unions.members(union).into_iter().find(|ty|if let Type::Literal(id,kind)=*ty{
            crate::literal_types::LiteralValue::expression(expr,kind).is_some_and(|value|value.is_ok_and(|value|self.program.records[id].literal_definition.as_ref()==Some(&value)))
        }else{false})
    }

    pub(super) fn prepare_expression_work(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, mut expected: Option<Type>, completion: &mut Vec<Type>) -> Result<super::checked_expression::PreparedExpression, Diagnostic> {
        // Contextual literal wrappers finish from the innermost value outward.
        // They retain no host stack frames and do not repeat completion effects.
        if let Some(Type::Union(union)) = expected {
            let literal = self.contextual_literal_member(expr,union);
            if let Some(literal) = literal { completion.push(expected.unwrap()); expected = Some(literal); }
        }
        if let Some(Type::Literal(id, kind)) = expected && let Some(value) = crate::literal_types::LiteralValue::expression(expr, kind) {
            let value = value?;
            if self.program.records[id].literal_definition.as_ref() != Some(&value) {
                return Err(Diagnostic::new("literal-type", format!("Expected {}, found {}.", self.type_name(Type::Literal(id, kind)), value.text()), expr.span));
            }
            completion.push(expected.unwrap());
            expected = Some(kind.base());
        }
        if matches!(&expr.kind, ExprKind::Unary { op: UnaryOp::RestPack, value, .. } if !matches!(value.kind, ExprKind::SpreadValue(_))) {
            return Ok(if let Some(frame) = self.prepare_rest_literal_work(context, expr, expected)? {
                super::checked_expression::PreparedExpression::Literal(frame)
            } else {
                super::checked_expression::PreparedExpression::Wrapper { frame: super::wrappers::Frame::Rest, mode, expected }
            });
        }
        if matches!(expr.kind, ExprKind::LogicalAssign { .. }) { self.truthy_logical_assignment(context, expr); }
        if matches!(expr.kind, ExprKind::Field(..)) { self.owned_assignment_projection(context, expr); }
        self.owned_assignment_value(context, expr);
        if matches!(expr.kind, ExprKind::Sequence(_)) {
            return Ok(super::checked_expression::PreparedExpression::Sequence { mode, expected });
        }
        if let Some(other) = self.prepare_other_expression(context, expr, mode, loans, expected)? {
            return Ok(match other {
                super::adapters::Other::Ready(ty) => super::checked_expression::PreparedExpression::Ready(ty),
                super::adapters::Other::Adapter(request) => super::checked_expression::PreparedExpression::Adapter(Box::new(request)),
            });
        }
        if matches!(expr.kind, ExprKind::TypeAssertion { .. }) {
            let (frame, expected) = self.prepare_assertion_work(context, expr, mode, expected)?;
            return Ok(super::checked_expression::PreparedExpression::Wrapper { frame, mode, expected });
        }
        // A declared nongeneric call already returns its exact tagged type.
        let expected = if matches!(expected, Some(Type::Union(_)))
            && matches!(&expr.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Name(name) if !context.names.contains_key(name) && self.function_names.contains_key(name) && self.generic_functions.template(name).is_none()))
            && self.expression_type_hint(context, expr) == expected
        { None } else { expected };
        self.lower_scalar_unary_plus(context, expr);
        self.lower_tuple_expression(context, expr, expected)?;
        self.prepare_checked_expression(context, expr, mode, loans, expected)
    }

    pub(super) fn finish_expression(&mut self, context: &mut Context, expr: &mut Expr, checked: Result<Type, Diagnostic>) -> Result<Type, Diagnostic> {
        if checked.as_ref().is_ok_and(|ty|self.is_task_type(*ty))
            && matches!(expr.target,Some(CallTarget::ClosureMethod(_)|CallTarget::Closure(_)|CallTarget::ConditionalClosure(_)|CallTarget::Callback(_)|CallTarget::ConditionalCallback(_)|CallTarget::StoredCallable(_)|CallTarget::StoredCallableField(_)))
            && let ExprKind::Call(_,args)=&expr.kind
            && let Some(params)=self.task_call_argument_contracts(expr) {
            self.retain_task_argument_loans(context,&params,args,expr.span);
        }
        if checked.is_ok() { self.apply_native_call_stores(context, expr)?; }
        if checked.is_ok() && matches!(expr.kind, ExprKind::Call(..) | ExprKind::Index(..) | ExprKind::Binary { .. } | ExprKind::Unary { .. }) {
            // Failure occurs after operand/argument evaluation. Exact spans
            // distinguish a nested call from a parent starting at the same byte.
            context.storage_exits.record_expression(expr.span, &context.bindings);
            match &expr.kind {
                ExprKind::Call(callee, _) => context.storage_exits.record_expression(callee.span, &context.bindings),
                ExprKind::Binary { operator, .. } | ExprKind::Unary { operator, .. } => context.storage_exits.record_expression(*operator, &context.bindings),
                _ => {},
            }
        }
        if checked.is_ok() {
            self.refresh_native_recursive_shapes(context);
        }
        checked
    }

}
