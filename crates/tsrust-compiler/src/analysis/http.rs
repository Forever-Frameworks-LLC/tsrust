//! HTTP callbacks reuse task transfer checks and ordinary receiver loans.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
 pub(super) fn http_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};
  if let ExprKind::Field(base,member)=&callee.kind {
   let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
   let name=match(identity,member.as_str()){
    (Some(NativeTypeId::HttpApp),"get"|"post"|"put"|"patch"|"delete"|"head"|"options"|"route")=>"__route",
    (Some(NativeTypeId::HttpApp),"use")=>"__use",(Some(NativeTypeId::HttpApp),"fallback")=>"__fallback",(Some(NativeTypeId::HttpApp),"listen")=>"__listen",
    (Some(NativeTypeId::HttpServer),"wait")=>"__wait",(Some(NativeTypeId::HttpServer),"address")=>"__address",(Some(NativeTypeId::HttpServer),"shutdown")=>"__shutdown",
    (Some(NativeTypeId::HttpClient),"send")=>"__send",(Some(NativeTypeId::HttpNext),"run")=>"__next",
    (Some(NativeTypeId::HttpBody),"next")=>"__bodyNext",(Some(NativeTypeId::HttpBody),"text")=>"__bodyText",
    (Some(NativeTypeId::HttpRequest),"method")=>"__method",(Some(NativeTypeId::HttpRequest),"url")=>"__url",(Some(NativeTypeId::HttpRequest),"param")=>"__param",(Some(NativeTypeId::HttpResponse),"status")=>"__status",
    (Some(NativeTypeId::HttpRequest),"header")=>"__requestHeader",(Some(NativeTypeId::HttpRequest),"setHeader")=>"__requestSetHeader",(Some(NativeTypeId::HttpRequest),"body")=>"__requestBody",(Some(NativeTypeId::HttpRequest),"text")=>"__requestText",
    (Some(NativeTypeId::HttpResponse),"header")=>"__responseHeader",(Some(NativeTypeId::HttpResponse),"setHeader")=>"__responseSetHeader",(Some(NativeTypeId::HttpResponse),"body")=>"__responseBody",(Some(NativeTypeId::HttpResponse),"text")=>"__responseText",_=>return Ok(None),
   };
   if name=="__route"&&member!="route" {args.insert(0,Expr::new(ExprKind::String(member.to_uppercase()),callee.span));}
   args.insert(0,(**base).clone());let args=std::mem::take(args);let span=expression.span;
   if matches!(name,"__listen"|"__wait"|"__shutdown"|"__send"|"__next"|"__bodyNext"|"__bodyText"|"__requestText"|"__responseText"){
    crate::stdlib::futures::lazy_module(expression,"std:http",name,args);
   }else{
    *expression=crate::stdlib::http::call(name,args,span);
    if matches!(name,"__route"|"__requestHeader"|"__requestSetHeader"|"__responseHeader"|"__responseSetHeader"){*expression=crate::stdlib::futures::unwrap(expression.clone());}
   }
   return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:http")else{return Ok(None);};
  let span=expression.span;
  if !binding.accepts_arity(args.len()){return Err(Diagnostic::new("arity",format!("{} requires {} arguments including its receiver.",binding.name,binding.parameters.len()),span));}
  if matches!(binding.name,"__listen"|"__wait"|"__shutdown"|"__send"|"__next"|"__bodyNext"|"__bodyText"|"__requestText"|"__responseText")&&!context.task_body{return Err(Diagnostic::new("async-context","HTTP I/O must run inside an async task.",span));}
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
     for (param,ty) in params.iter_mut().zip(&parameters){if matches!(param.ty,crate::syntax::ParameterAnnotation::Inferred(_)){param.ty=crate::syntax::ParameterAnnotation::Named(Box::new(self.checked_value_annotation(*ty,span)?));}}
    }
    let actual=self.expression(context,arg,Mode::Consume,&active,None)?;
    let Type::Closure(origin)=actual else{return Err(Diagnostic::new("http-handler","HTTP handlers require a concrete callback returning Task<Response>.",span));};
    let closure=&self.closures[origin];
    // shortcut: captured handler state needs an explicit shared-state contract before it can be retained concurrently.
    if !closure.captures.is_empty()||closure.program_scoped||!closure.program_captures.is_empty()||closure.parameters.len()!=parameters.len()||closure.parameters.iter().zip(&parameters).any(|(id,ty)|closure.function.bindings[*id].ty!=*ty){return Err(Diagnostic::new("http-handler","HTTP callbacks take Request (and Next for middleware), without captured or borrowed program state. Call an async function with owned arguments.",span));}
    let result=closure.function.result;
    if !self.is_task_type(result){return Err(Diagnostic::new("http-handler","HTTP handlers return Task<Response>; use a named async function.",span));}
    self.future_start_transfer(result,span)?;
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
}
