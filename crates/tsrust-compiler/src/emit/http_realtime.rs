//! Finite realtime boundaries preserve owned tagged payloads and native options.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn websocket_record(&self,ty:Type)->usize{match ty{Type::Record(record)=>record,_=>ty.union().and_then(|id|self.program.unions[id.index()].members.iter().find_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None})).expect("checked WebSocket record")}}
 fn websocket_field(&self,ty:Type,value:&str,text:bool)->String{
  let Type::Union(id)=ty else{unreachable!("optional WebSocket field")};let nullable=self.program.unions[id.index()].members.contains(&Type::Null);
  let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|{let result=if *ty==Type::Undefined{"None"}else if *ty==Type::Null{"Some(None)"}else if text{"Some(value.as_ref().to_owned())"}else if nullable{"Some(Some(*value))"}else{"Some(*value)"};format!("Union{}::V{tag}(value)=>{result}",id.index())}).collect::<Vec<_>>().join(",");format!("match &{value}{{{arms}}}")
 }
 pub(super) fn websocket_options(&self,ty:Type,value:&str)->String{
  let record=self.websocket_record(ty);let names=["read_buffer_size","write_buffer_size","max_write_buffer_size","max_message_size","max_frame_size","accept_unmasked_frames"];
  let fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)|format!("{}:{}",names[index],self.websocket_field(field.ty.unwrap(),&format!("{value}.field{index}"),false))).collect::<Vec<_>>().join(",");format!("tsrust_native_http::WebSocketOptions{{{fields}}}")
 }
 pub(super) fn websocket_send_message(&self,ty:Type,value:&str)->String{
  let payload=|record:usize,name:&str|match self.program.records[record].native_type{
   Some(NativeTypeId::HttpWebSocketText)=>format!("tsrust_native_http::WebSocketMessage::Text({name}.field1.into_owned())"),
   Some(NativeTypeId::HttpWebSocketBinary)=>format!("tsrust_native_http::WebSocketMessage::Binary({name}.field1.bytes)"),
   Some(NativeTypeId::HttpWebSocketClose)=>{
    let Type::Union(code)=self.program.records[record].fields[1].ty.unwrap()else{unreachable!()};
    let arms=self.program.unions[code.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",code.index(),if *ty==Type::Undefined{"None"}else{"Some(value)"})).collect::<Vec<_>>().join(",");
    let reason=self.http_owned_optional_text(self.program.records[record].fields[2].ty.unwrap(),&format!("{name}.field2"));
    format!("tsrust_native_http::WebSocketMessage::Close{{code:match {name}.field1{{{arms}}},reason:{reason}}}")
   },_=>unreachable!("checked WebSocket message identity"),
  };
  if let Type::Record(record)=ty{return format!("{{let message={value};{}}}",payload(record,"message"));}
  let Type::Union(id)=ty else{unreachable!("checked owned WebSocket message")};let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|{let Type::Record(record)=ty else{unreachable!()};format!("Union{}::V{tag}(message)=>{}",id.index(),payload(*record,"message"))}).collect::<Vec<_>>().join(",");format!("match {value}{{{arms}}}")
 }
 pub(super) fn websocket_received_value(&self,ty:Type,value:&str)->Option<String>{
  let Type::Union(id)=ty else{return None;};let members=&self.program.unions[id.index()].members;
  if !members.iter().any(|ty|matches!(ty,Type::Record(record)if self.program.records[*record].native_type==Some(NativeTypeId::HttpWebSocketText))){return None;}
  let optional=members.contains(&Type::Undefined);let mut arms=Vec::new();
  for(tag,ty)in members.iter().enumerate(){
   if *ty==Type::Undefined{arms.push(format!("None=>Union{}::V{tag}(TsrUndefined)",id.index()));continue;}
   let Type::Record(record)=ty else{unreachable!("checked received message alternatives")};let definition=&self.program.records[*record];
   let(kind,pattern,fields)=match definition.native_type{
    Some(NativeTypeId::HttpWebSocketText)=>("text","Text(data)",format!("field1:String::Owned(data),")),
    Some(NativeTypeId::HttpWebSocketBinary)=>("binary","Binary(data)",format!("field1:{},",self.http_value(definition.fields[1].ty.unwrap(),"data"))),
    Some(NativeTypeId::HttpWebSocketClose)=>("close","Close{code,reason}",format!("field1:{},field2:{},",self.http_value(definition.fields[1].ty.unwrap(),"code"),self.http_value(definition.fields[2].ty.unwrap(),"reason"))),
    _=>unreachable!("checked received message identity"),
   };
   let record=format!("Record{}{{field0:{},{fields}{}}}",self.native_record_declaration(*record),self.string_literal(kind),self.schema_record_order(*record).trim_start_matches(','));
   let pattern=format!("tsrust_native_http::WebSocketMessage::{pattern}");let pattern=if optional{format!("Some({pattern})")}else{pattern};arms.push(format!("{pattern}=>Union{}::V{tag}({record})",id.index()));
  }
  Some(format!("match {value}{{{}}}",arms.join(",")))
 }
 pub(super) fn websocket_connect(&self,url:&str,options:Option<(Type,String)>)->String{
  let Some((ty,value))=options else{return format!("tsrust_native_http::connect_websocket({url},None,std::iter::empty(),tsrust_native_http::WebSocketOptions::default()).await");};
  let record=self.websocket_record(ty);let fields=&self.program.records[record].fields;
  let Type::Union(ca)=fields[0].ty.unwrap()else{unreachable!()};let ca_arms=self.program.unions[ca.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",ca.index(),if *ty==Type::Undefined{"None"}else{"Some(value.as_ref())"})).collect::<Vec<_>>().join(",");
  let Type::Union(headers)=fields[1].ty.unwrap()else{unreachable!()};let header_arms=self.program.unions[headers.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",headers.index(),if *ty==Type::Undefined{"Vec::new()"}else{"value.iter().map(|header|(header.field0.as_ref().to_owned(),header.field1.as_ref().to_owned())).collect::<Vec<_>>()"})).collect::<Vec<_>>().join(",");
  let Type::Union(socket)=fields[2].ty.unwrap()else{unreachable!()};let socket_arms=self.program.unions[socket.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(socket)=>{}",socket.index(),if *ty==Type::Undefined{"tsrust_native_http::WebSocketOptions::default()".into()}else{self.websocket_options(*ty,"socket")})).collect::<Vec<_>>().join(",");
  format!("match {value}{{Some(options)=>{{let ca=match &options.field0{{{ca_arms}}};let headers=match &options.field1{{{header_arms}}};let config=match &options.field2{{{socket_arms}}};tsrust_native_http::connect_websocket({url},ca,headers,config).await}},None=>tsrust_native_http::connect_websocket({url},None,std::iter::empty(),tsrust_native_http::WebSocketOptions::default()).await}}")
 }
}
