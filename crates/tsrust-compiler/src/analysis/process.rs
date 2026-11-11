//! Process methods use ordinary native contracts and explicit lazy Result tasks.
use super::*;
impl Analyzer{
 pub(super) fn process_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&expression.kind else{return Ok(None)};let ExprKind::Field(receiver,name)=&callee.kind else{return Ok(None)};
  let Some(Type::Record(id))=self.expression_type_hint(context,receiver)else{return Ok(None)};let Some(binding)=crate::stdlib::process::method(self.program.records[id].native_type,name)else{return Ok(None)};
  let mut inputs=vec![(**receiver).clone()];inputs.extend(args.iter().cloned());let span=expression.span;
  if crate::stdlib::process::asynchronous(name){crate::stdlib::futures::lazy_result(expression,"std:process",binding.name,inputs);}else{*expression=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:process#{}",binding.name)),span)),inputs),span);}
  self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
 }
}
