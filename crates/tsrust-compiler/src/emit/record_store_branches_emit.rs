//! Convert selected callback store branches before joining native carrier types.
use super::*;
impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn record_callback_store_branches(
        &self,
        fun: &CheckedFunction,
        target: &Expr,
        value: &Expr,
    ) -> Option<String> {
        match &value.kind {
            ExprKind::Conditional{predicate,consequent,alternate}=>{
                let condition=self.expr(fun,predicate,Mode::Copy);
                let left=self.expr(fun,consequent,Mode::Consume);
                let right=self.expr(fun,alternate,Mode::Consume);
                let left=self.record_parameter_replacement(fun,target,consequent,&left)?;
                let right=self.record_parameter_replacement(fun,target,alternate,&right)?;
                Some(format!("if {condition} {{{left}}}else{{{right}}}"))
            }
            ExprKind::Sequence(values)=>{
                let (last,prefix)=values.split_last()?;
                let code=self.expr(fun,last,Mode::Consume);
                let converted=self.record_parameter_replacement(fun,target,last,&code)?;
                let prefix=prefix.iter().map(|value|format!("let _={};",self.expr(fun,value,Mode::Consume))).collect::<String>();
                Some(format!("{{{prefix}{converted}}}"))
            }
            ExprKind::Unary{op:UnaryOp::CallableStore,value,..} if value.ty.is_some_and(|ty|matches!(ty,Type::Closure(origin) if self.borrowed_owners.contains_key(&origin)))=>{
                let code=self.expr(fun,value,Mode::Consume);
                self.record_parameter_replacement(fun,target,value,&code)
            }
            _=>None,
        }
    }
}
