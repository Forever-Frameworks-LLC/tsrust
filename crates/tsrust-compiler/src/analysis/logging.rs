//! Source wrappers own lazy callback/error semantics; native leaves see concrete scalar records.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer{
 pub(super) fn logging_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};
  if let ExprKind::Field(base,name)=&callee.kind && matches!(self.expression_type_hint(context,base),Some(Type::Record(id))if self.program.records[id].native_type==Some(NativeTypeId::Logger)){
   if !crate::stdlib::logging::METHODS.contains(&name.as_str()){return Err(Diagnostic::new("logging-member","Unknown Logger operation",callee.span));}
   let operation=name.clone();args.insert(0,(**base).clone());callee.kind=ExprKind::Name(format!("{}{operation}",crate::stdlib::logging::PREFIX));
   return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:log-runtime")else{return Ok(None)};
  if !context.function_name.starts_with(crate::stdlib::logging::PREFIX){return Err(Diagnostic::new("logging-private","Logging runtime leaves are private",expression.span));}
  if !matches!(binding.name,"child"|"emit"){return Ok(None)}
  Self::arity(args.len(),binding.parameters.len(),expression.span)?;
  for(position,argument)in args.iter_mut().enumerate(){let parameter=&binding.parameters[position];let target=if parameter.ty==crate::native_bindings::NativeType::ObservedValue{None}else{Some(native_type(parameter.ty,&self.program.records))};let actual=self.expression(context,argument,parameter.access,loans,target)?;
   if parameter.ty==crate::native_bindings::NativeType::ObservedValue{
    let Type::Record(record)=actual else{return Err(Diagnostic::new("logging-fields","Logging fields require an owned concrete scalar record",argument.span))};
    if self.program.records[record].native_type.is_some(){return Err(Diagnostic::new("logging-fields","Native resource owners are not logging fields",argument.span));}
    for field in &self.program.records[record].fields{if !matches!(field.ty.unwrap().scalar_base(),Type::Int|Type::Number|Type::Boolean|Type::String){return Err(Diagnostic::new("logging-fields","Logging fields support string, int, number and boolean values",argument.span));}}
    if binding.name=="emit" && !self.fields[record].get("message").is_some_and(|ty|ty.scalar_base()==Type::String){return Err(Diagnostic::new("logging-message","The lazy logging record needs a string message field",argument.span));}
   }
  }
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::NativeBinding(id));expression.ty=Some(result);Ok(Some(result))
 }
}
