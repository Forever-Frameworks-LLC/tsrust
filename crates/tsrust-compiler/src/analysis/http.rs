//! HTTP callbacks reuse task transfer checks and ordinary receiver loans.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
 pub(super) fn http_constructor(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::NativeConstructor(args,Some(annotation))=&mut expression.kind else{return Ok(None);};
  let ty=self.resolve_value_annotation(annotation,false)?;let Type::Record(record)=ty else{return Ok(None);};
  if self.program.records[record].native_type!=Some(NativeTypeId::HttpRouter){return Ok(None);}
  let arguments=self.program.records[record].application.as_ref().unwrap().resolved_arguments.clone();
  self.http_shared_data(arguments[0],expression.span)?;
  if arguments[1]!=Type::Null && !matches!(arguments[1],Type::Record(id) if self.program.records[id].native_type.is_none()){return Err(Diagnostic::new("http-state","Request variables require a record type or null.",expression.span));}
  Self::arity(args.len(),usize::from(arguments[0]!=Type::Null),expression.span)?;
  if let Some(value)=args.first_mut(){let actual=self.expression(context,value,Mode::Consume,loans,Some(arguments[0]))?;self.require(actual,Some(arguments[0]),value.span)?;}
  self.require(ty,expected,expression.span)?;
  expression.target=Some(CallTarget::NativeBinding(crate::native_bindings::by_symbol("@native:std:http#__router").unwrap().0));Ok(Some(ty))
 }
 pub(super) fn http_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
  if matches!(self.expression_type_hint(context,callee),Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::HttpContextNext)) {
   Self::arity(args.len(),0,expression.span)?;
   let contexts=context.names.iter().filter(|(_,id)|matches!(context.bindings[**id].ty,Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::HttpContext))).map(|(name,_)|name.clone()).collect::<Vec<_>>();
   let [name]=contexts.as_slice() else{return Err(Diagnostic::new("http-context","next() requires one visible request Context.",expression.span));};
   let inputs=vec![Expr::new(ExprKind::Name(name.clone()),expression.span),(**callee).clone()];
   crate::stdlib::futures::lazy_module(expression,"std:http","__contextNext",inputs);
   return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
  }
  if let ExprKind::Field(base,member)=&callee.kind {
   let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
   let Some(mut name)=crate::stdlib::http::method(identity,member)else{return Ok(None);};
   let request_json=name=="__requestJson";
   if request_json{Self::arity(args.len(),0,expression.span)?;name="__requestText";}
   let mut receiver=(**base).clone();
   if name=="__routerUse" && args.first().is_some_and(|arg|matches!(arg.kind,ExprKind::String(_)) || self.expression_type_hint(context,arg)==Some(Type::String)){name="__routerUsePath";}
   if matches!(name,"__requestBody"|"__requestText"|"__form"|"__multipart") && let ExprKind::Field(context_base,field)=&base.kind && field=="req" && matches!(self.expression_type_hint(context,context_base),Some(Type::Record(id)) if self.program.records[id].native_type==Some(NativeTypeId::HttpContext)) {name=match name{"__requestBody"=>"__contextRequestBody","__form"=>"__contextRequestForm","__multipart"=>"__contextRequestMultipart",_=>"__contextRequestText"};receiver=(**context_base).clone();}
   if matches!(name,"__route"|"__routerRoute")&&member!="route" {args.insert(0,Expr::new(ExprKind::String(member.to_uppercase()),callee.span));}
   if name=="__contextJson" && !args.is_empty(){let value=args.remove(0);let span=expression.span;let encode=Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@json:encode".into()),span)),vec![value,Expr::new(ExprKind::Integer("1048576".into()),span)]),span);args.insert(0,crate::stdlib::futures::unwrap(encode));}
   args.insert(0,receiver);let args=std::mem::take(args);let span=expression.span;
   if crate::stdlib::http::asynchronous(name){
    crate::stdlib::futures::lazy_module(expression,"std:http",name,args);
    if request_json{crate::stdlib::http::parse_request_json(expression);}
    // Preserve the options context when lazy lowering evaluates arguments into locals.
    if matches!(name,"__listen"|"__routerListen") && let ExprKind::Match{body,..}=&mut expression.kind && let Some(crate::syntax::Stmt{kind:crate::syntax::StmtKind::Let{annotation,..},..})=body.get_mut(2){
     let options=self.schema_native_dependency(NativeTypeId::HttpTlsOptions,span)?;
     *annotation=Some(self.checked_value_annotation(Type::Record(options),span)?);
    }
   }else{
    *expression=crate::stdlib::http::call(name,args,span);
    if matches!(name,"__contextRequestBody"|"__routerStatic"|"__routerConfigure"|"__routerRoute"|"__routerUsePath"|"__contextText"|"__contextHtml"|"__contextJson"|"__contextHeader"|"__contextStatus"|"__configure"|"__route"|"__requestHeader"|"__requestSetHeader"|"__responseHeader"|"__responseSetHeader"){*expression=crate::stdlib::futures::unwrap(expression.clone());}
   }
   return self.expression(context,expression,if name=="__contextGet"{Mode::Read}else{Mode::Consume},loans,expected).map(Some);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|matches!(binding.module,"std:http"|"std:signals"))else{return Ok(None);};
  let span=expression.span;
  if !binding.accepts_arity(args.len()) || matches!(binding.name,"__routerRoute"|"__routerUse"|"__routerUsePath") && args.len()<binding.parameters.len(){return Err(Diagnostic::new("arity",format!("{} requires {} arguments including its receiver.",binding.name,binding.parameters.len()),span));}
  if (crate::stdlib::http::asynchronous(binding.name)||binding.name=="__receive")&&!context.task_body{return Err(Diagnostic::new("async-context","HTTP I/O must run inside an async task.",span));}
  if matches!(binding.name,"__contextGet"|"__contextSet"){return self.http_variable(context,expression,loans,expected,id).map(Some);}
  let mut active=loans.clone();
  let mut router_types=None;
  for (index,arg) in args.iter_mut().enumerate() {
   let param=&binding.parameters[index.min(binding.parameters.len()-1)];
   if index==0 && param.ty==crate::native_bindings::NativeType::ObservedValue{
    let ty=self.expression(context,arg,param.access,&active,None)?;
    let Type::Record(record)=ty else{return Err(Diagnostic::new("http-context","Expected a stored Router or Context.",arg.span));};
    let expected=if binding.name.starts_with("__router"){NativeTypeId::HttpRouter}else{NativeTypeId::HttpContext};
    if self.program.records[record].native_type!=Some(expected){return Err(Diagnostic::new("http-context","Incorrect HTTP receiver type.",arg.span));}
    router_types=self.program.records[record].application.as_ref().map(|application|application.resolved_arguments.clone());
    for root in Self::loan_roots(arg,param.access){active.entry(root).or_default().push((param.access,arg.span));}
   }else if binding.name=="__contextNext" && index==1{
    let expected=self.thread_application(NativeTypeId::HttpContextNext,&router_types.as_ref().unwrap()[..1],span)?;
    let actual=self.expression(context,arg,Mode::Consume,&active,Some(expected))?;self.require(actual,Some(expected),arg.span)?;
   }else if param.ty==crate::native_bindings::NativeType::ObservedValue{
    let hono=binding.name.starts_with("__router");
    let count=if hono{match &arg.kind{ExprKind::Closure{params,..}=>params.len(),ExprKind::Name(name)=>self.function_names.get(name).map(|&id|self.signatures[id].params.len()).unwrap_or(1),_=>1}}else if binding.name=="__use"{2}else{1};
    if count>2{return Err(Diagnostic::new("http-handler","Router callbacks take Context and an optional next callback.",span));}
    let middleware=count==2 || matches!(binding.name,"__routerUse"|"__routerUsePath");
    let request=if hono{self.thread_application(NativeTypeId::HttpContext,router_types.as_ref().expect("router receiver"),span)?}else{Type::Record(self.schema_native_dependency(NativeTypeId::HttpRequest,span)?)};let next=if hono{self.thread_application(NativeTypeId::HttpContextNext,&router_types.as_ref().unwrap()[..1],span)?}else{Type::Record(self.schema_native_dependency(NativeTypeId::HttpNext,span)?)};
    let parameters=[request,next][..count].to_vec();
    if matches!(arg.kind,ExprKind::Name(_)){
     let callee=arg.clone();let mut params=Vec::new();let mut inputs=Vec::new();
     for (index,ty) in parameters.iter().enumerate(){let name=format!("http#argument{index}");let token=crate::syntax::Token{kind:crate::syntax::TokenKind::Symbol,text:name.clone(),span};
      inputs.push(Expr::new(ExprKind::Name(name),span));
      params.push(crate::syntax::CallableParameter{name:token,ty:crate::syntax::ParameterAnnotation::Named(Box::new(self.checked_value_annotation(*ty,span)?)),access:None,rest:false,initializer:crate::syntax::ParameterInitializer::Required});
     }
     arg.kind=ExprKind::Closure{id:usize::MAX/4+arg.span.start,params,result:crate::syntax::ResultAnnotation::Inferred(span),body:vec![crate::syntax::Stmt{kind:crate::syntax::StmtKind::Return(Some(Expr::new(ExprKind::Call(Box::new(callee),inputs),span))),span,labels:Vec::new(),control_target:None}]};
    }
    if let ExprKind::Closure{id,params,..}=&mut arg.kind{
     self.escaping.insert(*id);
     self.http_handler_origins.insert(*id);
     if hono && let Some(param)=params.first_mut(){param.access=Some(crate::syntax::Token{kind:crate::syntax::TokenKind::Name,text:"mutate".into(),span});}
     for (param,ty) in params.iter_mut().zip(&parameters){if matches!(param.ty,crate::syntax::ParameterAnnotation::Inferred(_)){param.ty=crate::syntax::ParameterAnnotation::Named(Box::new(self.checked_value_annotation(*ty,span)?));}}
    }
    let actual=self.expression(context,arg,Mode::Consume,&active,None)?;
    let Type::Closure(origin)=actual else{return Err(Diagnostic::new("http-handler","HTTP handlers require a concrete callback returning Task<Response>.",span));};
    let closure=&self.closures[origin];
    if closure.program_scoped||!closure.program_captures.is_empty()||closure.parameters.len()!=parameters.len()||closure.parameters.iter().zip(&parameters).any(|(id,ty)|closure.function.bindings[*id].ty!=*ty){return Err(Diagnostic::new("http-handler","HTTP callbacks take Request (and Next for middleware) and cannot borrow program state.",span));}
    thread_transfer::require_owned_captures(closure,&self.program.records,&|id|self.unions.members(id))
     .map_err(|_|Diagnostic::new("http-state","HTTP handlers must own retained state; caller loans and callback-valued state are unsupported.",span))?;
    if !closure.captures.is_empty() && !matches!(closure.invocation,Mode::Read|Mode::Copy){return Err(Diagnostic::new("http-state","Concurrent HTTP handlers cannot mutate or consume retained state.",span));}
    for capture in &closure.captures {self.http_shared_data(closure.function.bindings[capture.inner].ty,span)?;}
    let result=closure.function.result;
    if hono && matches!(result,Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::HttpResponse)){continue;}
    if !self.is_task_type(result){return Err(Diagnostic::new("http-handler","HTTP handlers return Task<Response>; use a named async function.",span));}
    self.http_task_transfer(result,span,hono)?;
    let(output,error)=self.future_shape(result,span)?;
    let outputs=if let Type::Union(id)=output{self.unions.members(id)}else{vec![output]};
    if !outputs.into_iter().all(|output|matches!(output,Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::HttpResponse)) || hono && middleware && matches!(output,Type::Void|Type::Undefined)){return Err(Diagnostic::new("http-handler","HTTP handlers return Response or Task<Response>; Router middleware may also return no response after next().",span));}
    self.future_start_transfer(error,span)?;
   }else{
    let ty=native_type(param.ty,&self.program.records);let actual=self.expression(context,arg,param.access,&active,Some(ty))?;self.require(actual,Some(ty),arg.span)?;
    for root in Self::loan_roots(arg,param.access){active.entry(root).or_default().push((param.access,arg.span));}
   }
  }
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn http_variable(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>,binding:usize)->Result<Type,Diagnostic>{
  let ExprKind::Call(_,args)=&mut expression.kind else{unreachable!()};let set=args.len()==3;let mode=if set{Mode::Mutate}else{Mode::Read};
  let Type::Record(record)=self.expression(context,&mut args[0],mode,loans,None)? else{return Err(Diagnostic::new("http-context","Expected a Context.",expression.span));};
  if self.program.records[record].native_type!=Some(NativeTypeId::HttpContext){return Err(Diagnostic::new("http-context","Expected a Context.",expression.span));}
  let variables=self.program.records[record].application.as_ref().unwrap().resolved_arguments[1];
  let ExprKind::String(key)=&args[1].kind else{return Err(Diagnostic::new("http-variable","Request variable keys must be declared string literals.",args[1].span));};
  let value=if let Type::Record(id)=variables{self.program.records[id].fields.iter().find(|field|field.name.text==*key).and_then(|field|field.ty)}else{None}.ok_or_else(||Diagnostic::new("http-variable","Unknown key in this Context's request variable record.",args[1].span))?;
  crate::stdlib::cache::validate_payload(value,&self.program.records,|id|self.unions.members(id),expression.span)?;
  self.http_shared_data(value,expression.span)?;
  self.expression(context,&mut args[1],Mode::Read,loans,Some(Type::String))?;
  let mut active=loans.clone();for root in Self::loan_roots(&args[0],mode){active.entry(root).or_default().push((mode,args[0].span));}
  let result=if set{let actual=self.expression(context,&mut args[2],Mode::Consume,&active,Some(value))?;self.require(actual,Some(value),args[2].span)?;Type::Void}else{
   let result=self.unions.intern([value,Type::Undefined],expression.span)?;
   if !result.is_copy(){
    if !Self::stable_at_receiver(&args[0]){return Err(Diagnostic::new("borrow","Request variable reads require a stored Context.",args[0].span));}
    for place in native_view_places::sources(context,&args[0])?{context.cache_read_owners.insert((place.root,place.fields));}
    context.native_view_calls=true;Type::BorrowedUnion(result.union().unwrap())
   }else{result}
  };
  self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(binding));Ok(result)
 }
 // The emitted request future owns an Arc of the callback before borrowing its state.
 // This permits read-only retained loans, never caller loans or exclusive access.
 fn http_task_transfer(&self,root:Type,span:Span,hono:bool)->Result<(),Diagnostic>{
  let mut pending=vec![root];let mut seen=HashSet::new();
  while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}
   if let Type::Closure(id)=ty && self.is_task_type(ty){
    let closure=&self.closures[id];
    if closure.program_scoped||!closure.program_captures.is_empty(){return Err(Diagnostic::new("http-state","HTTP request tasks cannot borrow program state.",span));}
    for capture in &closure.captures {
     let ty=closure.function.bindings[capture.inner].ty;
     match capture.storage {
      Mode::Read|Mode::Mutate if hono && matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::HttpContext))=>{},
      Mode::Read=>self.http_shared_data(ty,span)?,
      Mode::Consume=>pending.push(ty),
      Mode::Copy if ty.is_copy()=>{},
      _=>return Err(Diagnostic::new("http-state","HTTP request tasks cannot retain mutable state loans.",span)),
     }
    }
   }else{self.future_start_transfer(ty,span)?;}
  }Ok(())
 }
 fn http_shared_data(&self,root:Type,span:Span)->Result<(),Diagnostic>{
  let mut pending=vec![root];let mut seen=HashSet::new();
  while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}
   match ty {
    Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Void|Type::Enum(..)|Type::Literal(..)=>{},
    Type::Array(element)=>pending.push(element.ty()),
    Type::Union(id)=>pending.extend(self.unions.members(id)),
    Type::Map(id)=>{let map=self.program.records[id].map_definition.as_ref().unwrap();pending.extend([map.key,map.value]);},
    Type::Set(id)=>pending.push(self.program.records[id].set_definition.as_ref().unwrap().key),
    Type::Record(id)=>{
     let record=&self.program.records[id];
     if let Some(native)=record.native_type {
      let declaration=crate::native_types::declaration(native);
      let shared=matches!(native,NativeTypeId::SharedLruCache|NativeTypeId::SqlPool|NativeTypeId::HttpClient|NativeTypeId::AsyncSender|NativeTypeId::Bytes|NativeTypeId::JsonValue|NativeTypeId::Decimal|NativeTypeId::CalendarDate|NativeTypeId::TimeOfDay|NativeTypeId::UtcInstant|NativeTypeId::TimeDuration|NativeTypeId::TimeZone|NativeTypeId::Uuid)
       || (declaration.representation==crate::native_types::Representation::Record && declaration.thread_transfer);
      if !shared{return Err(Diagnostic::new("http-state","HTTP retained state requires immutable owned data or a supported concurrent pool/client/sender; local caches, borrowed views and request resources cannot be shared.",span));}
     }
     if let Some(application)=&record.application{pending.extend(&application.resolved_arguments);}
     pending.extend(record.fields.iter().filter_map(|field|field.ty));
    },
    _=>return Err(Diagnostic::new("http-state","HTTP retained state cannot contain borrowed views, callbacks or tasks.",span)),
   }
  }Ok(())
 }

}
