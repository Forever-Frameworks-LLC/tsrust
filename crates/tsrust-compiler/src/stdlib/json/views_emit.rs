//! Borrowed JSON child adapters retain the source lifetime through Result/absence.
use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let record=|id|program.records.iter().position(|r|r.native_type==Some(id)).unwrap();
 let view=record(if binding.name=="asString"{NativeTypeId::StringView}else{NativeTypeId::JsonView});let error=record(NativeTypeId::JsonError);
 let NativeType::Structured(signature)=binding.result else{unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap() else{unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 if binding.name=="asString" {
  let result_types=members.iter().map(|ty|if *ty==Type::Record(ok){format!("Record{ok}<Record{view}<'a>>")}else{format!("Record{err}")}).collect::<Vec<_>>().join(",");
  return format!("fn tsrust_std_json_asString<'a,B:tsrust_native_json::ViewSource<'a>>(value:B)->Union{result}<{result_types}>{{match tsrust_native_json::read_string(value.json_view()){{Ok(text)=>Union{result}::V{ok_tag}(Record{ok}{{field0:Record{view}{{text}}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err));
 }
 let Type::Union(optional)=program.records[ok].fields[0].ty.unwrap() else{unreachable!()};
 let children=&program.unions[optional.index()].members;
 let present=children.iter().position(|ty|*ty==Type::Record(view)).unwrap();
 let absent=children.iter().position(|ty|*ty==Type::Undefined).unwrap();
 let optional_type=format!("Union{}<Record{view}<'a>>",optional.index());
 let result_types=members.iter().map(|ty|if *ty==Type::Record(ok){format!("Record{ok}<{optional_type}>")}else{format!("Record{err}")}).collect::<Vec<_>>().join(",");
 let (argument,call)=if binding.name=="get"{("key:&String","tsrust_native_json::get_key(value.json_view(),key.as_ref())")}else{("index:i64","tsrust_native_json::get_index(value.json_view(),index)")};
 format!("fn tsrust_std_json_{name}<'a,B:tsrust_native_json::ViewSource<'a>>(value:B,{argument})->Union{result}<{result_types}>{{match {call}{{Ok(child)=>Union{result}::V{ok_tag}(Record{ok}{{field0:match child{{Some(value)=>Union{optional}::V{present}(Record{view}{{value}}),None=>Union{optional}::V{absent}(TsrUndefined)}}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}\n",name=binding.name,result=result.index(),optional=optional.index(),ok_tag=tag(ok),err_tag=tag(err))
}
