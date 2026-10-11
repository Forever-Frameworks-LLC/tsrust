use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding)->String {
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).expect("environment result") else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let Type::Union(optional)=program.records[ok].fields[0].ty.unwrap() else {unreachable!()};
 let variants=&program.unions[optional.index()].members;
 let present=variants.iter().position(|ty|*ty==Type::String).unwrap();
 let absent=variants.iter().position(|ty|*ty==Type::Undefined).unwrap();
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::EnvError)).unwrap();
 format!("mod tsrust_native_env {{ {} }}
 fn tsrust_std_env_get(name:&String)->Union{result} {{
 match tsrust_native_env::get(name.as_ref()) {{
 Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:match value {{Some(value)=>Union{optional}::V{present}(String::Owned(value)),None=>Union{optional}::V{absent}(TsrUndefined)}}}}),
 Err(error)=>{{let (code,message)=match error {{tsrust_native_env::EnvError::InvalidName=>(\"InvalidName\",\"Environment names must be nonempty and contain neither '=' nor NUL\"),tsrust_native_env::EnvError::NotUnicode=>(\"NotUnicode\",\"Environment value is not valid Unicode\")}};
 Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message)}}}})}}
 }} }}",include_str!("runtime.rs"),result=result.index(),optional=optional.index(),ok_tag=tag(ok),err_tag=tag(err))
}
