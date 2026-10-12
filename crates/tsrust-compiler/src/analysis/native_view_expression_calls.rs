//! Expression callees use the same native return summaries as named callbacks.
use super::*;

impl Analyzer {
    pub(super) fn remember_native_expression_call(
        &self,context:&mut Context,target:CallTarget,callee:&mut Expr,hint:Type,span:Span,
    )->Result<(),Diagnostic> {
        if !self.native_borrow_types {return Ok(());}
        match target {
            CallTarget::StoredCallableField(family)=>self.remember_native_family_returns(context,family,span),
            CallTarget::ConditionalClosure(origin)=>self.remember_native_closure_returns(context,origin,None,span),
            CallTarget::ClosureMethod(method)=>{
                let Type::Closure(origin)=hint else {return Ok(());};
                let origin=match self.closures[origin].forward {Some(ClosureForward::Once(inner))=>inner,_=>origin};
                self.remember_native_closure_returns(context,origin,Some(method),span)?;
                callee.callable_origin=Some(CallableOrigin::Closure(origin));
                Ok(())
            }
            CallTarget::ForwardFunction(function)=>self.remember_native_function_returns(context,function,span),
            _=>Ok(()),
        }
    }
}
