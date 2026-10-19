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
  let Some((id,binding))=crate::native_bindings::by_symbol(name).filter(|(_,binding)|binding.module=="std:ui-runtime" && matches!(binding.name,"signal"|"handle"|"get"|"set"|"ownState")) else{return Ok(None)};
  Self::arity(args.len(),if matches!(binding.name,"set"|"ownState"){2}else{1},expression.span)?;
  let result=if binding.name=="signal" {
   let payload=self.expression(context,&mut args[0],Mode::Read,loans,None)?;
   self.ui_scalar(payload,args[0].span)?;
   self.thread_application(NativeTypeId::UiSignal,&[payload],expression.span)?
  } else {
   let at=usize::from(binding.name=="ownState");
   if at==1 {let node=self.ui_record(NativeTypeId::UiNode);let ty=self.expression(context,&mut args[0],Mode::Consume,loans,Some(node))?;self.require(ty,Some(node),args[0].span)?;}
   let ty=self.expression(context,&mut args[at],if at==1{Mode::Consume}else{Mode::Read},loans,None)?;
   let Type::Record(record)=ty else{return Err(Diagnostic::new("type","Expected component state.",args[at].span));};
   if self.program.records[record].native_type!=Some(NativeTypeId::UiSignal){return Err(Diagnostic::new("type","Expected component state.",args[at].span));}
   let payload=self.program.records[record].application.as_ref().expect("applied UI state").resolved_arguments[0];self.ui_scalar(payload,args[at].span)?;
   match binding.name {
    "handle"=>ty,
    "get"=>{let error=self.ui_record(NativeTypeId::UiError);self.thread_result(payload,error,expression.span)?},
    "set"=>{let actual=self.expression(context,&mut args[1],Mode::Read,loans,Some(payload))?;self.require(actual,Some(payload),args[1].span)?;let error=self.ui_record(NativeTypeId::UiError);self.thread_result(Type::Void,error,expression.span)?},
    "ownState"=>{let node=args[0].ty.unwrap();let error=self.ui_record(NativeTypeId::UiError);self.thread_result(node,error,expression.span)?},
    _=>unreachable!(),
   }
  };
  self.require(result,expected,expression.span)?;expression.ty=Some(result);expression.target=Some(CallTarget::NativeBinding(id));Ok(Some(result))
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
 fn ui_record(&self,identity:NativeTypeId)->Type {Type::Record(self.program.records.iter().position(|record|record.native_type==Some(identity)).expect("registered UI record"))}
 fn ui_scalar(&self,ty:Type,span:Span)->Result<(),Diagnostic>{
  if matches!(ty,Type::Int|Type::Number|Type::Boolean|Type::String){Ok(())}else{Err(Diagnostic::new("ui-state-type","UI state supports int, number, boolean and string; resources, arrays, records, nullable values and callbacks are not scalar state.",span))}
 }
}
