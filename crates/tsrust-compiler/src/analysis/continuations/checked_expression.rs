//! State around typed expression evaluation and failure-point publication.
use super::super::*;

pub(super) struct CheckedExpressionFrame {
    pub(super) invocation: bool,
    native_constructor: bool,
    bounds_or_nonnull: bool,
    integer_operation: bool,
    pub(super) mode: Mode,
    pub(super) expected: Option<Type>,
}
pub(super) enum PreparedExpression { Adapter(Box<super::adapters::Request>), Wrapper{frame:super::wrappers::Frame,mode:Mode,expected:Option<Type>}, Literal(super::literals::LiteralFrame), Optional{mode:Mode,expected:Option<Type>}, Ready(Type), Sequence { mode: Mode, expected: Option<Type> }, Evaluate(CheckedExpressionFrame), Prelude { mode:Mode, expected:Option<Type>, frame:super::prelude::Frame } }

impl Analyzer {
    pub(super) fn prepare_checked_expression(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<PreparedExpression, Diagnostic> {
        if let Some(prelude) = self.checked_expression_prelude(context, expr, mode, loans, expected)? {
            return Ok(match prelude{
                super::prelude::Prelude::Optional{mode,expected}=>PreparedExpression::Optional{mode,expected},
                super::prelude::Prelude::Ready(ty)=>{
                    expr.json_string_view=self.json_string_proof(context,expr).is_some() || self.is_string_view(context,expr);
                    PreparedExpression::Ready(ty)
                },
                super::prelude::Prelude::Child{mode,expected,frame}=>PreparedExpression::Prelude{mode,expected,frame},
            });
        }
        // The driver owns suspended frames; source nesting consumes no host frames.
        let invocation=matches!(expr.kind,ExprKind::Call(_, _));
        let native_constructor=matches!(expr.kind,ExprKind::NativeConstructor(..));
        let bounds_or_nonnull=matches!(expr.kind,ExprKind::Index(_,_)|ExprKind::Unary{op:UnaryOp::NonNull,..});
        let integer_operation=matches!(&expr.kind,ExprKind::Unary{op:UnaryOp::Negate,..}) || matches!(&expr.kind,ExprKind::Binary{op,..} if crate::error_effects::integer_operation(*op));
        Ok(PreparedExpression::Evaluate(CheckedExpressionFrame { invocation, native_constructor, bounds_or_nonnull, integer_operation, mode, expected }))
    }

    pub(super) fn finish_checked_expression(&mut self, context: &mut Context, expr: &mut Expr, frame: CheckedExpressionFrame, result: Result<Type, Diagnostic>) -> Result<Type, Diagnostic> {
        let CheckedExpressionFrame { invocation, native_constructor, bounds_or_nonnull, integer_operation, mode, expected } = frame;
        if result.is_ok() && (invocation || native_constructor || bounds_or_nonnull || (integer_operation && result.as_ref().is_ok_and(|ty|*ty==Type::Int))) {exception_state::record_point(context,expr.span);}
        let result = self.checked_expression_result(context, expr, mode, expected, result);
        result
    }

}
