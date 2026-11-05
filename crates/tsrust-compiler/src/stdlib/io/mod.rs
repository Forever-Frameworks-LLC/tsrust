//! Owned asynchronous I/O adapters with explicit Result payloads.
pub(crate) mod bindings;
mod wrappers;
pub(crate) fn runtime()->String{format!("{}\n}}\n",include_str!("runtime.rs").split_once("\n #[cfg(test)]mod tests").expect("I/O tests follow runtime").0)}
pub(crate) const HTTP_RUNTIME:&str=include_str!("http_runtime.rs");
pub(crate) fn asynchronous(name:&str)->bool{matches!(name,"read"|"readInto"|"readExact"|"readAll"|"write"|"writeAll"|"flush"|"copy"|"readLine"|"close")}
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 use crate::syntax::ExprKind;
 let mut used=std::collections::BTreeSet::new();let mut values=std::collections::BTreeSet::new();let mut direct=std::collections::HashSet::new();
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut |expression|{
  if let ExprKind::Call(callee,args)=&mut expression.kind && let ExprKind::Name(symbol)=&callee.kind && let Some(name)=symbol.strip_prefix("@native:std:io#"){
   used.insert(name.trim_start_matches('_').to_owned());
   if asynchronous(name){let name=format!("__{name}");let args=std::mem::take(args);crate::stdlib::futures::lazy_result(expression,"std:io",&name,args);}
   else{direct.insert(callee.span);}
  }else if let ExprKind::Name(symbol)=&expression.kind && let Some(name)=symbol.strip_prefix("@native:std:io#") && !name.starts_with('_') && !direct.contains(&expression.span){
   used.insert(name.to_owned());values.insert(name.to_owned());expression.kind=ExprKind::Name(wrappers::name(name));
  }Ok(())})?;}
 for name in values{let binding=crate::native_bindings::lookup("std:io",&name).expect("resolved I/O export");let wrapper=wrappers::adapter(program,binding)?;program.functions.push(wrapper);}
 if !used.is_empty(){for binding in bindings::OPERATIONS{if used.contains(binding.name.trim_start_matches('_')){crate::native_binding_types::prepare(&mut program.records,binding,crate::Span::default())?;}}}
 Ok(())
}
