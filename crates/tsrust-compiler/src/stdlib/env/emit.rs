use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).expect("environment result") else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let (parameters,call,payload)=if binding.name=="args" {
  let value="value.into_iter().map(String::Owned).collect()";
  let payload=if crate::storage::inline_array(program.records[ok].fields[0].ty.unwrap()){format!("Array::Heap({value})")}else{value.into()};
  ("","args()".to_owned(),payload)
 }else{
  let Type::Union(optional)=program.records[ok].fields[0].ty.unwrap() else {unreachable!()};
  let variants=&program.unions[optional.index()].members;
  let present=variants.iter().position(|ty|*ty==Type::String).unwrap();
  let absent=variants.iter().position(|ty|*ty==Type::Undefined).unwrap();
  ("name:&String","get(name.as_ref())".to_owned(),format!("match value {{Some(value)=>Union{}::V{present}(String::Owned(value)),None=>Union{}::V{absent}(TsrUndefined)}}",optional.index(),optional.index()))
 };
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::EnvError)).unwrap();
 let symbol=if binding.name=="args"{"tsrust_std_env_args"}else{"tsrust_std_env_get"};
 let unicode_message=if binding.name=="args"{"Process argument is not valid Unicode"}else{"Environment value is not valid Unicode"};
 let (ok_order,err_order,error_order)=(order(ok),order(err),order(error));
 format!("fn {symbol}({parameters})->Union{result} {{
 match tsrust_native_env::{call} {{
 Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}{ok_order}}}),
 Err(error)=>{{let (code,message)=match error {{tsrust_native_env::EnvError::InvalidName=>(\"InvalidName\",\"Environment names must be nonempty and contain neither '=' nor NUL\"),tsrust_native_env::EnvError::NotUnicode=>(\"NotUnicode\",{unicode_message:?})}};
 Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message){error_order}}}{err_order}}})}}
 }} }}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
