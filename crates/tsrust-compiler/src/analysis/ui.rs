//! State operations preserve one strong owner and non-retaining bindings.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Analyzer {
 pub(super) fn ui_empty_props(&self,context:&Context,expression:&mut Expr) {
  let ExprKind::Call(callee,args)=&mut expression.kind else{return;};
  let ExprKind::Name(name)=&callee.kind else{return;};
  if context.names.contains_key(name)||context.pending_names.contains(name){return;}
  let Some(&function)=self.function_names.get(name)else{return;};
  // JSX's empty props record shares the complete element span; ordinary {} arguments do not.
  if self.program.functions[function].params.is_empty() && args.len()==1 && args[0].span==expression.span && matches!(&args[0].kind,ExprKind::Record(fields) if fields.is_empty()){args.clear();}
 }

 pub(super) fn ui_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
  if let Some(result)=self.ui_reactive_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_string_binding_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_computed_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_visibility_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_route_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_branch_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_text_binding_call(context,expression,loans,expected)? {return Ok(Some(result));}
  if let Some(result)=self.ui_input_call(context,expression,loans,expected)? {return Ok(Some(result));}
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};
  if let ExprKind::Field(base,member)=&callee.kind {
   let identity=self.expression_type_hint(context,base).and_then(|ty|if let Type::Record(id)=ty{self.program.records[id].native_type}else{None});
   let Some(binding)=identity.and_then(|id|crate::native_bindings::ui_method(id,member)) else{return Ok(None)};
   args.insert(0,(**base).clone());callee.kind=ExprKind::Name(format!("@native:std:ui-runtime#{}",binding.name));
   *expression=crate::stdlib::futures::unwrap(expression.clone());
   return self.expression(context,expression,Mode::Consume,loans,expected).map(Some);
  }
  let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime" && matches!(binding.name,"signal"|"handle"|"get"|"computedGet"|"set"|"ownState")) else{return Ok(None)};
  Self::arity(args.len(),if matches!(binding.name,"set"|"ownState"){2}else{1},expression.span)?;
  let result=if binding.name=="signal" {
   let payload=self.expression(context,&mut args[0],Mode::Read,loans,None)?;
   self.ui_scalar(payload,args[0].span)?;
   self.thread_application(NativeTypeId::UiSignal,&[payload],expression.span)?
  } else {
   let at=usize::from(binding.name=="ownState");
   if at==1 {let ty=self.expression(context,&mut args[0],Mode::Consume,loans,None)?;self.ui_component_result(ty,args[0].span)?;}
   let ty=self.expression(context,&mut args[at],if at==1{Mode::Consume}else{Mode::Read},loans,None)?;
   let Type::Record(record)=ty else{return Err(Diagnostic::new("type","Expected component state.",args[at].span));};
   if !matches!(self.program.records[record].native_type,Some(NativeTypeId::UiSignal|NativeTypeId::UiComputed)){return Err(Diagnostic::new("type","Expected component state.",args[at].span));}
   if self.program.records[record].native_type==Some(NativeTypeId::UiComputed)&&binding.name=="set"{return Err(Diagnostic::new("ui-computed-readonly","Computed values are read-only.",expression.span));}
   let payload=self.program.records[record].application.as_ref().expect("applied UI state").resolved_arguments[0];
   if self.program.records[record].native_type==Some(NativeTypeId::UiSignal){self.ui_scalar(payload,args[at].span)?;}else{self.ui_computed_scalar(payload,args[at].span)?;}
   match binding.name {
    "handle"=>ty,
    "get"|"computedGet"=>{let error=self.ui_record(NativeTypeId::UiError);self.thread_result(payload,error,expression.span)?},
    "set"=>{let actual=self.expression(context,&mut args[1],Mode::Read,loans,Some(payload))?;self.require(actual,Some(payload),args[1].span)?;let error=self.ui_record(NativeTypeId::UiError);self.thread_result(Type::Void,error,expression.span)?},
    "ownState"=>{let node=args[0].ty.unwrap();let error=self.ui_record(NativeTypeId::UiError);self.thread_result(node,error,expression.span)?},
    _=>unreachable!(),
   }
  };
  self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_reactive_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  if name!="@native:std:ui-runtime#reactive"{return Ok(None);}Self::arity(args.len(),1,expression.span)?;
  let (record,_)=self.ui_reactive_prop_type(expected,expression.span)?;
  let getter=args.remove(0);let field=crate::syntax::Token{text:"evaluate".into(),span:expression.span,kind:crate::syntax::TokenKind::Name};
  expression.kind=ExprKind::Record(vec![(field,getter)]);
  let actual=self.expression(context,expression,Mode::Consume,loans,Some(Type::Record(record)))?;self.require(actual,expected,expression.span)?;Ok(Some(actual))
 }
 pub(super) fn ui_reactive_prop_type(&self,expected:Option<Type>,span:Span)->Result<(usize,Type),Diagnostic>{
  let mut pending=expected.into_iter().collect::<Vec<_>>();let mut found=None;
  while let Some(ty)=pending.pop(){match ty{Type::Union(id)=>pending.extend(self.unions.members(id)),Type::Record(record)=>{let fields=&self.program.records[record].fields;if fields.len()==1&&fields[0].name.text=="evaluate"{if let Some((payload,_))=fields[0].ty.and_then(|ty|self.ui_computed_callback(ty)){self.ui_computed_scalar(payload,span)?;if found.is_some(){return Err(Diagnostic::new("ui-reactive-prop","Use one ReactiveValue<T> contract for a live scalar prop.",span));}found=Some((record,fields[0].ty.unwrap()));}}},_=>{}}}
  found.ok_or_else(||Diagnostic::new("ui-reactive-prop","This JSX expression requires a declared ReactiveValue<T> scalar prop.",span))
 }
 fn ui_string_binding_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime"&&matches!(binding.name,"bindClass"|"bindVariant")) else{return Ok(None)};
  Self::arity(args.len(),2,expression.span)?;let annotation=crate::type_syntax::parse("Owned<Function<string>>")?;let callback=crate::callable_annotations::callback(&annotation).unwrap();let getter=self.resolve_callback(&callback)?;
  let node=self.ui_record(NativeTypeId::UiNode);let contract=|ty|ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:false};self.check_call_arguments(context,args,&[contract(node),contract(getter)],loans)?;
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_computed_scalar(&self,ty:Type,span:Span)->Result<(),Diagnostic>{
  let mut pending=vec![ty];while let Some(ty)=pending.pop(){match ty{Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Literal(..)|Type::Enum(..)=>{},Type::Union(id)=>pending.extend(self.unions.members(id)),_=>return Err(Diagnostic::new("ui-computed-type","Computed values support scalars, literal unions and nullable scalars; resources, arrays, records and callbacks are not scalar results.",span))}}Ok(())
 }
 fn ui_computed_callback(&self,ty:Type)->Option<(Type,usize)>{let id=match ty{Type::Callback(id)=>id,Type::StoredCallable(family)=>self.callables.storage.declaration(family)?,_=>return None};let contract=&self.callables.callbacks[id];(contract.kind.is_owned()&&!contract.kind.is_once()&&contract.params.is_empty()).then_some((contract.result,id))}
 fn ui_computed_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,_))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime"&&binding.name=="computed") else{return Ok(None)};
  Self::arity(args.len(),1,expression.span)?;
  let Some(result@Type::Record(record))=expected else{return Err(Diagnostic::new("ui-computed-type","A computed expression requires its declared Computed<T> result.",expression.span))};
  if self.program.records[record].native_type!=Some(NativeTypeId::UiComputed){return Err(Diagnostic::new("ui-computed-type","Expected Computed<T>.",expression.span));}
  let payload=self.program.records[record].application.as_ref().expect("computed application").resolved_arguments[0];self.ui_computed_scalar(payload,expression.span)?;
  let hint=self.expression_type_hint(context,&args[0]);
  let wanted=if hint.is_some_and(|ty|matches!(ty,Type::Callback(_)|Type::StoredCallable(_)|Type::Closure(_))){
   let result=self.checked_value_annotation(payload,expression.span)?;let annotation=crate::syntax::CallbackAnnotation{kind:crate::syntax::CallbackKind::Owned,params:vec![],result:Box::new(crate::syntax::ResultAnnotation::Named(Box::new(result))),span:expression.span};Some(self.resolve_callback(&annotation)?)
  }else{None};
  let actual=self.expression(context,&mut args[0],Mode::Consume,loans,wanted)?;let mut pending=vec![actual];
  while let Some(value)=pending.pop(){
   if let Type::Union(union)=value{pending.extend(self.unions.members(union));continue;}
   let output=if let Type::Closure(id)=value{self.closures[id].function.result}else if let Some((output,_))=self.ui_computed_callback(value){output}else if let Type::Record(record)=value{
    let fields=&self.program.records[record].fields;
    if fields.len()!=1||fields[0].name.text!="evaluate"{return Err(Diagnostic::new("ui-computed-type","A reactive prop requires a reusable zero-argument evaluate callback.",args[0].span));}
    self.ui_computed_callback(fields[0].ty.expect("reactive field")).ok_or_else(||Diagnostic::new("ui-computed-type","A reactive prop requires a reusable zero-argument evaluate callback.",args[0].span))?.0
   }else{self.ui_computed_scalar(value,args[0].span)?;value};
   self.require(output,Some(payload),args[0].span)?;
  }
  expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_visibility_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime"&&binding.name=="visibility") else{return Ok(None)};
  Self::arity(args.len(),2,expression.span)?;
  let annotation=crate::type_syntax::parse("Owned<Function<void, [visible: boolean]>>")?;
  let callback=crate::callable_annotations::callback(&annotation).expect("visibility callback");let handler=self.resolve_callback(&callback)?;
  let node=self.native_parameter_type(binding.parameters[0].ty)?;
  let contract=|ty|ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:false};
  self.check_call_arguments(context,args,&[contract(node),contract(handler)],loans)?;
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_route_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime"&&binding.name=="route") else{return Ok(None)};
  Self::arity(args.len(),3,expression.span)?;
  let node=self.ui_record(NativeTypeId::UiNode);let result=self.checked_value_annotation(node,expression.span)?;
  let annotation=crate::syntax::CallbackAnnotation{kind:crate::syntax::CallbackKind::Owned,params:vec![],result:Box::new(crate::syntax::ResultAnnotation::Named(Box::new(result))),span:expression.span};let render=self.resolve_callback(&annotation)?;
  let contract=|ty,mode|ArgumentContract{ty,mode,omission:crate::ParameterOmission::Required,rest:false};
  self.check_call_arguments(context,args,&[contract(Type::String,Mode::Read),contract(Type::Boolean,Mode::Read),contract(render,Mode::Consume)],loans)?;
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_branch_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime"&&binding.name=="branch") else{return Ok(None)};
  Self::arity(args.len(),3,expression.span)?;
  let result=self.native_parameter_type(crate::native_bindings::NativeType::Structured(&crate::stdlib::ui::CHILD_VALUE))?;
  let result=self.checked_value_annotation(result,expression.span)?;
  let callback=crate::syntax::CallbackAnnotation{kind:crate::syntax::CallbackKind::Owned,params:vec![],result:Box::new(crate::syntax::ResultAnnotation::Named(Box::new(result))),span:expression.span};
  let child=self.resolve_callback(&callback)?;let predicate=self.native_parameter_type(crate::native_bindings::NativeType::OwnedBooleanCallback)?;
  let contract=|ty|ArgumentContract{ty,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:false};
  self.check_call_arguments(context,args,&[contract(predicate),contract(child),contract(child)],loans)?;
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_text_binding_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic> {
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None)};let ExprKind::Name(name)=&callee.kind else{return Ok(None)};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime" && binding.name=="bindText") else{return Ok(None)};
  Self::arity(args.len(),1,expression.span)?;
  let annotation=crate::type_syntax::parse("Owned<Function<string | int | number | boolean | null | undefined>>")?;
  let callback=crate::callable_annotations::callback(&annotation).expect("text binding callback");let wanted=self.resolve_callback(&callback)?;
  self.check_call_arguments(context,args,&[ArgumentContract{ty:wanted,mode:Mode::Consume,omission:crate::ParameterOmission::Required,rest:false}],loans)?;
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 #[inline(never)]
 fn ui_input_call(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Option<Type>,Diagnostic>{
  let ExprKind::Call(callee,args)=&mut expression.kind else{return Ok(None);};let ExprKind::Name(name)=&callee.kind else{return Ok(None);};
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime" && matches!(binding.name,"input"|"inputChange")) else{return Ok(None);};
  Self::arity(args.len(),binding.parameters.len(),expression.span)?;
  for (index,arg) in args.iter_mut().enumerate(){
   if index==0 {let actual=self.expression(context,arg,Mode::Consume,loans,None)?;self.ui_input_value(actual,arg.span)?;continue;}
   if index==4 {let actual=self.expression(context,arg,Mode::Consume,loans,None)?;self.ui_input_handler(actual,arg.span)?;continue;}
   let wanted=self.native_parameter_type(binding.parameters[index].ty)?;
   let actual=self.expression(context,arg,binding.parameters[index].access,loans,Some(wanted))?;self.require(actual,Some(wanted),arg.span)?;
  }
  let result=native_type(binding.result,&self.program.records);self.require(result,expected,expression.span)?;
  expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
 }
 fn ui_input_handler(&self,ty:Type,span:Span)->Result<(),Diagnostic> {
  let declaration=match ty{Type::Callback(id)=>Some(id),Type::StoredCallable(family)=>self.callables.storage.declaration(family),_=>None};
  let valid=declaration.is_some_and(|id|{
   let contract=&self.callables.callbacks[id];
   if !contract.kind.is_owned() || contract.kind.is_once() || contract.params.len()!=1 || contract.params[0].mode!=Mode::Read || contract.result!=Type::Void{return false;}
   let Type::Record(event)=contract.params[0].ty else{return false;};let fields=&self.program.records[event].fields;
   if self.program.records[event].native_type.is_some(){return false;}
   if fields.len()!=2 || fields[0].name.text!="target" || fields[1].name.text!="currentTarget" || fields[0].ty!=fields[1].ty{return false;}
   let Some(Type::Record(target))=fields[0].ty else{return false;};let fields=&self.program.records[target].fields;
   if self.program.records[target].native_type.is_some(){return false;}
   fields.len()==3 && fields.iter().zip(["value","name","id"]).all(|(field,name)|field.name.text==name && field.ty==Some(Type::String))
  });
  if valid{Ok(())}else{Err(Diagnostic::new("ui-input-handler","Input change handlers require an owned reusable callback taking a read-only event snapshot and returning void.",span))}
 }
 fn ui_input_value(&self,ty:Type,span:Span)->Result<(),Diagnostic>{
  let mut pending=vec![ty];
  while let Some(ty)=pending.pop(){match ty {
   Type::String|Type::Int|Type::Number|Type::Undefined=>{},
   Type::Union(id)=>pending.extend(self.unions.members(id)),
   Type::Record(id)=>{
    let record=&self.program.records[id];
    let signal=record.native_type==Some(NativeTypeId::UiSignal) && record.application.as_ref().is_some_and(|app|app.resolved_arguments.len()==1 && matches!(app.resolved_arguments[0],Type::String|Type::Int|Type::Number));
    let derived=record.native_type.is_none() && record.fields.len()==1 && record.fields[0].name.text=="evaluate" && record.fields[0].ty.and_then(|ty|match ty{Type::Callback(id)=>Some(id),Type::StoredCallable(family)=>self.callables.storage.declaration(family),_=>None}).is_some_and(|id|{let contract=&self.callables.callbacks[id];contract.kind.is_owned() && !contract.kind.is_once() && contract.params.is_empty() && match contract.result{Type::String|Type::Int|Type::Number=>true,Type::Union(id)=>self.unions.members(id).into_iter().all(|ty|matches!(ty,Type::String|Type::Int|Type::Number)),_=>false}});
    if !signal && !derived{return Err(Diagnostic::new("ui-input-value","Input values must be strings, ints, numbers or checked bindings of those types.",span));}
   },
   _=>return Err(Diagnostic::new("ui-input-value","Input values must be strings, ints, numbers or checked bindings of those types.",span)),
  }}Ok(())
 }
 fn ui_component_result(&self,ty:Type,span:Span)->Result<(),Diagnostic> {
  let mut pending=vec![ty];while let Some(ty)=pending.pop(){match ty {
   Type::Null|Type::Undefined=>{},
   Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::UiNode)=>{},
   Type::Union(id)=>pending.extend(self.unions.members(id)),
   _=>return Err(Diagnostic::new("ui-state-return","A component with local state must return an owned Node, null or undefined.",span)),
  }}Ok(())
 }
 fn ui_record(&self,identity:NativeTypeId)->Type {Type::Record(self.program.records.iter().position(|record|record.native_type==Some(identity)).expect("registered UI record"))}
 fn ui_scalar(&self,ty:Type,span:Span)->Result<(),Diagnostic>{
  if matches!(ty,Type::Int|Type::Number|Type::Boolean|Type::String){Ok(())}else{Err(Diagnostic::new("ui-state-type","UI state supports int, number, boolean and string; resources, arrays, records, nullable values and callbacks are not scalar state.",span))}
 }
}
