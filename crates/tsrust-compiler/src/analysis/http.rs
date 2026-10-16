//! HTTP callbacks reuse task transfer checks and ordinary receiver loans.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
 pub(super) fn http_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
  if let ExprKind::Field(base,member)=&callee.kind {
   let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
   let Some(name)=crate::stdlib::http::method(identity,member)else{return Ok(None);};
   if name=="__route"&&member!="route" {args.insert(0,Expr::new(ExprKind::String(member.to_uppercase()),callee.span));}
   args.insert(0,(**base).clone());let args=std::mem::take(args);let span=expression.span;
   if crate::stdlib::http::asynchronous(name){
    crate::stdlib::futures::lazy_module(expression,"std:http",name,args);
    // Preserve the options context when lazy lowering evaluates arguments into locals.
    if name=="__listen" && let ExprKind::Match{body,..}=&mut expression.kind && let Some(crate::syntax::Stmt{kind:crate::syntax::StmtKind::Let{annotation,..},..})=body.get_mut(2){
     let options=self.schema_native_dependency(NativeTypeId::HttpTlsOptions,span)?;
     *annotation=Some(self.checked_value_annotation(Type::Record(options),span)?);
    }
   }else{
    *expression=crate::stdlib::http::call(name,args,span);
    if matches!(name,"__configure"|"__route"|"__requestHeader"|"__requestSetHeader"|"__responseHeader"|"__responseSetHeader"){*expression=crate::stdlib::futures::unwrap(expression.clone());}
   }
   return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|matches!(binding.module,"std:http"|"std:signals"))else{return Ok(None);};
  let span=expression.span;
  if !binding.accepts_arity(args.len()){return Err(Diagnostic::new("arity",format!("{} requires {} arguments including its receiver.",binding.name,binding.parameters.len()),span));}
  if (crate::stdlib::http::asynchronous(binding.name)||binding.name=="__receive")&&!context.task_body{return Err(Diagnostic::new("async-context","HTTP I/O must run inside an async task.",span));}
  let mut active=loans.clone();
  for (arg,param) in args.iter_mut().zip(binding.parameters) {
   if param.ty==crate::native_bindings::NativeType::ObservedValue{
    let request=Type::Record(self.schema_native_dependency(NativeTypeId::HttpRequest,span)?);let next=Type::Record(self.schema_native_dependency(NativeTypeId::HttpNext,span)?);
    let parameters=if binding.name=="__use"{vec![request,next]}else{vec![request]};
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
    if !self.is_task_type(result){return Err(Diagnostic::new("http-handler","HTTP handlers return Task<Response>; use a named async function.",span));}
    self.http_task_transfer(result,span)?;
    let(output,error)=self.future_shape(result,span)?;
    if !matches!(output,Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::HttpResponse)){return Err(Diagnostic::new("http-handler","HTTP handlers must return Task<Response>.",span));}
    self.future_start_transfer(error,span)?;
   }else{
    let ty=native_type(param.ty,&self.program.records);self.expression(context,arg,param.access,&active,Some(ty))?;
    for root in Self::loan_roots(arg,param.access){active.entry(root).or_default().push((param.access,arg.span));}
   }
  }
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 // The emitted request future owns an Arc of the callback before borrowing its state.
 // This permits read-only retained loans, never caller loans or exclusive access.
 fn http_task_transfer(&self,root:Type,span:Span)->Result<(),Diagnostic>{
  let mut pending=vec![root];let mut seen=HashSet::new();
  while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}
   if let Type::Closure(id)=ty && self.is_task_type(ty){
    let closure=&self.closures[id];
    if closure.program_scoped||!closure.program_captures.is_empty(){return Err(Diagnostic::new("http-state","HTTP request tasks cannot borrow program state.",span));}
    for capture in &closure.captures {
     let ty=closure.function.bindings[capture.inner].ty;
     match capture.storage {
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
