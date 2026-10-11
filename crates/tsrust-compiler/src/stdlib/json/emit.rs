use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let record=|id|program.records.iter().position(|r|r.native_type==Some(id)).unwrap();
 let value=record(NativeTypeId::JsonValue);
 if binding.name=="kind" {return format!("fn tsrust_std_json_kind(value:&Record{value})->String {{String::Borrowed(tsrust_native_json::kind(&value.value))}}\n");}
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap() else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=record(NativeTypeId::JsonError);
 let (parameters,call,payload)=if binding.name=="parse" {
  let limits=record(NativeTypeId::JsonLimits);
  (format!("text:&String,limits:Option<&Record{limits}>"),
   "(|| {let (bytes,depth,entries,nodes)=if let Some(limits)=limits {(limits.field0,limits.field1,limits.field2,limits.field3)} else {(16*1024*1024,64,1_000_000,1_000_000)}; let convert=|value:i64|usize::try_from(value).map_err(|_|tsrust_native_json::invalid_limits());tsrust_native_json::parse(text.as_ref().as_bytes(),tsrust_native_json::Limits{bytes:convert(bytes)?,depth:convert(depth)?,entries:convert(entries)?,nodes:convert(nodes)?})})()".to_string(),
   format!("Record{value}{{value}}"))
 }else{
  (format!("value:&Record{value},max_bytes:Option<i64>"),"usize::try_from(max_bytes.unwrap_or(16*1024*1024)).map_err(|_|tsrust_native_json::invalid_limits()).and_then(|limit|tsrust_native_json::stringify(&value.value,limit))".to_string(),"String::Owned(value)".to_string())
 };
 format!("fn tsrust_std_json_{name}({parameters})->Union{result} {{match {call} {{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}\n",name=binding.name,result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
