//! Validate source text and preserve its borrow through the public Result.
use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let record=|id|program.records.iter().position(|r|r.native_type==Some(id)).unwrap();
 let view=record(NativeTypeId::JsonRaw);let error=record(NativeTypeId::JsonError);let limits=record(NativeTypeId::JsonLimits);
 let NativeType::Structured(signature)=binding.result else{unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap() else{unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let result_types=members.iter().map(|ty|if *ty==Type::Record(ok){format!("Record{ok}<Record{view}<'a>>")}else{format!("Record{err}")}).collect::<Vec<_>>().join(",");
 format!("fn tsrust_std_json_borrowRaw<'a>(text:&'a String,limits:Option<&Record{limits}>)->Union{result}<{result_types}>{{let result=(||{{let (bytes,depth,entries,nodes)=if let Some(limits)=limits{{(limits.field0,limits.field1,limits.field2,limits.field3)}}else{{(16*1024*1024,64,1_000_000,1_000_000)}};let convert=|value:i64|usize::try_from(value).map_err(|_|tsrust_native_json::invalid_limits());tsrust_native_json::raw::borrow(text.as_ref(),tsrust_native_json::Limits{{bytes:convert(bytes)?,depth:convert(depth)?,entries:convert(entries)?,nodes:convert(nodes)?}})}})();match result{{Ok(fragment)=>Union{result}::V{ok_tag}(Record{ok}{{field0:Record{view}{{fragment}}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Borrowed(error.message),field2:error.offset as i64}}}})}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
