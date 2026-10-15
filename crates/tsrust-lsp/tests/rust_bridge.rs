use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};
#[test]
fn rust_imports_offer_checked_signatures_docs_and_real_definitions(){
 let entry=SourceId("/rust-bridge/main.tsr".into());let dependency=SourceId("/rust-bridge/compute.rs".into());
 let source="import {sum as calculate,identity} from './compute.rs';function main(){const values=[1,2,3,4,5];const total=calculate(values);console.log(total);}";
 let rust="// Native implementation\n/// Sum without consuming the caller's buffer; function sum retains no borrow.\npub fn sum(values:&[i64])->i64{values.iter().sum()}\npub fn identity(values:Vec<i64>)->Vec<i64>{values}\n";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(dependency.clone(),1,rust.into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let request=|needle:&str,method:&str|index.request(method,&json!({"position":{"line":0,"character":source.find(needle).unwrap()}}),"file:///rust-bridge/main.tsr").unwrap().unwrap();
 let hover=request("calculate(values)","textDocument/hover").to_string();assert!(hover.contains("ReadOnly<int[]>")&&hover.contains("int")&&hover.contains("Sum without consuming"),"{hover}");
 let definition=request("calculate(values)","textDocument/definition");assert_eq!(definition["uri"],"file:///rust-bridge/compute.rs");assert_eq!(definition["range"]["start"]["line"],2);assert_eq!(definition["range"]["start"]["character"],7);
 let references=request("calculate(values)","textDocument/references");assert!(references.as_array().unwrap().iter().any(|item|item["uri"]=="file:///rust-bridge/compute.rs" && item["range"]["start"]["line"]==2),"{references}");
 let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":source.find("calculate(values)").unwrap()+10}}),"file:///rust-bridge/main.tsr").unwrap().unwrap().to_string();assert!(help.contains("ReadOnly<int[]>"),"{help}");
 workspace.open(dependency,2,"pub fn sum(values:&str)->bool{true}\npub fn identity(values:Vec<i64>)->Vec<i64>{values}".into());
 assert!(workspace.snapshot().check(&entry).is_err(),"edited Rust signature must invalidate caller");
}
#[test]
fn channel_capacity_hover_explains_buffering_and_backpressure(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let entry=catalog["std:threads"].as_array().unwrap().iter().find(|entry|entry["name"]=="Channel").unwrap();
 let docs=entry["documentation"].as_str().unwrap();for phrase in ["unread messages","not bytes or workers","third send waits","trySend returns full"]{assert!(docs.contains(phrase),"{phrase}");}
 assert!(entry["parameters"][0]["documentation"].as_str().unwrap().contains("receive() removes a message"));
}
