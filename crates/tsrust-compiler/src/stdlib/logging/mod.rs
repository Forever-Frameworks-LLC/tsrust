//! Explicit local structured logging; public wrappers preserve checked callback errors.
pub(crate) const SOURCE:&str=include_str!("source.tsr");
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
pub(crate) const PREFIX:&str="@stdlib:log#";
pub(crate) const METHODS:&[&str]=&["debug","info","warn","error","child","flush","close"];
use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{NativeTypeId,Signature}};
const LOGGER:Signature=Signature::Named(NativeTypeId::Logger);
const ERROR:Signature=Signature::Named(NativeTypeId::LogError);
const fn parameter(name:&'static str,ty:NativeType,access:Mode)->NativeParameter{NativeParameter{name,ty,access,optional:false,rest:false}}
const fn binding(name:&'static str,parameters:&'static[NativeParameter],result:NativeType)->NativeBinding{NativeBinding{module:"std:log-runtime",name,parameters,result,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"logging_intrinsic"},features:&["logging"]}}
const READ:NativeParameter=parameter("logger",NativeType::Structured(&LOGGER),Mode::Read);
pub(crate) const CREATE:NativeBinding=binding("create",&[parameter("options",NativeType::Structured(&Signature::Named(NativeTypeId::LoggerOptions)),Mode::Read)],NativeType::Structured(&Signature::Result(&LOGGER,&ERROR)));
pub(crate) const ENABLED:NativeBinding=binding("enabled",&[READ,parameter("level",NativeType::String,Mode::Read)],NativeType::Boolean);
pub(crate) const EMIT:NativeBinding=binding("emit",&[READ,parameter("level",NativeType::String,Mode::Read),parameter("fields",NativeType::ObservedValue,Mode::Consume)],NativeType::Void);
pub(crate) const CHILD:NativeBinding=binding("child",&[READ,parameter("fields",NativeType::ObservedValue,Mode::Consume)],NativeType::Structured(&LOGGER));
pub(crate) const FLUSH:NativeBinding=binding("flush",&[READ],NativeType::Structured(&Signature::Result(&Signature::Void,&ERROR)));
pub(crate) const CLOSE:NativeBinding=binding("close",&[parameter("logger",NativeType::Structured(&LOGGER),Mode::Consume)],FLUSH.result);

#[cfg(test)]#[allow(dead_code)]mod tests{include!("runtime.rs");include!("runtime_tests.rs");}

/// This trusted source module has no initializers. Delay its own native
/// preparation until linked external references decide whether it is needed.
pub(crate) fn omit_unused(program:&mut crate::syntax::Program,names:&std::collections::BTreeMap<String,String>)->Result<bool,crate::Diagnostic>{
 use crate::syntax::ExprKind;
 let facades=names.values().cloned().collect::<std::collections::BTreeSet<_>>();
 // External type annotations have already been prepared by their own module.
 let mut referenced=program.records.iter().any(|record|record.native_type.is_some_and(|id|crate::native_types::declaration(id).module=="std:log-runtime"));
 let mut visit=|expression:&mut crate::syntax::Expr|{if let ExprKind::Name(name)=&expression.kind{referenced|=facades.contains(name);}Ok(())};
 for function in &mut program.functions{if !facades.contains(&function.name.text){crate::source_walk::function_body(function,&mut |_|Ok(()),&mut visit)?;}}
 crate::source_walk::body(&mut program.module_bindings,&mut |_|Ok(()),&mut visit)?;
 if !referenced{program.functions.retain(|function|!facades.contains(&function.name.text));}
 Ok(!referenced)
}
