//! Explicit, deterministic source discovery. It never resolves or downloads crates.
use super::*;
use serde::Serialize;
#[path="rust_rustdoc.rs"] pub mod rustdoc;
#[derive(Debug,Serialize)]
pub struct Skipped {pub name:String,pub reason:String,#[serde(skip_serializing_if="Option::is_none")]pub signature:Option<String>}
#[derive(Debug)]
pub struct Generated {pub adapter:String,pub declarations:String,pub stubs:String,pub skipped:Vec<Skipped>,pub selected:Vec<String>}
fn selected_names(selected:&[String])->Result<BTreeSet<String>,String>{
 if selected.is_empty(){return Err("Select at least one concrete export.".into());}
 if selected.iter().any(|name|syn::parse_str::<syn::Ident>(name).is_err()||name.starts_with("r#")){return Err("Selected names must be ordinary Rust identifiers.".into());}
 Ok(selected.iter().cloned().collect())
}
fn comment(value:&str)->String{value.replace('\r'," ").lines().map(|line|format!("// {}\n",line.chars().map(|character|if character.is_control(){' '}else{character}).collect::<String>())).collect()}
fn stub(item:&Skipped)->String{
 let signature=comment(item.signature.as_deref().unwrap_or("Signature unavailable; select one concrete root declaration."));
 format!("{}{signature}pub fn adapt_{}() {{\n    todo!(\"Choose concrete owned parameters/result, then call backend safely\");\n}}\n\n",comment(&format!("{}: {}",item.name,item.reason)),item.name)
}
fn named(ty:&Type,pending:&mut Vec<String>){match ty{Type::Named(name)=>pending.push(name.clone()),Type::Array(inner)|Type::Option(inner,_)|Type::Borrow(inner,_)=>named(inner,pending),Type::Result(ok,err)=>{named(ok,pending);named(err,pending);},_=>{}}}
fn text<'a>(source:&'a str,span:proc_macro2::Span)->&'a str{let span=range(source,span);&source[span.start..span.end]}
fn nominal(ty:&Type)->bool{match ty{Type::Named(_)=>true,Type::Array(inner)|Type::Option(inner,_)=>nominal(inner),Type::Result(ok,err)=>nominal(ok)||nominal(err),_=>false}}
fn foreign_owner(item:&syn::ItemImpl)->Option<String>{if item.trait_.is_none(){return None;}if let syn::Type::Path(path)=item.self_ty.as_ref(){path.path.segments.last().map(|segment|segment.ident.to_string())}else{None}}
fn moved(ty:&Type,value:&str,to_foreign:bool,crate_name:&str,types:&BTreeMap<String,Data>)->String{
 if !nominal(ty){return value.into();}
 match ty{
  Type::Array(inner)=>format!("({value}).into_iter().map(|item|{}).collect::<Vec<_>>()",moved(inner,"item",to_foreign,crate_name,types)),
  Type::Option(inner,_)=>format!("({value}).map(|item|{})",moved(inner,"item",to_foreign,crate_name,types)),
  Type::Result(ok,err)=>format!("({value}).map(|item|{}).map_err(|error|{})",moved(ok,"item",to_foreign,crate_name,types),moved(err,"error",to_foreign,crate_name,types)),
  Type::Named(name)=>{let target=if to_foreign{format!("{crate_name}::{name}")}else{name.clone()};let from=if to_foreign{name.clone()}else{format!("{crate_name}::{name}")};match &types[name]{
   Data::Struct(fields)=>format!("{{let native={value};{target}{{{}}}}",fields.iter().map(|(name,ty)|format!("{name}:{}",moved(ty,&format!("native.{name}"),to_foreign,crate_name,types))).collect::<Vec<_>>().join(",")),
   Data::Enum(variants)=>{let arms=variants.iter().map(|variant|{let names=variant.fields.iter().enumerate().map(|(index,_)|format!("item{index}")).collect::<Vec<_>>();let source=if variant.tuple{format!("({})",names.join(","))}else if variant.named{format!("{{{}}}",variant.fields.iter().zip(&names).map(|((field,_),name)|format!("{field}:{name}")).collect::<Vec<_>>().join(","))}else{String::new()};let values=variant.fields.iter().zip(&names).map(|((field,ty),name)|{let value=moved(ty,name,to_foreign,crate_name,types);if variant.named{format!("{field}:{value}")}else{value}}).collect::<Vec<_>>().join(",");let output=if variant.tuple{format!("({values})")}else if variant.named{format!("{{{values}}}")}else{String::new()};format!("{from}::{}{source}=>{target}::{}{output}",variant.name,variant.name)}).collect::<Vec<_>>().join(",");format!("match {value}{{{arms}}}")}
  }},_=>value.into()
 }
}
fn function(source:&str,file:&syn::File,function:&syn::ItemFn,crate_name:&str)->Result<String,String>{
 let sig=&function.sig;
 if sig.asyncness.is_some()||matches!(sig.safety,syn::Safety::Unsafe(_))||sig.constness.is_some()||sig.abi.is_some()||sig.variadic.is_some()||!sig.generics.params.is_empty()||sig.generics.where_clause.is_some()||function.attrs.iter().any(|attr|!attr.path().is_ident("doc")){return Err("Only unconditional, safe, concrete synchronous free functions can be generated.".into());}
 let mut pending=Vec::new();let mut arguments=Vec::new();let mut parameter_types=Vec::new();
 for input in &sig.inputs{let syn::FnArg::Typed(input)=input else{return Err("Methods need an explicit adapter.".into())};let syn::Pat::Ident(pattern)=input.pat.as_ref() else{return Err("Parameter patterns need an explicit adapter.".into())};if pattern.by_ref.is_some()||pattern.subpat.is_some(){return Err("Parameter patterns need an explicit adapter.".into());}arguments.push(pattern.ident.to_string());let ty=shape(&input.ty,false,"RustAdapter")?;named(&ty,&mut pending);parameter_types.push(ty);}
 if let syn::ReturnType::Type(_,ty)=&sig.output{named(&shape(ty,true,"RustAdapter")?,&mut pending);}
 let mut records=BTreeMap::new();while let Some(name)=pending.pop(){if records.contains_key(&name){continue;}
  let mut matches=file.items.iter().filter(|item|match item{syn::Item::Struct(item)=>item.ident==name,syn::Item::Enum(item)=>item.ident==name,_=>false});
  let item=matches.next().ok_or_else(||format!("Type {name} is not a concrete public root record or enum; write an explicit adapter."))?;
  if matches.next().is_some(){return Err(format!("Conditional or duplicate type {name} needs an explicit adapter."));}
  if file.items.iter().any(|item|matches!(item,syn::Item::Impl(item)if foreign_owner(item).as_deref()==Some(name.as_str()))){return Err(format!("Trait-implemented foreign data {name} needs an explicit adapter; source discovery cannot prove absence of an aliased Drop implementation."));}
  let fields=match item{syn::Item::Struct(item)=>item.fields.iter().collect::<Vec<_>>(),syn::Item::Enum(item)=>item.variants.iter().flat_map(|variant|variant.fields.iter()).collect(),_=>unreachable!()};
  for field in fields{named(&shape(&field.ty,true,"RustAdapter")?,&mut pending);}
  records.insert(name,text(source,item.span()).to_owned());
 }
 let docs=documentation(&function.attrs);let docs=docs.lines().map(|line|format!("/// {line}\n")).collect::<String>();
 let declarations=records.values().cloned().collect::<Vec<_>>().join("\n");let data_file=syn::parse_file(&declarations).map_err(|error|error.to_string())?;let types=data::collect(&data_file,"RustAdapter").map_err(|(_,message)|message)?;
 let arguments=parameter_types.iter().zip(&arguments).map(|(ty,value)|moved(ty,value,true,crate_name,&types)).collect::<Vec<_>>().join(",");
 let result=if let syn::ReturnType::Type(_,ty)=&sig.output{shape(ty,true,"RustAdapter")?}else{Type::Void};let call=format!("{crate_name}::{}({arguments})",sig.ident);let output=moved(&result,&call,false,crate_name,&types);
 let wrapper=format!("{declarations}\n{docs}pub {}{{{output}}}\n",text(source,sig.span()));
 module(SourceFile{id:SourceId("generated.rs".into()),text:wrapper.clone()}).map_err(|error|format!("{error:?}"))?;Ok(wrapper)
}
/// Discover only selected root exports from an explicitly supplied crate source.
/// Unsupported selected items remain in the skipped report and editable comments.
/// Bodies are never copied: generated calls use the named locked crate dependency.
pub fn source(crate_name:&str,source:&str,selected:&[String])->Result<Generated,String>{
 if syn::parse_str::<syn::Ident>(crate_name).is_err()||crate_name.starts_with("r#"){return Err("Expected a Rust dependency identifier.".into());}
 let file=syn::parse_file(source).map_err(|error|error.to_string())?;
 if !file.attrs.iter().all(|attr|attr.path().is_ident("doc")||attr.path().is_ident("allow")||attr.path().is_ident("warn")||attr.path().is_ident("deny")||attr.path().is_ident("forbid")||attr.path().is_ident("no_std")||(attr.path().is_ident("cfg_attr")&&attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated).is_ok_and(|args|args.len()>=2&&args.iter().skip(1).all(|meta|meta.path().is_ident("no_std"))))){return Err("Crate-level configuration requires explicit normalized input; source fallback cannot infer active cfg.".into());}
 let selected=selected_names(selected)?;
 let mut output=Generated{adapter:String::new(),declarations:String::new(),stubs:String::new(),skipped:Vec::new(),selected:Vec::new()};let mut records=BTreeMap::new();let mut wrappers=Vec::new();
 for name in selected{
  let candidates=file.items.iter().filter(|item|matches!(item,syn::Item::Fn(item)if item.sig.ident==name&&matches!(item.vis,syn::Visibility::Public(_)))).collect::<Vec<_>>();
  let result=if candidates.len()==1{if let syn::Item::Fn(item)=candidates[0]{function(source,&file,item,crate_name)}else{unreachable!()}}else{Err("Selected name must resolve to exactly one unconditional public root function; methods, reexports and macros need an explicit adapter.".into())};
  match result{Ok(wrapper)=>{let parsed=syn::parse_file(&wrapper).map_err(|error|error.to_string())?;for item in parsed.items{match item{syn::Item::Struct(ref value)=>{records.insert(value.ident.to_string(),text(&wrapper,item.span()).to_owned());},syn::Item::Enum(ref value)=>{records.insert(value.ident.to_string(),text(&wrapper,item.span()).to_owned());},_=>wrappers.push(text(&wrapper,item.span()).to_owned())}}output.selected.push(name);},Err(reason)=>{let signature=if let [item]=candidates.as_slice(){if let syn::Item::Fn(function)=item{Some(text(source,function.sig.span()).to_owned())}else{None}}else{None};let skipped=Skipped{name,reason,signature};output.stubs.push_str(&stub(&skipped));output.skipped.push(skipped);}}
 }
 output.adapter=format!("{}\n{}\n",records.values().cloned().collect::<Vec<_>>().join("\n"),wrappers.join("\n"));
 if !output.selected.is_empty(){let checked=module(SourceFile{id:SourceId("generated.rs".into()),text:output.adapter.clone()}).map_err(|error|format!("{error:?}"))?;let rust=checked.rust.as_ref().unwrap();let parsed=syn::parse_file(&output.adapter).map_err(|error|error.to_string())?;
  let prefix=data::option_prefix(&parsed);if rust.types.keys().any(|name|data::uses_option(&Type::Named(name.clone()),&rust.types))||rust.exports.iter().any(|export|export.parameters.iter().chain(std::iter::once(&export.result)).any(|ty|data::uses_option(ty,&rust.types))){output.declarations.push_str(&data::option_declarations(&prefix));}
  output.declarations.push_str(&data::declarations_only(&rust.types,&parsed));
  for export in &rust.exports{let function=parsed.items.iter().find_map(|item|if let syn::Item::Fn(function)=item{(function.sig.ident==export.name).then_some(function)}else{None}).unwrap();let params=export.parameters.iter().enumerate().map(|(index,ty)|format!("{}:{}",if let syn::FnArg::Typed(input)=&function.sig.inputs[index]{if let syn::Pat::Ident(pattern)=input.pat.as_ref(){pattern.ident.to_string()}else{unreachable!()}}else{unreachable!()},if ty.owned(){format!("Owned<{}>",ty.source())}else{ty.source()})).collect::<Vec<_>>().join(",");output.declarations.push_str(&format!("/** {} */\nexport type {}Signature = Function<{},[{params}]>;\n",documentation(&function.attrs),export.name,export.result.source()));}
 }Ok(output)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn selected_source_fallback_is_deterministic_and_keeps_nested_tags(){
  let input="pub struct Row{pub words:Vec<String>} /// Move rows exactly once.\npub fn rows(value:Option<Vec<Row>>)->Option<Vec<Row>>{value} pub fn generic<T>(value:T)->T{value} pub unsafe fn danger(){}";
  let selected=vec!["rows".into(),"generic".into(),"danger".into()];
  let first=source("example",input,&selected).unwrap();let second=source("example",input,&selected.into_iter().rev().collect::<Vec<_>>()).unwrap();
  assert_eq!(first.adapter,second.adapter);assert_eq!(first.declarations,second.declarations);assert_eq!(first.selected,["rows"]);assert_eq!(first.skipped.len(),2);assert!(first.stubs.contains("pub fn adapt_generic()"));assert!(first.stubs.contains("todo!("));for item in syn::parse_file(&first.stubs).unwrap().items{let syn::Item::Fn(function)=item else{panic!("stub must contain only safe functions")};assert!(!matches!(function.sig.safety,syn::Safety::Unsafe(_)));assert!(function.sig.asyncness.is_none()&&function.sig.abi.is_none()&&function.sig.generics.params.is_empty()&&function.sig.inputs.is_empty());}
  assert!(first.adapter.contains("example::rows("));assert!(first.adapter.contains("example::Row{"));assert!(first.adapter.contains("native.words"));assert!(!first.adapter.contains("unsafe"));assert!(first.declarations.contains("RustAdapterOption"));assert!(first.declarations.contains("value:Owned<"));assert!(!first.declarations.contains("rustSeed"));
 }
 #[test]fn discovery_never_admits_drop_borrows_or_conditional_exports(){
  for input in ["pub struct Row{pub value:String}impl std::ops::Drop for Row{fn drop(&mut self){}}pub fn selected(value:Row)->Row{value}","pub fn selected()->&'static str{\"x\"}","#[cfg(feature=\"other\")]pub fn selected()->i64{1}","pub fn selected(value:Vec<Result<String,String>>){}","use std::ops::Drop as D;pub struct Row{pub words:Vec<String>}impl D for crate::Row{fn drop(&mut self){}}pub fn selected(value:Row)->Row{value}"]{
   let output=source("example",input,&["selected".into()]).unwrap();assert!(output.selected.is_empty(),"{input}");assert_eq!(output.skipped.len(),1);assert!(!output.stubs.is_empty());
  }
 }
 #[test]fn explicit_options_do_not_flatten_nested_absence_or_change_plain_adapters(){
  let checked=module(SourceFile{id:SourceId("options.rs".into()),text:"pub fn nested(value:Option<Option<Vec<String>>>)->Option<Option<Vec<String>>>{value} pub fn unit()->Option<()>{Some(())}".into()}).unwrap();
  assert!(checked.declaration_source.contains("RustAdapterOption<RustAdapterOption<string[]>>"));assert!(checked.declaration_source.contains("RustAdapterUnitOption"));
  let collision=module(SourceFile{id:SourceId("collision.rs".into()),text:"pub struct RustAdapterNone{pub value:i64}pub fn tagged()->Option<i64>{None}".into()}).unwrap();assert!(collision.declaration_source.contains("RustAdapter_Option<int>"));
  let plain=module(SourceFile{id:SourceId("plain.rs".into()),text:"pub fn plain()->i64{1}".into()}).unwrap();assert!(!plain.declaration_source.contains("RustAdapter"));
 }
}

