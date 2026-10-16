use super::*;
impl Analyzer {
 pub(super) fn sql_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
 let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};
 if let ExprKind::Field(base,member)=&callee.kind {
 let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
 let Some(name)=crate::stdlib::sql::method(identity,member)else{return Ok(None)};
 if let Some(parameters)=args.get_mut(1) && matches!(parameters.kind,ExprKind::Array(_)) {
  let crate::native_bindings::NativeType::Structured(signature)=crate::stdlib::sql::EXECUTE.parameters[2].ty else{unreachable!()};
  let annotation=crate::native_type_resolution::project(&mut self.program.records,signature,parameters.span)?;
  let value=parameters.clone();
  parameters.kind=ExprKind::TypeAssertion{value:Box::new(value),annotation:Box::new(annotation),satisfies:false};
 }
 args.insert(0,(**base).clone());
 if crate::stdlib::sql::asynchronous(name){let args=std::mem::take(args);crate::stdlib::futures::lazy_module(expression,"std:sql",name,args);
 }else{callee.kind=ExprKind::Name(format!("@native:std:sql#{name}"));}
 return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
 }
 if let ExprKind::Name(name)=&callee.kind && let Some((_,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:sql") && crate::stdlib::sql::asynchronous(binding.name) && !context.task_body{return Err(Diagnostic::new("async-context","SQL I/O must run inside an async task.",expression.span));}
 if let ExprKind::Name(name)=&callee.kind && let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:sql" && matches!(binding.name,"__cursor"|"__query"|"__execute"|"__txquery"|"__txexecute")) {
  if !binding.accepts_arity(args.len()){return Err(Diagnostic::new("arity","SQL query and execute require SQL text and optional parameters.",expression.span));}
  let mut active=loans.clone();
  for (index,(argument,parameter)) in args.iter_mut().zip(binding.parameters).enumerate(){
   let expected=if index==2{None}else{Some(native_type(parameter.ty,&self.program.records))};
   let actual=self.expression(context,argument,parameter.access,&active,expected)?;
   if index!=2{self.require(actual,expected,argument.span)?;}
   if index==2 {
    let Type::Array(element)=actual else{return Err(Diagnostic::new("sql-parameters","SQL parameters must be an array of int, number, boolean, string, null or Bytes.",argument.span));};
    let members=if let Type::Union(id)=element.ty(){self.unions.members(id)}else{vec![element.ty()]};
    if members.iter().any(|ty|!matches!(ty,Type::Int|Type::Number|Type::Boolean|Type::String|Type::Null) && !matches!(ty,Type::Record(id) if self.program.records[*id].native_type==Some(crate::native_signatures::NativeTypeId::Bytes))){return Err(Diagnostic::new("sql-parameters","SQL parameters accept only int, number, boolean, string, null or Bytes.",argument.span));}
   }
   for root in Self::loan_roots(argument,parameter.access){active.entry(root).or_default().push((parameter.access,argument.span));}
  }
  let result=native_type(binding.result,&self.program.records);self.remember_native_call_borrow(context,args,binding,result)?;self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));return Ok(Some(result));
 }
 Ok(None)
 }
}
