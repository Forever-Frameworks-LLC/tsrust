//! Checked I/O calls preserve explicit Result values across lazy task suspension.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn io_value(&self,ty:Type,value:&str)->String{
  if let Type::Record(id)=ty && self.program.records[id].native_type==Some(NativeTypeId::Bytes){return format!("Record{}{{bytes:{value}}}",self.native_record_declaration(id));}
  if ty==Type::String{return format!("String::Owned({value})");}
  if let Type::Union(id)=ty{let members=&self.program.unions[id.index()].members;let absent=members.iter().position(|ty|*ty==Type::Undefined).expect("checked optional output");let present=1-absent;let payload=self.io_value(members[present],"value");return format!("match {value}{{Some(value)=>Union{id}::V{present}({payload}),None=>Union{id}::V{absent}(TsrUndefined)}}",id=id.index());}
  value.into()
 }
 fn io_result(&self,ty:Type,call:&str)->String{
  let Type::Union(id)=ty else{unreachable!("checked I/O Result")};let members=&self.program.unions[id.index()].members;let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
  let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
  let record=|id,fields:String|format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id).trim_start_matches(','));
  let fields=self.program.records[ok].fields.first().map(|field|format!("field0:{},",self.io_value(field.ty.unwrap(),"value"))).unwrap_or_default();
  let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap()else{unreachable!()};let Type::Union(os)=self.program.records[error].fields[3].ty.unwrap()else{unreachable!()};
  let integer=self.program.unions[os.index()].members.iter().position(|ty|*ty==Type::Int).unwrap();let absent=self.program.unions[os.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
  let failed=record(error,format!("field0:String::Borrowed(error.code),field1:String::Borrowed(error.operation),field2:String::Owned(error.message),field3:match error.os_code{{Some(code)=>Union{}::V{integer}(code),None=>Union{}::V{absent}(TsrUndefined)}},",os.index(),os.index()));
  format!("match {call}{{Ok(value)=>Union{}::V{}({}),Err(error)=>Union{}::V{}({})}}",id.index(),tag(ok),record(ok,fields),id.index(),tag(err),record(err,format!("field0:{failed},")))
 }
 pub(super) fn io_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:io"{return None;}if let Some(call)=self.compression_io_native(fun,expr,id,args){return Some(call);}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().zip(binding.parameters).map(|(arg,parameter)|self.expr(fun,arg,parameter.access)).collect::<Vec<_>>();
  let receiver=values.first().cloned().unwrap_or_default();let optional=|index:usize|args.get(index).map(|arg|self.native_optional_argument(&binding.parameters[index],values[index].clone(),arg.ty.unwrap())).unwrap_or("None".into());
  let name=binding.name.trim_start_matches('_');
  let call=match name{
   "reader"=>{let Some(Type::Record(input))=args[0].ty else{unreachable!()};match self.program.records[input].native_type{
    Some(NativeTypeId::File)=>format!("tsrust_native_io::Reader::file(({receiver}).handle)"),Some(NativeTypeId::BufferedReader)=>format!("tsrust_native_io::Reader::buffered(({receiver}).handle)"),Some(NativeTypeId::Bytes)=>format!("tsrust_native_io::Reader::bytes(({receiver}).bytes)"),Some(NativeTypeId::HttpBody)=>format!("tsrust_native_http_io::reader({receiver})"),_=>unreachable!("checked reader adapter")}},
   "writer"=>{let Some(Type::Record(input))=args[0].ty else{unreachable!()};format!("tsrust_native_io::Writer::{}(({receiver}).handle)",if self.program.records[input].native_type==Some(NativeTypeId::File){"file"}else{"buffered"})},
   "stdout"|"stderr"=>format!("tsrust_native_io::Writer::{name}()"),"stdin"=>"tsrust_native_io::Stdin::new()".into(),
   "body"=>format!("tsrust_native_http_io::body({receiver})"),
   "read"|"readStdin"=>format!("({receiver}).read({}.unwrap_or(tsrust_native_io::BUFFER_SIZE as i64)){}",optional(1),if name=="read"{".await"}else{""}),
   "readAll"=>format!("({receiver}).read_all({}).await",optional(1)),
   "readInto"|"readExact"=>format!("({receiver}).{}(({}).writable_bytes()).await",if name=="readInto"{"read_into"}else{"read_exact"},values[1]),
   "write"|"writeAll"=>format!("({receiver}).{}(({}).readable_bytes()).await",if name=="write"{"write"}else{"write_all"},values[1]),
   "flush"|"close"=>format!("({receiver}).{name}().await"),
   "copy"=>{let tcp=|index:usize|matches!(args[index].ty,Some(Type::Record(record))if self.program.records[record].native_type==Some(NativeTypeId::TcpStream));if tcp(0)||tcp(1){format!("tsrust_native_net_io::copy(tsrust_native_net_io::CopyReader::{}({receiver}),tsrust_native_net_io::CopyWriter::{}({}),{}.unwrap_or(tsrust_native_io::BUFFER_SIZE as i64),{}).await",if tcp(0){"Tcp"}else{"Io"},if tcp(1){"Tcp"}else{"Io"},values[1],optional(2),optional(3))}else{format!("tsrust_native_io::copy({receiver},{},{}.unwrap_or(tsrust_native_io::BUFFER_SIZE as i64),{}).await",values[1],optional(2),optional(3))}},
   "lines"=>format!("tsrust_native_io::LineReader::new({receiver},{})",optional(1)),
   "readLine"=>format!("({receiver}).read_line().await"),"readLineStdin"=>format!("({receiver}).read_line({})",optional(1)),
   _=>unreachable!("registered I/O operation"),
  };
  Some(if matches!(binding.result,crate::native_bindings::NativeType::Structured(crate::native_signatures::Signature::Result(..))){self.io_result(expr.ty.unwrap(),&call)}else{call})
 }
}
