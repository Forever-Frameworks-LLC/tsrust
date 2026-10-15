//! Convert only the checked HTTP boundary; application errors stay out of responses.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn http_value(&self,ty:Type,value:&str)->String{
  if let Type::Record(id)=ty && self.program.records[id].native_type==Some(NativeTypeId::Bytes){return format!("Record{}{{bytes:{value}}}",self.native_record_declaration(id));}
  if ty==Type::String{return format!("String::Owned({value})");}
  if let Type::Union(id)=ty {let members=&self.program.unions[id.index()].members;let absent=members.iter().position(|ty|*ty==Type::Undefined).unwrap();let present=1-absent;let payload=self.http_value(members[present],"value");return format!("match {value}{{Some(value)=>Union{}::V{present}({payload}),None=>Union{}::V{absent}(TsrUndefined)}}",id.index(),id.index());}
  value.into()
 }
 pub(super) fn http_result(&self,ty:Type,call:&str)->String{
  let Type::Union(id)=ty else{unreachable!()};let members=&self.program.unions[id.index()].members;
  let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
  let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();
  let record=|id,fields:String|format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id).trim_start_matches(','));
  let fields=self.program.records[ok].fields.first().map(|field|format!("field0:{},",self.http_value(field.ty.unwrap(),"value"))).unwrap_or_default();
  let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap()else{unreachable!()};
  let failed=record(error,"field0:String::Borrowed(error.code),field1:String::Owned(error.message),".into());
  format!("match {call}{{Ok(value)=>Union{}::V{}({}),Err(error)=>Union{}::V{}({})}}",id.index(),tag(ok),record(ok,fields),id.index(),tag(err),record(err,format!("field0:{failed},")))
 }
 pub(super) fn http_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:http"{return None;}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().zip(binding.parameters).map(|(arg,param)|{let value=self.expr(fun,arg,param.access);if arg.ty==Some(Type::String){if param.access==Mode::Consume{format!("({value}).as_ref().to_owned()")}else{format!("({value}).as_ref()")}}else{value}}).collect::<Vec<_>>();
  let receiver=values.first().cloned().unwrap_or_default();
  let call=match binding.name{
   "__app"=>"tsrust_native_http::App::new()".into(),"__client"=>if args.is_empty(){"tsrust_native_http::Client::new()".into()}else{format!("{{let options={receiver};tsrust_native_http::Client::with_ca(options.field0.as_ref())}}")},
   "__request"=>format!("tsrust_native_http::Request::new({})",values.join(",")),
   "__text"=>format!("tsrust_native_http::Response::text({})",values.join(",")),
   "__html"|"__json"=>format!("tsrust_native_http::Response::content({}, {}, {:?})",values[0],values[1],if binding.name=="__html"{"text/html; charset=utf-8"}else{"application/json"}),
   "__stream"=>format!("tsrust_native_http::Response::stream({})",values.join(",")),
   "__route"|"__use"|"__fallback"=>{
    let index=args.len()-1;let Type::Closure(origin)=args[index].ty.unwrap()else{unreachable!()};let closure=&self.program.closures[origin];let result=closure.function.result;
    let error=self.future_error_type(result,expr.span);let poll=self.future_poll(result,"task",error,expr.span);
    let names=if binding.name=="__use"{vec!["request","next"]}else{vec!["request"]};
    let inputs=names.iter().zip(&closure.parameters).map(|(name,id)|match closure.function.bindings[*id].mode{Mode::Read=>format!("&{name}"),Mode::Mutate=>format!("&mut {name}"),_=>name.to_string()}).collect::<Vec<_>>().join(",");
    let unwrap=if self.result_closures.borrow().contains_key(&origin){"let task=match task{Ok(task)=>task,Err(_)=>return tsrust_native_http::failed()};"}else{""};
    let callback=format!("move|{}|{{let task=callback.call({inputs});async move{{{unwrap}match {poll}{{Ok(response)=>response,Err(_)=>tsrust_native_http::failed()}}}}}}",names.iter().map(|name|format!("mut {name}")).collect::<Vec<_>>().join(","));
    let registration=if binding.name=="__route"{format!("app.route(method,path,{callback})")}else{format!("app.{}({callback})",if binding.name=="__use"{"use_middleware"}else{"fallback"})};
    let route_args=if binding.name=="__route"{format!("let method={};let path={};",values[1],values[2])}else{String::new()};
    format!("{{let app={receiver};{route_args}let callback={};{registration}}}",values[index])
   },
   "__listen"=>if args.len()==2{format!("({receiver}).listen({}).await",values[1])}else{format!("{{let app={receiver};let address={};let tls={};app.listen_tls(address,tls.field0.as_ref(),tls.field1.as_ref()).await}}",values[1],values[2])},
   "__wait"=>format!("{{({receiver}).wait().await;Ok::<_,tsrust_native_http::Error>(())}}"),
   "__shutdown"=>format!("{{({receiver}).shutdown().await;Ok::<_,tsrust_native_http::Error>(())}}"),
   "__next"=>format!("Ok::<_,tsrust_native_http::Error>(({receiver}).run({}).await)",values[1]),
   "__send"=>format!("({receiver}).send({}).await",values[1]),
   "__bodyNext"=>format!("({receiver}).next().await"),
   "__bodyText"|"__requestText"=>format!("({receiver}).text().await"),
   "__responseText"=>format!("({receiver}).text_body().await"),
   "__requestHeader"|"__responseHeader"=>format!("({receiver}).header({})",values[1]),
   "__requestSetHeader"|"__responseSetHeader"=>format!("({receiver}).set_header({}, {})",values[1],values[2]),
   "__requestBody"|"__responseBody"=>format!("({receiver}).body()"),
   "__param"=>format!("({receiver}).param({})",values[1]),
   "__address"|"__method"|"__url"|"__status"=>format!("({receiver}).{}()",binding.name.trim_start_matches('_')),
   _=>unreachable!("prepared HTTP operation"),
  };
  let result=expr.ty.unwrap();Some(if matches!(binding.result,crate::native_bindings::NativeType::Structured(crate::native_signatures::Signature::Result(..))){self.http_result(result,&call)}else{self.http_value(result,&call)})
 }
 pub(super) fn http_record(&self,output:&mut String,id:usize)->bool{
  let name=match self.program.records[id].native_type{Some(NativeTypeId::HttpApp)=>"App",Some(NativeTypeId::HttpClient)=>"Client",Some(NativeTypeId::HttpRequest)=>"Request",Some(NativeTypeId::HttpResponse)=>"Response",Some(NativeTypeId::HttpServer)=>"Server",Some(NativeTypeId::HttpBody)=>"Stream",Some(NativeTypeId::HttpNext)=>"Next",_=>return false};
  writeln!(output,"type Record{id}=tsrust_native_http::{name};").unwrap();true
 }
}
