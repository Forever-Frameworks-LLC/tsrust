use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeBackend,NativeType},native_signatures::{Signature,NativeTypeId},syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let date=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::CalendarDate)).expect("date declaration");
 let NativeBackend::Bundled{symbol}=binding.backend;
 let mut parameters=Vec::new();let mut arguments=Vec::new();
 for parameter in binding.parameters{
  let name=parameter.name;
  let (ty,value)=match parameter.ty{NativeType::Int=>("i64".into(),name.to_owned()),NativeType::String=>("&String".into(),format!("{name}.as_ref()")),NativeType::Structured(&Signature::Named(NativeTypeId::CalendarDate))=>(format!("&Record{date}"),name.to_owned()),_=>unreachable!()};
  parameters.push(format!("{name}:{ty}"));arguments.push(value);
 }
 let call=format!("tsrust_native_date::{}({})",symbol.strip_prefix("tsrust_date_").unwrap(),arguments.join(","));
 let (ty,body)=match binding.result{
 NativeType::Boolean=>("bool".into(),call),NativeType::Int=>("i64".into(),call),NativeType::String=>("String".into(),format!("String::Owned({call})")),
 NativeType::Structured(&Signature::Named(NativeTypeId::CalendarDate))=>(format!("Record{date}"),call),
 NativeType::Structured(signature)=>{
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap()else{unreachable!()};let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::DateError)).unwrap();
 let value=if program.records[ok].fields[0].ty==Some(Type::String){"String::Owned(value)"}else{"value"};
 (format!("Union{}",result.index()),format!("match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{value}}}),Err((code,message))=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message)}}}})}}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err)))
 },_=>unreachable!()};
 format!("fn {symbol}({})->{ty}{{{body}}}\n",parameters.join(","))
}
