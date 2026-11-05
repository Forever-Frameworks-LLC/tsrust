use serde_json::{json,Value};
use tsrust_lsp::{project_diagnostics::source_id,project_intelligence::ProjectIndex,workspace::Workspace};
fn uri()->&'static str{if cfg!(windows){"file:///C:/tsrust-byte-operations/main.tsr"}else{"file:///tsrust-byte-operations/main.tsr"}}
fn ask(index:&ProjectIndex,source:&str,needle:&str,method:&str)->Value{
 let offset=source.rfind(needle).unwrap();let prefix=&source[..offset];let position=json!({"line":prefix.bytes().filter(|byte|*byte==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});index.request(method,&json!({"position":position}),uri()).unwrap().unwrap()
}
#[test]
fn byte_operation_catalog_hover_parameter_help_and_compiler_agree(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 for name in ["concat","copyInto","copyWithin","equals","compare","indexOf","readInt","writeInt","readUint","writeUint","readFloat32","writeFloat32","readFloat64","writeFloat64"]{
  let entry=catalog["std:bytes"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();let source=tsrust_compiler::format(entry["examples"][0]["source"].as_str().unwrap()).unwrap();
  let mut workspace=Workspace::default();workspace.open(source_id(uri()).unwrap(),1,source.clone());let checked=workspace.snapshot().check(&source_id(uri()).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
  let hover=ask(&index,&source,&format!("{name}("),"textDocument/hover");assert!(hover.to_string().contains("Ownership"),"{name}: {hover}");
  if name.starts_with("read")||name.starts_with("write") {let order=if source.contains("\"little\""){ "\"little\"" }else{"\"big\""};let help=ask(&index,&source,order,"textDocument/signatureHelp");assert_eq!(help["activeParameter"],entry["parameters"].as_array().unwrap().len()-1,"{help}");assert!(help.to_string().contains("byteOrder")&&help.to_string().contains("big"),"{help}");}
 }
}
#[test]
fn byte_operations_complete_partial_imports_and_preserve_stored_alias_docs(){
 for (prefix,expected) in [("copy",vec!["copy","copyInto","copyWithin"]),("readFloat",vec!["readFloat32","readFloat64"]),("write",vec!["writeInt","writeUint","writeFloat32","writeFloat64"]) ]{
  let source=format!("import {{ {prefix} }} from 'std:bytes';function main(){{}}");let mut workspace=Workspace::default();workspace.open(source_id(uri()).unwrap(),1,source.clone());let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&source_id(uri()).unwrap().0).unwrap();let completion=ask(&ProjectIndex::new(&graph),&source," }","textDocument/completion");let labels=completion.as_array().unwrap().iter().map(|item|item["label"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();assert_eq!(labels,expected.into_iter().collect(),"{completion}");
 }
 let source=tsrust_compiler::format("import {alloc,writeInt as put} from 'std:bytes';function main(){const saved=put;const bytes=unwrap(alloc(4));saved(bytes,1,-2,2,'big');}").unwrap();let mut workspace=Workspace::default();workspace.open(source_id(uri()).unwrap(),1,source.clone());let checked=workspace.snapshot().check(&source_id(uri()).unwrap()).unwrap();let hover=ask(&ProjectIndex::checked(&checked),&source,"saved(bytes,","textDocument/hover");assert!(hover.to_string().contains("integer-range")&&hover.to_string().contains("width"),"{hover}");
}
