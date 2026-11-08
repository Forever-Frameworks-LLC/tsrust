//! Regex members share the checked native-call path with imported functions.
use super::*;
impl Analyzer {
 pub(super) fn regex_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&expression.kind else{return Ok(None)};
  let ExprKind::Field(receiver,name)=&callee.kind else{return Ok(None)};
  let ty=self.expression_type_hint(context,receiver);
  let binding=match ty {Some(ty) if self.is_string_observation(ty)=>crate::stdlib::regex::string_method(name),Some(Type::Record(id))=>crate::stdlib::regex::method(self.program.records[id].native_type,name),_=>None};
  let Some(binding)=binding else{return Ok(None)};
  self.prepare_receiver_binding(binding,name,args.len(),expression.span)?;
  let mut arguments=vec![(**receiver).clone()];arguments.extend(args.iter().cloned());
  let span=expression.span;
  *expression=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(format!("@native:std:regex#{}",binding.name)),span)),arguments),span);
  self.expression(context,expression,Mode::Consume,loans,expected).map(Some)
 }
 /// A real receiver can reach a lowered built-in without importing its module.
 /// Check arity by the public member name, then prepare only this recognized
 /// binding through the checked-record append path; hints never create these.
 pub(super) fn prepare_receiver_binding(&mut self,binding:&crate::native_bindings::NativeBinding,member:&str,arguments:usize,span:Span)->Result<(),Diagnostic>{
  let parameters=&binding.parameters[1..];
  let required=parameters.iter().filter(|parameter|!parameter.optional&&!parameter.rest).count();
  if arguments<required||!parameters.iter().any(|parameter|parameter.rest)&&arguments>parameters.len(){
   return Err(Diagnostic::new("arity",format!("{member} expects {required} to {} arguments, found {arguments}.",parameters.len()),span));
  }
  let start=self.program.records.len();
  crate::native_binding_types::prepare(&mut self.program.records,binding,span)?;
  if self.program.records.len()!=start{
   for record in start..self.program.records.len(){self.record_names.insert(self.program.records[record].name.text.clone(),record);}
   append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
  }
  Ok(())
 }
}

#[cfg(test)]
mod tests{
 #[test]
 fn string_regex_calls_without_imports_report_arity_and_type_errors(){
  for annotation in ["string","Format"]{
   for member in ["match","matchAll","search"]{
    for(arguments,code)in [("","arity"),("undefined","type")]{
     let source=format!("type Format='gzip'|'zlib'|'raw';function probe(x:ReadOnly<{annotation}>){{x.{member}({arguments});}}function main(){{}}");
     let failure=match crate::check(&source){Ok(_)=>panic!("invalid regex call accepted: {source}"),Err(error)=>error};
     assert_eq!(failure.code,code,"{source}: {failure:?}");
     if code=="arity"{assert!(failure.message.starts_with(&format!("{member} expects 1 to 1")),"{failure:?}");}
    }
   }
  }
 }
 #[test]
 fn string_locale_compare_without_imports_reports_arity_and_type_errors(){
  for(arguments,code)in [("","arity"),("'b'","arity"),("'b','en'","type"),("'b','en',1","arity")]{
   let source=format!("function main(){{console.log('a'.localeCompare({arguments}));}}");
   let failure=match crate::check(&source){Ok(_)=>panic!("invalid localeCompare accepted: {source}"),Err(error)=>error};
   assert_eq!(failure.code,code,"{source}: {failure:?}");
   if code=="arity"{assert!(failure.message.starts_with("localeCompare expects 2 to 2"),"{failure:?}");}
  }
 }
}
