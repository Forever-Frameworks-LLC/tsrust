pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
pub(crate) const LISTEN:NativeBinding=NativeBinding{module:"std:signals",name:"listen",parameters:&[],result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::SignalListener),&Signature::Named(NativeTypeId::SignalError))),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"signal_intrinsic"},features:&["signals"]};
pub(crate) const RECEIVE:NativeBinding=NativeBinding{module:"std:signals",name:"receive",parameters:&[NativeParameter{name:"listener",ty:NativeType::Structured(&Signature::Named(NativeTypeId::SignalListener)),access:Mode::Mutate,optional:false,rest:false}],result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::SignalError))),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"signal_intrinsic"},features:&["signals"]};
pub(crate) const POLL:NativeBinding=NativeBinding{name:"__receive",..RECEIVE};
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 use crate::syntax::ExprKind;
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
  if let ExprKind::Call(callee,args)=&mut expression.kind && matches!(&callee.kind,ExprKind::Name(name) if name=="@native:std:signals#receive"){
   let args=std::mem::take(args);if args.len()!=1{return Err(crate::Diagnostic::new("arity","receive expects one signal listener.",expression.span));}
   crate::stdlib::futures::lazy_module(expression,"std:signals","__receive",args);
  }Ok(())
 })?;}Ok(())
}
