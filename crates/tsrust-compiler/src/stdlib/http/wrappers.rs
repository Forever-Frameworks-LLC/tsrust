//! HTTP request factories use ordinary checked forwarding for aliases and stored calls.
use crate::{Diagnostic,Span,native_bindings::NativeBinding,syntax::*};
fn name(operation:&str)->String{format!("@http-adapter:{operation}")}
fn factory(operation:&str)->bool{matches!(operation,"cors"|"request"|"requestBytes"|"requestStream"|"requestChannel"|"requestForm"|"requestMultipart")}
pub(super) fn prepare(program:&mut Program)->Result<(),Diagnostic>{
 let mut used=std::collections::BTreeSet::new();
 let mut rewrite=|value:&mut Expr|{if let ExprKind::Name(symbol)=&value.kind && let Some(operation)=symbol.strip_prefix("@native:std:http#") && factory(operation){used.insert(operation.to_owned());value.kind=ExprKind::Name(name(operation));}Ok(())};
 for function in &mut program.functions{crate::source_walk::function_body(function,&mut |_|Ok(()),&mut rewrite)?;}
 crate::source_walk::body(&mut program.module_bindings,&mut |_|Ok(()),&mut rewrite)?;
 for operation in used{let binding=crate::native_bindings::lookup("std:http",&operation).expect("registered HTTP factory");let wrapper=adapter(program,binding)?;program.functions.push(wrapper);}
 Ok(())
}
fn adapter(program:&mut Program,binding:&NativeBinding)->Result<Function,Diagnostic>{
 let parameters=binding.parameters.iter().enumerate().map(|(index,parameter)|format!("value{index}{}:{}",if parameter.optional{"?"}else{""},crate::type_syntax::access_type_text(parameter.access,&format!("P{index}")))).collect::<Vec<_>>().join(",");
 let arguments=(0..binding.parameters.len()).map(|index|format!("value{index}")).collect::<Vec<_>>().join(",");
 let mut function=crate::syntax::parse(&format!("function adapter({parameters}){{return backend({arguments});}}"))?.functions.remove(0);
 let offset=program.functions.iter().map(|function|function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0)+1;
 let shift=|span:&mut Span|{span.start+=offset;span.end+=offset;};function.name.text=name(binding.name);function.name.kind=TokenKind::Symbol;shift(&mut function.name.span);shift(&mut function.scope_span);
 // Callable contexts need the unwrapped result before async body discovery.
 let crate::native_signatures::Signature::Result(success,_)=binding.result.signature() else{unreachable!("HTTP request factory result")};
 function.result=Some(ResultAnnotation::Named(Box::new(crate::native_type_resolution::project(&mut program.records,success,function.name.span)?)));
 let mut replacements=std::collections::HashMap::new();for(index,parameter)in binding.parameters.iter().enumerate(){replacements.insert(format!("P{index}"),crate::native_type_resolution::project(&mut program.records,&parameter.ty.signature(),function.name.span)?);}
 crate::source_walk::substitute_function_annotations(&mut function,&replacements)?;for(parameter,_,_)in &mut function.params{shift(&mut parameter.span);}
 crate::source_walk::substitute_function_body(&mut function,&replacements,&mut |expression|{shift(&mut expression.span);Ok(())})?;
 crate::source_walk::function_body(&mut function,&mut |_|Ok(()),&mut |expression|{if let ExprKind::Call(callee,args)=&mut expression.kind && matches!(&callee.kind,ExprKind::Name(value)if value=="backend"){
  let args=std::mem::take(args);*expression=crate::stdlib::futures::unwrap(super::call(&format!("__{}",binding.name),args,expression.span));
 }Ok(())})?;Ok(function)
}

#[cfg(test)]
mod tests{
 #[test]
 fn stored_binary_factory_typed_catch_survives_async_testing_import(){
  use crate::project::{SourceProvider,SourceId,load_project,check_graph};
  struct Source;
  impl SourceProvider for Source{
   fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
   fn read(&self,id:&SourceId)->Result<String,String>{
    if id.0!="main.tsr"{return Err(format!("Missing source: {}",id.0));}
    Ok("import {strictEqual} from 'std:testing';import {Request,requestBytes} from 'std:http';import {Bytes,fromArray} from 'std:bytes';type Factory=Function<Request,[method:ReadOnly<string>,url:ReadOnly<string>,body:Owned<Bytes>]>;async function main():Task<void>{const make:Factory=requestBytes;try{make('bad method','http://localhost/',unwrap(fromArray([1])));}catch(error){console.log(error.code);}}".into())
   }
  }
  let checked=check_graph(load_project(&Source,"main.tsr").unwrap());
  assert!(checked.is_ok(),"Stored Request factory must preserve HttpError through async catch: {:?}",checked.err());
 }
 #[test]
 fn request_factory_forwarders_keep_success_results_in_async_callable_contexts(){
  use crate::{project::{check_graph,load_project,FileSourceProvider},syntax::Type,native_signatures::NativeTypeId};
  let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/http-client-uploads/main.tsr");
  let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap_or_else(|errors|panic!("{errors:?}"));
  let program=&checked.analysis.program;
  for operation in ["request","requestBytes","requestStream","requestChannel"]{
   let suffix=format!("@http-adapter:{operation}");let mut found=false;
   for function in &program.functions{if !function.template&&function.name.ends_with(&suffix){
    found=true;let Type::Record(record)=function.result else{panic!("{operation} must return its Request owner, got {:?}",function.result)};
    assert_eq!(program.records[record].native_type,Some(NativeTypeId::HttpRequest));
   }}assert!(found,"Missing checked {operation} forwarder");
  }
 }
}

#[cfg(test)]
#[test]
fn cors_forwarders_preserve_optional_arity_for_direct_alias_and_stored_calls(){
 use crate::project::{SourceProvider,SourceId,load_project,check_graph};
 struct Source(String);
 impl SourceProvider for Source{
  fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
  fn read(&self,id:&SourceId)->Result<String,String>{if id.0!="main.tsr"{return Err(format!("Missing source: {}",id.0));}Ok(self.0.clone())}
 }
 let direct="import {App,Middleware,CorsOptions,cors} from 'std:http';type Factory=Function<Middleware,[options?:ReadOnly<CorsOptions>]>;function main(){const app=new App();app.use(cors());const alias=cors;app.use(alias());const stored:Factory=cors;app.use(stored());app.use(stored(undefined));app.use(stored({origins:['https://allowed.example']}));}";
 let typed_first="import {strictEqual} from 'std:testing';import {App,Middleware,CorsOptions,cors} from 'std:http';type Factory=Function<Middleware,[options?:ReadOnly<CorsOptions>]>;async function main():Task<void>{const stored:Factory=cors;const app=new App();app.use(stored());app.use(stored(undefined));app.use(stored({origins:['https://allowed.example']}));strictEqual(1,1);}";
 for source in [direct,typed_first]{let checked=check_graph(load_project(&Source(source.into()),"main.tsr").unwrap());assert!(checked.is_ok(),"Optional native factory forwarding must retain omission: {:?}",checked.err());}
}
