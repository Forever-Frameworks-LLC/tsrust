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
 fn ui_record(&self,identity:NativeTypeId)->Type {Type::Record(self.program.records.iter().position(|record|record.native_type==Some(identity)).expect("registered UI record"))}
 fn ui_scalar(&self,ty:Type,span:Span)->Result<(),Diagnostic>{
  if matches!(ty,Type::Int|Type::Number|Type::Boolean|Type::String){Ok(())}else{Err(Diagnostic::new("ui-state-type","UI state supports int, number, boolean and string; resources, arrays, records, nullable values and callbacks are not scalar state.",span))}
 }
}
