//! Locale members share the checked native-call path with imported functions.
use super::*;
impl Analyzer {
 pub(super) fn locale_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&expression.kind else{return Ok(None)};
  let ExprKind::Field(receiver,name)=&callee.kind else{return Ok(None)};
  let ty=self.expression_type_hint(context,receiver);
  let binding=match ty {Some(ty) if self.is_string_observation(ty)=>if name=="localeCompare"{Some(&crate::stdlib::locale::STRING_COMPARE)}else{None},Some(Type::Record(id))=>crate::stdlib::locale::method(self.program.records[id].native_type,name),_=>None};
  let Some(binding)=binding else{return Ok(None)};
  let mut arguments=vec![(**receiver).clone()];arguments.extend(args.iter().cloned());
  let span=expression.span;
  *expression=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:locale#{}",binding.name)),span)),arguments),span);
  self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
 }
}
