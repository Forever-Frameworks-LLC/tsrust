//! Regex members share the checked native-call path with imported functions.
use super::*;
impl Analyzer {
 pub(super) fn regex_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&expression.kind else{return Ok(None)};
  let ExprKind::Field(receiver,name)=&callee.kind else{return Ok(None)};
  let ty=self.expression_type_hint(context,receiver);
  let binding=match ty {Some(Type::String)=>crate::stdlib::regex::string_method(name),Some(Type::Record(id))=>crate::stdlib::regex::method(self.program.records[id].native_type,name),_=>None};
  let Some(binding)=binding else{return Ok(None)};
  let mut arguments=vec![(**receiver).clone()];arguments.extend(args.iter().cloned());
  let span=expression.span;
  *expression=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:regex#{}",binding.name)),span)),arguments),span);
  self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
 }
}
