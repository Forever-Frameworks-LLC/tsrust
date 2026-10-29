use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeBackend,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 let NativeType::Structured(signature)=binding.result else {unreachable!()};
 let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).expect("numeric result") else {unreachable!()};
 let members=&program.unions[result.index()].members;
 let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();
 let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
 let error=program.records.iter().position(|r|r.native_type==Some(NativeTypeId::NumericError)).unwrap();
 let (parameters,call)=match binding.name {
 "toInt"|"toFloat32"=>{
 let NativeType::Structured(input)=binding.parameters[0].ty else {unreachable!()};
 let Type::Union(union)=crate::native_binding_types::resolve(&program.records,input).unwrap() else {unreachable!()};
 let arms=program.unions[union.index()].members.iter().enumerate().map(|(i,ty)|{
 let call=match (binding.name,ty){("toInt",Type::Int)=>"Ok(value)",("toInt",Type::Number)=>"tsrust_numeric::number_to_int(value)",(_,Type::Int)=>"tsrust_numeric::int_to_float32(value)",(_,Type::Number)=>"tsrust_numeric::number_to_float32(value)",_=>unreachable!()};
 format!("Union{}::V{i}(value)=>{call}",union.index())}).collect::<Vec<_>>().join(",");
 (format!("value:Union{}",union.index()),format!("match value {{{arms}}}"))
 },
 "toNumber"=>("value:i64".into(),"tsrust_numeric::to_number(value)".into()),
 "checkSigned"=>("value:i64,bits:i64".into(),"tsrust_numeric::check_signed(value,bits)".into()),
 "checkUnsigned"=>("value:i64,bits:i64".into(),"tsrust_numeric::check_unsigned(value,bits)".into()),
 "parseInt"=>("text:&String".into(),"tsrust_numeric::parse_int(text.as_ref())".into()),_=>unreachable!()};
 let NativeBackend::Bundled{symbol}=binding.backend;
 let (ok_order,err_order,error_order)=(order(ok),order(err),order(error));
 format!("fn {symbol}({parameters})->Union{result}{{match {call}{{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:value{ok_order}}}),Err((code,message))=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(code),field1:String::Borrowed(message){error_order}}}{err_order}}})}}}}",result=result.index(),ok_tag=tag(ok),err_tag=tag(err))
}
