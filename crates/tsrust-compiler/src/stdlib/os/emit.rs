use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn observation(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 let NativeType::Structured(signature)=binding.result else{unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap()else{unreachable!()};
 let members=&program.unions[result.index()].members;let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::OsError)).unwrap();
 let (symbol,call,payload)=if binding.name=="hostname"{("tsrust_std_os_hostname","tsrust_native_hostname::get()","String::Owned(value)")}else{("tsrust_std_os_available_parallelism","tsrust_native_os::available_parallelism()","value")};
 format!("fn {symbol}()->Union{result}{{match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:{payload}{ok_order}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Owned(error.message){error_order}}}{err_order}}})}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err),ok_order=order(ok),err_order=order(err),error_order=order(error))
}
