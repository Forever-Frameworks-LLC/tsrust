//! Rust 1.99.1 compatibility shape, verified by the real shlex 2.0.1 rustdoc job.
//! Only format 61 is admitted; unsupported formats require explicit source fallback.
use super::*;
use serde_json::Value;
const FORMAT:u64=61;
fn required<'a>(value:&'a Value,key:&str)->Result<&'a Value,String>{value.get(key).ok_or_else(||format!("Missing rustdoc field {key}."))}
fn array<'a>(value:&'a Value,key:&str)->Result<&'a Vec<Value>,String>{required(value,key)?.as_array().ok_or_else(||format!("Invalid rustdoc array {key}."))}
fn flag(value:&Value,key:&str)->Result<bool,String>{required(value,key)?.as_bool().ok_or_else(||format!("Invalid rustdoc boolean {key}."))}
fn ty(value:&Value,paths:&Value)->Result<String,String>{
 if let Some(value)=value.get("primitive"){return match value.as_str(){Some(value @ ("i64"|"f64"|"bool"|"str"))=>Ok(value.into()),_=>Err("Primitive type is outside the supported owned boundary.".into())};}
 if let Some(value)=value.get("tuple"){if value.as_array().is_some_and(Vec::is_empty){return Ok("()".into());}return Err("Non-unit tuples need an explicit adapter.".into());}
 if let Some(value)=value.get("borrowed_ref"){if !required(value,"lifetime")?.is_null(){return Err("Explicit borrowed lifetimes need an adapter.".into());}return Ok(format!("&{}{}",if flag(value,"is_mutable")?{"mut "}else{""},ty(required(value,"type")?,paths)?));}
 if let Some(value)=value.get("slice"){return Ok(format!("[{}]",ty(value,paths)?));}
 let path=value.get("resolved_path").ok_or("Generic, opaque, callable or pointer type needs an explicit adapter.")?;
 let id=required(path,"id")?.to_string();let target=paths.get(&id).ok_or("Resolved type is missing its rustdoc path.")?;
 let components=array(target,"path")?.iter().map(|part|part.as_str().ok_or("Invalid rustdoc path.")).collect::<Result<Vec<_>,_>>()?;let canonical=components.join("::");
 let name=match canonical.as_str(){"alloc::string::String"=>"String".into(),"alloc::vec::Vec"=>"Vec".into(),"core::option::Option"=>"Option".into(),"core::result::Result"=>"Result".into(),_=>{
  if required(target,"crate_id")?.as_u64()!=Some(0)||!matches!(target.get("kind").and_then(Value::as_str),Some("struct"|"enum")){return Err(format!("Type {canonical} needs an explicit adapter."));}
  let name=required(path,"path")?.as_str().ok_or("Invalid local type path.")?;if syn::parse_str::<syn::Ident>(name).is_err(){return Err("Only root concrete data types can be generated.".into());}name.into()
 }};
 let args=required(path,"args")?;if args.is_null(){return Ok(name);}
 let args=required(args,"angle_bracketed")?;if !array(args,"constraints")?.is_empty(){return Err("Associated type constraints need an explicit adapter.".into());}
 let args=array(args,"args")?.iter().map(|arg|ty(required(arg,"type")?,paths)).collect::<Result<Vec<_>,_>>()?;Ok(format!("{name}<{}>",args.join(",")))
}
fn signature(item:&Value,paths:&Value)->Result<String,String>{
 let function=required(required(item,"inner")?,"function")?;let header=required(function,"header")?;
 if flag(header,"is_const")?||flag(header,"is_unsafe")?||flag(header,"is_async")?||required(header,"abi")?.as_str()!=Some("Rust"){return Err("Only safe, concrete synchronous Rust functions can be generated.".into());}
 let generics=required(function,"generics")?;if !array(generics,"params")?.is_empty()||!array(generics,"where_predicates")?.is_empty(){return Err("Generic exports need an explicit adapter.".into());}
 let signature=required(function,"sig")?;if flag(signature,"is_c_variadic")?{return Err("Variadic exports need an explicit adapter.".into());}
 let mut inputs=Vec::new();for input in array(signature,"inputs")?{let input=input.as_array().filter(|input|input.len()==2).ok_or("Invalid rustdoc parameter.")?;let name=input[0].as_str().ok_or("Invalid parameter name.")?;if syn::parse_str::<syn::Ident>(name).is_err()||name.starts_with("r#"){return Err("Parameter names require an explicit adapter.".into());}inputs.push(format!("{name}:{}",ty(&input[1],paths)?));}
 let name=required(item,"name")?.as_str().ok_or("Missing export name.")?;if syn::parse_str::<syn::Ident>(name).is_err()||name.starts_with("r#"){return Err("Invalid export name.".into());}
 let output=required(signature,"output")?;let output=if output.is_null(){"()".into()}else{ty(output,paths)?};
 let docs=item.get("docs").and_then(Value::as_str).unwrap_or("").replace("*/","* /").lines().map(|line|format!("/// {line}\n")).collect::<String>();
 Ok(format!("{docs}pub fn {name}({})->{output}{{unreachable!()}}\n",inputs.join(",")))
}
/// Read the actual pinned JSON shape, then use the same owned boundary validator
/// and code generator as source fallback. Source supplies reachable root data
/// declarations only; crate function bodies are never copied.
pub fn generate(crate_name:&str,json:&str,root_source:&str,selected:&[String])->Result<Generated,String>{
 let selected=selected_names(selected)?;
 let document:Value=serde_json::from_str(json).map_err(|error|error.to_string())?;
 if document.get("format_version").and_then(Value::as_u64)!=Some(FORMAT){return Err("Unsupported rustdoc JSON format; expected pinned Rust 1.99.1 format 61. Use explicit source fallback or regenerate with the pinned toolchain.".into());}
 let index=required(&document,"index")?;let root=required(&document,"root")?.to_string();let root=index.get(&root).ok_or("Missing rustdoc crate root.")?;
 if root.get("name").and_then(Value::as_str)!=Some(crate_name){return Err("Rustdoc crate name does not match the selected binding dependency.".into());}
 let module=required(required(root,"inner")?,"module")?;if !flag(module,"is_crate")?{return Err("Rustdoc root is not a crate module.".into());}
 let paths=required(&document,"paths")?;let parsed=syn::parse_file(root_source).map_err(|error|error.to_string())?;let mut normalized=String::new();
 for item in &parsed.items{match item{syn::Item::Struct(_)|syn::Item::Enum(_)=>{normalized.push_str(text(root_source,item.span()));normalized.push('\n');},syn::Item::Impl(item)=>{if let Some(name)=foreign_owner(item){normalized.push_str(&format!("impl UnprovenForeignBoundaryTrait for {name}{{}}\n"));}},_=>{}}}
 let mut skipped=Vec::new();let mut admitted=Vec::new();for name in selected{
  let matches=array(module,"items")?.iter().filter_map(|id|index.get(&id.to_string())).filter(|item|item.get("name").and_then(Value::as_str)==Some(&name)&&item.get("visibility").and_then(Value::as_str)==Some("public")).collect::<Vec<_>>();
  let result=if matches.len()==1{signature(matches[0],paths)}else{Err("Selected name must be one public root function; methods and reexports need an explicit adapter.".into())};
  match result{Ok(function)=>{normalized.push_str(&function);admitted.push(name);},Err(reason)=>{let signature=parsed.items.iter().find_map(|item|if let syn::Item::Fn(function)=item{(function.sig.ident==name).then(||text(root_source,function.sig.span()).to_owned())}else{None});skipped.push(Skipped{name,reason,signature});}}
 }
 let mut output=if admitted.is_empty(){Generated{adapter:String::new(),declarations:String::new(),stubs:String::new(),skipped:Vec::new(),selected:Vec::new()}}else{source(crate_name,&normalized,&admitted)?};
 output.skipped.extend(skipped);output.skipped.sort_by(|a,b|a.name.cmp(&b.name));
 output.stubs=output.skipped.iter().map(stub).collect();Ok(output)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn pinned_real_shlex_rustdoc_selects_root_split_and_reports_generic_borrowed_items(){
  let json=include_str!("fixtures/shlex-rustdoc-1.99.1.json");let source="pub fn split(in_str:&str)->Option<Vec<String>>{None}";
  let output=generate("shlex",json,source,&["split".into(),"try_join".into(),"try_quote".into()]).unwrap();assert_eq!(output.selected,["split"]);assert_eq!(output.skipped.len(),2);assert!(output.adapter.contains("shlex::split(in_str)"));assert!(output.adapter.contains("Option<Vec<String>>"));assert!(output.declarations.contains("RustAdapterOption<string[]>"));assert!(!output.adapter.contains("u8"),"must not choose the bytes::split homonym");
  let mut other:Value=serde_json::from_str(json).unwrap();other["format_version"]=Value::from(60);assert!(generate("shlex",&other.to_string(),source,&["split".into()]).is_err());assert!(generate("different",json,source,&["split".into()]).is_err());
 }
}

