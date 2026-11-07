//! Password Result payloads preserve crypto, admission and worker categories.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn crypto_password_result(&self,ty:Type,call:&str)->String{
  let Type::Union(id)=ty else{unreachable!()};let members=&self.program.unions[id.index()].members;let[ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();let failure=self.program.records[err].fields[0].ty.unwrap();let Type::Union(errors)=failure else{unreachable!()};
  let record=|identity|self.program.unions[errors.index()].members.iter().find_map(|ty|if let Type::Record(record)=ty{(self.program.records[*record].native_type==Some(identity)).then_some(*record)}else{None}).unwrap();
  let value_record=|id:usize,fields:Vec<String>|{let fields=fields.into_iter().enumerate().map(|(index,value)|format!("field{index}:{value}")).collect::<Vec<_>>().join(",");format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id))};
  let failed=|record,fields|value_record(err,vec![self.future_inject(Type::Record(record),failure,&value_record(record,fields))]);
  let crypto=failed(record(NativeTypeId::CryptoError),vec!["String::Borrowed(code)".into(),"String::Borrowed(message)".into()]);
  let admission=failed(record(NativeTypeId::SpawnError),vec!["String::Borrowed(error.code)".into(),"String::Owned(error.to_string())".into()]);
  let worker=record(NativeTypeId::ThreadError);let panic=failed(worker,vec!["String::Borrowed(\"panic\")".into(),"TsrUndefined".into()]);let cancelled=failed(worker,vec!["String::Borrowed(\"cancelled\")".into(),"TsrUndefined".into()]);
  let payload=if self.program.records[ok].fields[0].ty==Some(Type::String){"String::Owned(value)"}else{"value"};let success=value_record(ok,vec![payload.into()]);
  format!("match {call}{{Ok(value)=>Union{}::V{}({success}),Err(error)=>Union{}::V{}(match error{{tsrust_native_crypto_password::Failure::Crypto((code,message))=>{crypto},tsrust_native_crypto_password::Failure::Admission(error)=>{admission},tsrust_native_crypto_password::Failure::Completion(tsrust_native_threads::JoinError::Panic)=>{panic},tsrust_native_crypto_password::Failure::Completion(tsrust_native_threads::JoinError::Cancelled)=>{cancelled},tsrust_native_crypto_password::Failure::Completion(tsrust_native_threads::JoinError::Worker(never))=>match never{{}}}})}}",id.index(),tag(ok),id.index(),tag(err))
 }
 pub(super) fn crypto_password_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:crypto" || !crate::stdlib::crypto::password_operation(binding.name){return None;}
  self.native_bindings.borrow_mut().insert(id);let values=args.iter().zip(binding.parameters).map(|(arg,parameter)|self.expr(fun,arg,parameter.access)).collect::<Vec<_>>();let name=binding.name.trim_start_matches('_');
  let call=if name=="hashPassword"{
   let fields=["memory","iterations","parallelism","output"];
   let options=if let Some(arg)=args.get(2){let optional=self.native_optional_argument(&binding.parameters[2],"crypto_options".into(),arg.ty.unwrap());let record=match arg.ty.unwrap(){Type::Record(record)=>Some(record),Type::Union(union)=>self.program.unions[union.index()].members.iter().find_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None}),_=>None};if let Some(record)=record{
    let fields=fields.iter().enumerate().map(|(index,name)|{let Type::Union(union)=self.program.records[record].fields[index].ty.unwrap()else{unreachable!()};let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,member)|format!("Union{}::V{tag}(value)=>{}",union.index(),if *member==Type::Undefined{"None"}else{"Some(value)"})).collect::<Vec<_>>().join(",");format!("{name}:({optional}).and_then(|options|match options.field{index}{{{arms}}})")}).collect::<Vec<_>>().join(",");format!("tsrust_native_crypto_password::HashOptions{{{fields}}}")
   }else{"tsrust_native_crypto_password::HashOptions{memory:None,iterations:None,parallelism:None,output:None}".into()}}else{"tsrust_native_crypto_password::HashOptions{memory:None,iterations:None,parallelism:None,output:None}".into()};
   let setup=if args.len()>2{format!("let crypto_options={};",values[2])}else{String::new()};format!("{{let crypto_pool={};let crypto_password={};{setup}tsrust_native_crypto_password::hash(crypto_pool,crypto_password.readable_bytes(),{options}).await}}",values[0],values[1])
  }else{
   let policy=if name=="verifyPasswordBounded"{"Some(tsrust_native_crypto_password::VerifyOptions{memory:crypto_policy.field0,iterations:crypto_policy.field1,parallelism:crypto_policy.field2})"}else{"None"};let setup=if args.len()>3{format!("let crypto_policy={};",values[3])}else{String::new()};format!("{{let crypto_pool={};let crypto_password={};let crypto_encoded={};{setup}tsrust_native_crypto_password::verify(crypto_pool,crypto_password.readable_bytes(),crypto_encoded.as_ref(),{policy}).await}}",values[0],values[1],values[2])
  };Some(self.crypto_password_result(expr.ty.unwrap(),&call))
 }
}
