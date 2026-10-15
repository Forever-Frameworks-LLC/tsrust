use super::*;
impl Analyzer {
 pub(super) fn sqlite_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
 let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};
 let ExprKind::Field(base,member)=&callee.kind else{return Ok(None)};
 let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
 let Some(name)=crate::stdlib::sqlite::method(identity,member)else{return Ok(None)};
 args.insert(0,(**base).clone());callee.kind=ExprKind::Name(format!("@native:std:sqlite#{name}"));
 self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
 }
}
