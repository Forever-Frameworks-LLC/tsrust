use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeBackend,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>,inline_pairs:bool)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).expect("environment result") else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let payload_type=program.records[ok].fields[0].ty.unwrap();
 let optional=|value:&str|{
  let Type::Union(id)=payload_type else{unreachable!()};let members=&program.unions[id.index()].members;
  let present=members.iter().position(|ty|*ty==Type::String).unwrap();let absent=members.iter().position(|ty|*ty==Type::Undefined).unwrap();
  format!("match {value}{{Some(value)=>Union{}::V{present}(String::Owned(value)),None=>Union{}::V{absent}(TsrUndefined)}}",id.index(),id.index())
 };
 let array=|value:String|if crate::storage::inline_array(payload_type)||inline_pairs{format!("Array::Heap({value})")}else{value};
 let (parameters,call,payload)=match binding.name {
  "get"=>("name:&String","get(name.as_ref())".to_owned(),optional("value")),
  "homeDir"=>("","home_dir()".into(),optional("value")),
  "args"=>("","args()".into(),array("value.into_iter().map(String::Owned).collect()".into())),
  "vars"=>{
   let row=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::EnvVar)).unwrap();
   let fields=format!("Record{row}{{field0:String::Owned(name),field1:String::Owned(value){}}}",order(row));
   ("","vars()".into(),array(format!("value.into_iter().map(|(name,value)|{fields}).collect()")))
  },
  "currentDir"=>("","current_dir()".into(),"String::Owned(value)".into()),
  "currentExe"=>("","current_exe()".into(),"String::Owned(value)".into()),
  "tempDir"=>("","temp_dir()".into(),"String::Owned(value)".into()),
  _=>unreachable!("registered environment observation"),
 };
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::EnvError)).unwrap();
 let NativeBackend::Bundled{symbol}=binding.backend;
 let unicode_message=match binding.name{"args"=>"Process argument is not valid Unicode","get"=>"Environment value is not valid Unicode","vars"=>"Environment name or value is not valid Unicode",_=>"Observed path is not valid Unicode"};
 let (ok_order,err_order,error_order)=(order(ok),order(err),order(error));
 format!("fn {symbol}({parameters})->Union{result}{{match tsrust_native_env::{call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}{ok_order}}}),Err(error)=>{{let(code,message)=match error{{tsrust_native_env::EnvError::InvalidName=>(\"InvalidName\",String::Borrowed(\"Environment names must be nonempty and contain neither '=' nor NUL\")),tsrust_native_env::EnvError::NotUnicode=>(\"NotUnicode\",String::Borrowed({unicode_message:?})),tsrust_native_env::EnvError::Io(message)=>(\"Io\",String::Owned(message))}};Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:message{error_order}}}{err_order}}})}}}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
