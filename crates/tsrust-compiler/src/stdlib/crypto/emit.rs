use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeBackend,NativeType},native_signatures::{Signature,NativeTypeId},syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 let bytes=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::Bytes));
 let NativeBackend::Bundled{symbol}=binding.backend;
 let mut parameters=Vec::new();let mut arguments=Vec::new();
 for parameter in binding.parameters{
  let name=parameter.name;
  let (ty,value)=match parameter.ty{NativeType::Structured(&Signature::Named(NativeTypeId::Bytes))=>(format!("&Record{}",bytes.unwrap()),format!("&{name}.bytes")),NativeType::Int=>("i64".into(),name.to_owned()),NativeType::String=>("&String".into(),format!("{name}.as_ref()")),_=>unreachable!()};
  parameters.push(format!("{name}:{ty}"));arguments.push(value);
 }
 let call=format!("tsrust_native_crypto::{}({})",symbol.strip_prefix("tsrust_crypto_").unwrap(),arguments.join(","));
 let (ty,body)=match binding.result{
 NativeType::Structured(&Signature::Named(NativeTypeId::Bytes))=>(format!("Record{}",bytes.unwrap()),format!("Record{}{{bytes:{call}}}",bytes.unwrap())),
 NativeType::Int=>("i64".into(),call),NativeType::String=>("String".into(),format!("String::Owned({call})")),
 NativeType::Boolean=>("bool".into(),call),
 NativeType::Structured(signature)=>{
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap()else{unreachable!()};let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::CryptoError)).unwrap();
 let value=format!("Record{}{{bytes:value}}",bytes.unwrap());
 let (ok_order,err_order,error_order)=(order(ok),order(err),order(error));
 (format!("Union{}",result.index()),format!("match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{value}{ok_order}}}),Err((code,message))=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message){error_order}}}{err_order}}})}}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err)))
 },_=>unreachable!()};
 format!("fn {symbol}({})->{ty}{{{body}}}\n",parameters.join(","))
}
