//! HTTP request factories use ordinary checked forwarding for aliases and stored calls.
use crate::{Diagnostic,Span,native_bindings::NativeBinding,syntax::*};
fn name(operation:&str)->String{format!("@http-adapter:{operation}")}
fn factory(operation:&str)->bool{matches!(operation,"request"|"requestBytes"|"requestStream"|"requestChannel")}
pub(super) fn prepare(program:&mut Program)->Result<(),Diagnostic>{
 let mut used=std::collections::BTreeSet::new();
 let mut rewrite=|value:&mut Expr|{if let ExprKind::Name(symbol)=&value.kind && let Some(operation)=symbol.strip_prefix("@native:std:http#") && factory(operation){used.insert(operation.to_owned());value.kind=ExprKind::Name(name(operation));}Ok(())};
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut rewrite)?;}
 crate::source_walk::body(&mut program.module_bindings,&mut |_|Ok(()),&mut rewrite)?;
 for operation in used{let binding=crate::native_bindings::lookup("std:http",&operation).expect("registered HTTP factory");let wrapper=adapter(program,binding)?;program.functions.push(wrapper);}
 Ok(())
}
fn adapter(program:&mut Program,binding:&NativeBinding)->Result<Function,Diagnostic>{
 let parameters=binding.parameters.iter().enumerate().map(|(index,parameter)|format!("value{index}:{}",crate::type_syntax::access_type_text(parameter.access,&format!("P{index}")))).collect::<Vec<_>>().join(",");
 let arguments=(0..binding.parameters.len()).map(|index|format!("value{index}")).collect::<Vec<_>>().join(",");
 let mut function=crate::syntax::parse(&format!("function adapter({parameters}){{return backend({arguments});}}"))?.functions.remove(0);
 let offset=program.functions.iter().map(|function|function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0)+1;
 let shift=|span:&mut Span|{span.start+=offset;span.end+=offset;};function.name.text=name(binding.name);function.name.kind=TokenKind::Symbol;shift(&mut function.name.span);shift(&mut function.scope_span);
 let mut replacements=std::collections::HashMap::new();for(index,parameter)in binding.parameters.iter().enumerate(){replacements.insert(format!("P{index}"),crate::native_type_resolution::project(&mut program.records,&parameter.ty.signature(),function.name.span)?);}
 crate::source_walk::substitute_function_annotations(&mut function,&replacements)?;for(parameter,_,_)in &mut function.params{shift(&mut parameter.span);}
 crate::source_walk::substitute_function_body(&mut function,&replacements,&mut |expression|{shift(&mut expression.span);Ok(())})?;
 crate::source_walk::function_body(&mut function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&mut expression.kind && matches!(&callee.kind,ExprKind::Name(value)if value=="backend"){
  let args=std::mem::take(args);*expression=crate::stdlib::futures::unwrap(super::call(&format!("__{}",binding.name),args,expression.span));
 }Ok(())})?;Ok(function)
}
