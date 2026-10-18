//! Signal access is ordinary checked native I/O; arithmetic stays in application code.
use super::*;
impl Analyzer {
    pub(super) fn ui_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
        let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};
        let ExprKind::Field(base,member)=&callee.kind else{return Ok(None)};
        let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
        let Some(binding)=identity.and_then(|id|crate::native_bindings::ui_method(id,member)) else{return Ok(None)};
        args.insert(0,(**base).clone());callee.kind=ExprKind::Name(format!("@native:std:ui-runtime#{}",binding.name));
        *expression=crate::stdlib::futures::unwrap(expression.clone());
        self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
    }
}
