//! Owned subprocesses reuse checked task/Result and I/O transport.
pub(crate) mod bindings;
mod wrappers;
pub(crate) const RUNTIME:&str=include_str!("runtime.rs");
pub(crate) fn asynchronous(name:&str)->bool{matches!(name,"run"|"wait"|"kill")}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 use crate::syntax::ExprKind;
 let child=program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::ProcessChild));
 let mut used=std::collections::BTreeSet::new();let mut values=std::collections::BTreeSet::new();let mut direct=std::collections::HashSet::new();
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
  if let ExprKind::Call(callee,args)=&mut expression.kind && let ExprKind::Name(symbol)=&callee.kind && let Some(name)=symbol.strip_prefix("@native:std:process#"){
   used.insert(name.trim_start_matches('_').to_owned());
   if asynchronous(name){let name=format!("__{name}");let args=std::mem::take(args);crate::stdlib::futures::lazy_result(expression,"std:process",&name,args);}
   else{direct.insert(callee.span);}
  }else if let ExprKind::Name(symbol)=&expression.kind && let Some(name)=symbol.strip_prefix("@native:std:process#") && !name.starts_with('_') && !direct.contains(&expression.span){
   used.insert(name.to_owned());values.insert(name.to_owned());expression.kind=ExprKind::Name(wrappers::name(name));
  }Ok(())})?;}
 for name in values{let binding=crate::native_bindings::lookup("std:process",&name).expect("resolved process export");let wrapper=wrappers::adapter(program,binding)?;program.functions.push(wrapper);}
 if child||!used.is_empty(){for binding in bindings::OPERATIONS{crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}
 Ok(())
}
pub(crate) fn method(identity:Option<crate::native_signatures::NativeTypeId>,name:&str)->Option<&'static crate::native_bindings::NativeBinding>{if identity!=Some(crate::native_signatures::NativeTypeId::ProcessChild){return None;}Some(match name{"id"=>&bindings::ID,"takeStdin"=>&bindings::TAKE_STDIN,"takeStdout"=>&bindings::TAKE_STDOUT,"takeStderr"=>&bindings::TAKE_STDERR,"wait"=>&bindings::WAIT,"tryWait"=>&bindings::TRY_WAIT,"kill"=>&bindings::KILL,_=>return None})}
