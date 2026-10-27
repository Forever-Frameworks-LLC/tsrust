//! Convert only the checked HTTP boundary; application errors stay out of responses.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 pub(super) fn http_constructor_expression(&self,fun:&CheckedFunction,expr:&Expr,mode:Mode)->Option<String>{
  let Some(Type::Record(record))=expr.ty else{return None;};
  if self.program.records[record].native_type!=Some(NativeTypeId::HttpRouter){return None;}
  let ExprKind::NativeConstructor(args,_)=&expr.kind else{return None;};
  let state=args.first().map(|value|self.expr(fun,value,Mode::Consume)).unwrap_or_else(||"TsrNull".into());
  Some(Self::access_wrapped(expr,mode,format!("tsrust_native_http::Router::with_state({state})")))
 }
 fn http_cookie_options(&self,ty:Type)->String{
  let Type::Record(record)=ty else{unreachable!()};let names=["path","domain","max_age","secure","http_only","same_site","partitioned"];
  let fields=self.program.records[record].fields.iter().enumerate().map(|(field,item)|{
   let Type::Union(id)=item.ty.unwrap()else{unreachable!()};
   let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|{
    let value: String=if *ty==Type::Undefined{"None".into()}else if *ty==Type::String{"Some(value.as_ref().to_owned())".into()}else{"Some(*value)".into()};
    format!("Union{}::V{tag}(value)=>{value}",id.index())
   }).collect::<Vec<_>>().join(",");format!("{}:match &options.field{field}{{{arms}}}",names[field])
  }).collect::<Vec<_>>().join(",");format!("tsrust_native_http::CookieOptions{{{fields}}}")
 }
 fn http_value(&self,ty:Type,value:&str)->String{
  if let Type::Record(id)=ty && self.program.records[id].native_type==Some(NativeTypeId::Bytes){return format!("Record{}{{bytes:{value}}}",self.native_record_declaration(id));}
  if let Type::Array(ArrayElement::Record(id,0))=ty && matches!(self.program.records[id].native_type,Some(NativeTypeId::HttpFormField|NativeTypeId::HttpMultipartPart)) {
   let item=self.http_value(Type::Record(id),"item");let values=format!("({value}).into_iter().map(|item|{item}).collect::<Vec<_>>()");
   return if self.small_record(id){format!("Array::Heap({values})")}else{values};
  }
  if let Type::Record(id)=ty && matches!(self.program.records[id].native_type,Some(NativeTypeId::HttpFormField|NativeTypeId::HttpMultipartPart)){
   let record=&self.program.records[id];let fields=record.fields.iter().enumerate().map(|(index,field)|format!("field{index}:{}",self.http_value(field.ty.unwrap(),&format!("value.{}",["name",if record.native_type==Some(NativeTypeId::HttpFormField){"value"}else{"file_name"},"content_type","data"][index])))).collect::<Vec<_>>().join(",");
   return format!("{{let value={value};Record{}{{{fields}{}}}}}",self.native_record_declaration(id),self.schema_record_order(id));
  }
  if ty==Type::Array(ArrayElement::String(0)){return format!("Array::Heap(({value}).into_iter().map(String::Owned).collect())");}
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
  let binding=&crate::native_bindings::BINDINGS[id];if !matches!(binding.module,"std:http"|"std:signals"){return None;}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().enumerate().map(|(index,arg)|{let param=&binding.parameters[index.min(binding.parameters.len()-1)];let value=self.expr(fun,arg,param.access);if arg.ty==Some(Type::String){if param.access==Mode::Consume{format!("({value}).as_ref().to_owned()")}else{format!("({value}).as_ref()")}}else{value}}).collect::<Vec<_>>();
  let receiver=values.first().cloned().unwrap_or_default();
  let call=match binding.name{
   "listen" if binding.module=="std:signals"=>"tsrust_native_signals::listen()".into(),
   "__receive" if binding.module=="std:signals"=>format!("({receiver}).receive().await"),
   "__router"=>"tsrust_native_http::Router::new()".into(),
   "__app"=>"tsrust_native_http::App::new()".into(),"__client"=>if args.is_empty(){"tsrust_native_http::Client::new()".into()}else{format!("{{let options={receiver};tsrust_native_http::Client::with_ca(options.field0.as_ref())}}")},
   "__configure"|"__routerConfigure"=>format!("{{let app={receiver};let options={};app.configure([options.field0,options.field1,options.field2,options.field3,options.field4,options.field5,options.field6])}}",values[1]),
   "__routerStatic"=>format!("({receiver}).serve_static({}, {}, {})",values[1],values[2],values.get(3).map(String::as_str).unwrap_or("\"\"")),
   "__request"=>format!("tsrust_native_http::Request::new({})",values.join(",")),
   "__text"=>format!("tsrust_native_http::Response::text({})",values.join(",")),
   "__html"|"__json"=>format!("tsrust_native_http::Response::content({}, {}, {:?})",values[0],values[1],if binding.name=="__html"{"text/html; charset=utf-8"}else{"application/json"}),
   "__streamChannel"=>format!("tsrust_native_http::Response::stream_channel({}, {}, |chunk|chunk.bytes)",values[0],values[1]),
   "__stream"=>format!("tsrust_native_http::Response::stream({})",values.join(",")),
   "__routerRoute"|"__routerUse"|"__routerUsePath"=>{
    let Type::Record(router)=args[0].ty.unwrap()else{unreachable!()};let state=self.native_type(self.program.records[router].application.as_ref().unwrap().resolved_arguments[0]);
    let start=match binding.name{"__routerRoute"=>3,"__routerUsePath"=>2,_=>1};
    let route_args=match binding.name{"__routerRoute"=>format!("let method={};let path={};",values[1],values[2]),"__routerUsePath"=>format!("let path={};",values[1]),_=>"let path=\"*\";".into()};
    let mut callbacks=String::new();let mut names=Vec::new();
    for index in start..args.len(){let Type::Closure(origin)=args[index].ty.unwrap()else{unreachable!()};let closure=&self.program.closures[origin];let result=closure.function.result;
    let inputs=match closure.parameters.len(){0=>"",1=>"context",_=>"context,next"};
    let is_task=matches!(result,Type::Closure(id) if self.program.task_origins.contains(&id));
    let body=if is_task{
     let error=self.future_error_type(result,expr.span);let poll=self.future_poll(result,"task",error,expr.span);
     let output=self.program.closures[if let Type::Closure(id)=result{id}else{unreachable!()}].function.result;
     let success=if matches!(output,Type::Void|Type::Undefined){"None".into()}else if let Type::Union(id)=output{
      let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",id.index(),if matches!(ty,Type::Void|Type::Undefined){"None"}else{"Some(value)"})).collect::<Vec<_>>().join(",");
      format!("match response{{{arms}}}")
     }else{"Some(response)".into()};
     let unwrap=if self.result_closures.borrow().contains_key(&origin){"let task=match task{Ok(task)=>task,Err(_)=>return Some(tsrust_native_http::failed())};"}else{""};
     format!("let task=callback.call({inputs});{unwrap}match {poll}{{Ok(response)=>{success},Err(_)=>Some(tsrust_native_http::failed())}}")
    }else if self.result_closures.borrow().contains_key(&origin){format!("Some(match callback.call({inputs}){{Ok(response)=>response,Err(_)=>tsrust_native_http::failed()}})")}else{format!("Some(callback.call({inputs}))")};
    let callback=format!("move|context,next|{{let callback=std::sync::Arc::clone(&callback);Box::pin(async move{{{body}}})}}");
    callbacks.push_str(&format!("let handler{index}:tsrust_native_http::ContextMiddleware<{state}>={{let callback=std::sync::Arc::new({});std::sync::Arc::new({callback})}};",values[index]));
    names.push(format!("handler{index}"));
    }
    let handlers=names.join(",");
    let registration=if binding.name=="__routerRoute"{format!("app.route_many(method,path,vec![{handlers}])")}else if binding.name=="__routerUsePath"{format!("app.use_many(path,vec![{handlers}])")}else{format!("app.use_many(path,vec![{handlers}]).expect(\"the global pattern is valid\")")};
    format!("{{let app={receiver};{route_args}{callbacks}{registration}}}")
   },
   "__route"|"__use"|"__fallback"=>{
    let index=args.len()-1;let Type::Closure(origin)=args[index].ty.unwrap()else{unreachable!()};let closure=&self.program.closures[origin];let result=closure.function.result;
    let error=self.future_error_type(result,expr.span);let poll=self.future_poll(result,"task",error,expr.span);
    let names=if binding.name=="__use"{vec!["request","next"]}else{vec!["request"]};
    let inputs=names.iter().zip(&closure.parameters).map(|(name,id)|match closure.function.bindings[*id].mode{Mode::Read=>format!("&{name}"),Mode::Mutate=>format!("&mut {name}"),_=>name.to_string()}).collect::<Vec<_>>().join(",");
    let unwrap=if self.result_closures.borrow().contains_key(&origin){"let task=match task{Ok(task)=>task,Err(_)=>return tsrust_native_http::failed()};"}else{""};
    let callback=format!("move|{}|{{let callback=std::sync::Arc::clone(&callback);async move{{let task=callback.call({inputs});{unwrap}match {poll}{{Ok(response)=>response,Err(_)=>tsrust_native_http::failed()}}}}}}",names.iter().map(|name|format!("mut {name}")).collect::<Vec<_>>().join(","));
    let registration=if binding.name=="__route"{format!("app.route(method,path,{callback})")}else{format!("app.{}({callback})",if binding.name=="__use"{"use_middleware"}else{"fallback"})};
    let route_args=if binding.name=="__route"{format!("let method={};let path={};",values[1],values[2])}else{String::new()};
    format!("{{let app={receiver};{route_args}let callback=std::sync::Arc::new({});{registration}}}",values[index])
   },
   "__listen"|"__routerListen"=>if args.len()==2{format!("({receiver}).listen({}).await",values[1])}else{format!("{{let app={receiver};let address={};let tls={};app.listen_tls(address,tls.field0.as_ref(),tls.field1.as_ref()).await}}",values[1],values[2])},
   "__wait"=>format!("{{({receiver}).wait().await;Ok::<_,tsrust_native_http::Error>(())}}"),
   "__shutdown"=>format!("{{({receiver}).shutdown().await;Ok::<_,tsrust_native_http::Error>(())}}"),
   "__contextRequestForm"=>format!("({receiver}).request_form().await"),
   "__contextRequestMultipart"=>format!("({receiver}).request_multipart().await"),
   "__contextRequestText"=>format!("({receiver}).request_text().await"),
   "__contextNext"=>format!("{{({}).run({receiver}).await;Ok::<_,tsrust_native_http::Error>(())}}",values[1]),
   "getCookie"=>format!("({receiver}).cookie({})",values[1]),
   "__setCookie"|"__deleteCookie"=>{
    let index=if binding.name=="__setCookie"{3}else{2};
    let options=if let Some(value)=args.get(index){format!("let options={};let cookie_options={};",values[index],self.http_cookie_options(value.ty.unwrap()))}else{"let cookie_options=tsrust_native_http::CookieOptions::default();".into()};
    let value=if index==3{format!("let value={};",values[2])}else{String::new()};
    let call=if index==3{"context.set_cookie(name,value,cookie_options)"}else{"context.delete_cookie(name,cookie_options)"};
    format!("{{let context={receiver};let name={};{value}{options}{call}}}",values[1])
   },
   "__contextSet"=>format!("({receiver}).set({}, {})",values[1],self.expr(fun,&args[2],Mode::Consume)),
   "__contextGet"=>{
    let Type::Record(record)=args[0].ty.unwrap()else{unreachable!()};let Type::Record(variables)=self.program.records[record].application.as_ref().unwrap().resolved_arguments[1]else{unreachable!()};
    let ExprKind::String(key)=&args[1].kind else{unreachable!()};let value=self.program.records[variables].fields.iter().find(|field|field.name.text==*key).unwrap().ty.unwrap();
    return Some(self.cache_optional(expr.ty.unwrap(),value,&format!("({receiver}).get::<{}>({}){}",self.native_type(value),values[1],if value.is_copy(){".copied()"}else{""})));
   },
   "__contextHeader"=>format!("({receiver}).header({}, {})",values[1],values[2]),
   "__contextStatus"=>format!("({receiver}).status({})",values[1]),
   "__contextText"|"__contextHtml"|"__contextJson"=>format!("({receiver}).content({}, {}, {:?})",values[1],values.get(2).map(|value|format!("Some({value})")).unwrap_or_else(||"None".into()),match binding.name{"__contextHtml"=>"text/html; charset=utf-8","__contextJson"=>"application/json",_=>"text/plain; charset=utf-8"}),
   "__next"=>format!("Ok::<_,tsrust_native_http::Error>(({receiver}).run({}).await)",values[1]),
   "__send"=>format!("({receiver}).send({}).await",values[1]),
   "__form"|"__multipart"=>format!("({receiver}).{}().await",binding.name.trim_start_matches('_')),
   "__bodyNext"=>format!("({receiver}).next().await"),
   "__bodyText"|"__requestText"=>format!("({receiver}).text().await"),
   "__responseText"=>format!("({receiver}).text_body().await"),
   "__requestHeader"|"__responseHeader"=>format!("({receiver}).header({})",values[1]),
   "__requestSetHeader"|"__responseSetHeader"=>format!("({receiver}).set_header({}, {})",values[1],values[2]),
   "__requestBody"|"__responseBody"=>format!("({receiver}).body()"),
   "__param"|"__query"|"__queries"=>format!("({receiver}).{}({})",binding.name.trim_start_matches("__"),values[1]),
   "__address"|"__method"|"__url"|"__status"=>format!("({receiver}).{}()",binding.name.trim_start_matches('_')),
   _=>unreachable!("prepared HTTP operation"),
  };
  let result=expr.ty.unwrap();Some(if matches!(binding.result,crate::native_bindings::NativeType::Structured(crate::native_signatures::Signature::Result(..))){self.http_result(result,&call)}else{self.http_value(result,&call)})
 }
 pub(super) fn http_record(&self,output:&mut String,id:usize)->bool{
  let name=match self.program.records[id].native_type{Some(NativeTypeId::HttpRouter)=>"Router",Some(NativeTypeId::HttpContext)=>"Context",Some(NativeTypeId::HttpContextNext)=>"ContextNext",Some(NativeTypeId::HttpApp)=>"App",Some(NativeTypeId::HttpClient)=>"Client",Some(NativeTypeId::HttpRequest)=>"Request",Some(NativeTypeId::HttpResponse)=>"Response",Some(NativeTypeId::HttpServer)=>"Server",Some(NativeTypeId::HttpBody)=>"Stream",Some(NativeTypeId::HttpNext)=>"Next",_=>return false};
  let arguments=if matches!(self.program.records[id].native_type,Some(NativeTypeId::HttpRouter|NativeTypeId::HttpContext)){format!("<{}>",self.native_type(self.program.records[id].application.as_ref().unwrap().resolved_arguments[0]))}else{String::new()};
  writeln!(output,"type Record{id}=tsrust_native_http::{name}{arguments};").unwrap();true
 }
}