#[cfg(test)]mod authored_drop_owner_tests{
 use super::*;
 #[test]fn authored_boundary_drop_resolves_local_owners_without_foreign_name_aliasing(){
  let plain="pub struct Row{pub words:Vec<String>}pub fn echo(value:Row)->Row{value}";
  for implementation in ["impl Drop for Row{fn drop(&mut self){}}","impl std::ops::Drop for self::Row{fn drop(&mut self){}}","impl core::ops::Drop for crate::Row{fn drop(&mut self){}}","use std::ops::Drop as D;impl D for Row{fn drop(&mut self){}}","use core::ops::{Drop as D};impl D for self::Row{fn drop(&mut self){}}","use std::{ops as operations};impl operations::Drop for Row{fn drop(&mut self){}}"]{
   assert!(module(SourceFile{id:SourceId("drop.rs".into()),text:format!("{plain}{implementation}")}).is_err(),"accepted {implementation}");
  }
  for addition in ["","mod other{pub struct Row;}impl Drop for other::Row{fn drop(&mut self){}}","mod other{pub trait Drop{}}impl other::Drop for Row{}","mod other{pub trait Drop{}}use other::Drop as D;impl D for Row{}"]{
   let input=format!("{plain}{addition}");let parsed=syn::parse_file(&input).unwrap();let types=data::collect(&parsed,"RustAdapter").unwrap();assert!(data::declarations(&types,&parsed).contains("words: string[]"));if addition.is_empty(){assert!(module(SourceFile{id:SourceId("owned.rs".into()),text:input}).is_ok());}
  }
 }
}

#[cfg(test)]mod recursive_result_tests{
 use super::*;
 #[test]fn nested_results_are_owned_output_only_and_keep_unit_error_rejection(){
  let checked=module(SourceFile{id:SourceId("nested.rs".into()),text:"pub fn nested()->Result<Result<String,String>,Result<i64,String>>{Ok(Ok(String::new()))}".into()}).unwrap();assert!(checked.declaration_source.contains("Result<Result<string, string>, Result<int, string>>"));
  for input in ["pub fn bad(value:Option<Result<Result<String,String>,String>>){}","pub fn bad()->Result<Result<i64,()>,String>{todo!()}","pub fn bad()->Result<i64,()>{todo!()}"]{assert!(module(SourceFile{id:SourceId("bad.rs".into()),text:input.into()}).is_err(),"accepted {input}");}
 }
}