#[cfg(test)]mod foreign_ownership_tests{
 use super::*;
 #[test]fn both_discovery_paths_reject_custom_drop_but_move_ordinary_owned_fields(){
  let mut document:Value=serde_json::from_str(include_str!("fixtures/shlex-rustdoc-1.99.1.json")).unwrap();
  let row=serde_json::json!({"resolved_path":{"path":"Row","id":9000,"args":null}});
  document["index"]["148"]["inner"]["function"]["sig"]["inputs"]=serde_json::json!([["value",row.clone()]]);document["index"]["148"]["inner"]["function"]["sig"]["output"]=row;
  document["paths"]["9000"]=serde_json::json!({"crate_id":0,"path":["shlex","Row"],"kind":"struct"});
  let plain="pub struct Row{pub words:Vec<String>}pub fn split(value:Row)->Row{value}";
  let accepted=generate("shlex",&document.to_string(),plain,&["split".into()]).unwrap();assert_eq!(accepted.selected,["split"]);assert!(accepted.adapter.contains("shlex::Row{"));assert!(accepted.adapter.contains("native.words"));
  for implementation in ["impl std::ops::Drop for Row{fn drop(&mut self){}}","use std::ops::Drop as D;impl D for crate::Row{fn drop(&mut self){}}"]{
   let input=format!("{plain}{implementation}");for output in [generate("shlex",&document.to_string(),&input,&["split".into()]).unwrap(),source("shlex",&input,&["split".into()]).unwrap()]{assert!(output.selected.is_empty());assert_eq!(output.skipped.len(),1);assert!(output.skipped[0].reason.contains("Trait-implemented foreign data"));assert!(!output.stubs.is_empty());}
  }
 }
}

#[cfg(test)]mod selection_validation_tests{
 use super::*;
 #[test]fn selected_names_cannot_inject_stub_code_in_either_discovery_path(){
  let json=include_str!("fixtures/shlex-rustdoc-1.99.1.json");
  for name in ["split\npub fn injected(){}","r#unsafe","parent::split",""]{assert!(generate("shlex",json,"",&[name.into()]).is_err());assert!(source("shlex","",&[name.into()]).is_err());}
 }
}
