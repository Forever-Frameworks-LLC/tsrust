//! Native values become ordinary checked wrappers before generic specialization.
//! Generic parameters preserve the native operation's accepted storage alternatives.
use crate::{Diagnostic,Span,native_bindings::NativeBinding,syntax::*};
pub(super) fn name(operation:&str)->String{format!("@crypto-adapter:{operation}")}
pub(super) fn adapter(program:&mut Program,binding:&NativeBinding)->Result<Function,Diagnostic>{
 let generic=binding.parameters.iter().enumerate().filter_map(|(index,_)|(!binding.argument_alternatives(index).is_empty()).then_some(index)).collect::<Vec<_>>();
 let types=generic.iter().map(|index|format!("G{index}")).collect::<Vec<_>>().join(",");let types=if types.is_empty(){types}else{format!("<{types}>")};
 let parameters=binding.parameters.iter().enumerate().map(|(index,parameter)|{
  let ty=if generic.contains(&index){format!("G{index}")}else{format!("P{index}")};let ty=crate::type_syntax::access_type_text(parameter.access,&ty);format!("value{index}{}:{ty}",if parameter.optional{"?"}else{""})
 }).collect::<Vec<_>>().join(",");let arguments=(0..binding.parameters.len()).map(|index|format!("value{index}")).collect::<Vec<_>>().join(",");
 let mut function=crate::syntax::parse(&format!("function adapter{types}({parameters}){{return backend({arguments});}}"))?.functions.remove(0);
 let offset=program.functions.iter().map(|function|function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0)+1;
 let shift=|span:&mut Span|{span.start+=offset;span.end+=offset;};function.name.text=name(binding.name);function.name.kind=TokenKind::Symbol;shift(&mut function.name.span);shift(&mut function.scope_span);
 let mut replacements=std::collections::HashMap::new();for(index,parameter)in binding.parameters.iter().enumerate(){if !generic.contains(&index){let annotation=crate::native_type_resolution::project(&mut program.records,&parameter.ty.signature(),function.name.span)?;replacements.insert(format!("P{index}"),annotation);}}
 crate::source_walk::substitute_function_annotations(&mut function,&replacements)?;
 for(parameter,_,_)in &mut function.params{shift(&mut parameter.span);}
 for parameter in &mut function.type_parameters{shift(&mut parameter.span);}
 crate::source_walk::substitute_function_body(&mut function,&replacements,&mut |expression|{shift(&mut expression.span);Ok(())})?;
 crate::source_walk::function_body(&mut function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&mut expression.kind && matches!(&callee.kind,ExprKind::Name(name)if name=="backend"){
  let inputs=std::mem::take(args);super::lower(expression,binding.name,inputs);
 }Ok(())})?;
 Ok(function)
}
