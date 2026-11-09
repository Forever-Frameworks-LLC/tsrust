use crate::{analysis::CheckedProgram,native_bindings::{NativeBinding,NativeType},native_signatures::NativeTypeId,syntax::Type};
pub(crate) fn bridge(program:&CheckedProgram,binding:&NativeBinding,ordered:&std::collections::HashSet<usize>)->String {
 let order=|id|crate::records::runtime::declaration_order_suffix(program,ordered,id);
 match binding.name {
  "parse"=>{
   let row=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::PathParts)).unwrap();
   format!("fn tsrust_std_path_parse(path:&String)->Record{row}{{let value=tsrust_native_path::parse_parts(path.as_ref(),cfg!(windows));Record{row}{{field0:String::Owned(value.root),field1:String::Owned(value.dir),field2:String::Owned(value.base),field3:String::Owned(value.ext),field4:String::Owned(value.name){}}}}}\n",order(row))
  },
  "format"=>{
   let row=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::PathFormatOptions)).unwrap();
   let values=program.records[row].fields.iter().enumerate().map(|(field,metadata)|{
    let Type::Union(id)=metadata.ty.unwrap()else{unreachable!()};
    let arms=program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}({})=>{}",id.index(),if *ty==Type::Undefined{"_"}else{"value"},if *ty==Type::Undefined{"\"\""}else{"value.as_ref()"})).collect::<Vec<_>>().join(",");
    format!("match &parts.field{field}{{{arms}}}")
   }).collect::<Vec<_>>().join(",");
   format!("fn tsrust_std_path_format(parts:&Record{row})->String{{String::Owned(tsrust_native_path::format_parts({values},cfg!(windows)))}}\n")
  },
  "resolve"=>{
   let NativeType::Structured(signature)=binding.result else{unreachable!()};
   let Type::Union(result)=crate::native_binding_types::resolve(&program.records,signature).unwrap()else{unreachable!()};
   let members=&program.unions[result.index()].members;let [ok,err]=crate::result_identity::variants(&program.records,members).unwrap();let tag=|id|members.iter().position(|ty|*ty==Type::Record(id)).unwrap();
   let error=program.records.iter().position(|record|record.native_type==Some(NativeTypeId::PathError)).unwrap();
   format!("fn tsrust_std_path_resolve(paths:&[&str])->Union{result}{{match tsrust_native_path::resolve_host(paths){{Ok(value)=>Union{result}::V{ok_tag}(Record{ok}{{field0:String::Owned(value){ok_order}}}),Err(error)=>Union{result}::V{err_tag}(Record{err}{{field0:Record{error}{{field0:String::Borrowed(error.code),field1:String::Owned(error.message){error_order}}}{err_order}}})}}}}\n",result=result.index(),ok_tag=tag(ok),err_tag=tag(err),ok_order=order(ok),err_order=order(err),error_order=order(error))
  },
  "separator"=>"fn tsrust_std_path_separator()->String{String::Borrowed(if cfg!(windows){\"\\\\\"}else{\"/\"})}\n".into(),
  "delimiter"=>"fn tsrust_std_path_delimiter()->String{String::Borrowed(if cfg!(windows){\";\"}else{\":\"})}\n".into(),
  _=>String::new(),
 }
}
