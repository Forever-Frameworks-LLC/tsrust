mod support;
use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};

/// Unix virtual paths stay as written; Windows gains a drive so both are absolute.
fn host_uri(path:&str)->String{format!("file://{}{path}",if cfg!(windows){"/C:"}else{""})}
fn host_id(path:&str)->tsrust_compiler::project::SourceId{tsrust_lsp::project_diagnostics::source_id(&host_uri(path)).unwrap()}
#[test]
fn rust_imports_offer_checked_signatures_docs_and_real_definitions(){
 let entry=host_id("/rust-bridge/main.tsr");let dependency=host_id("/rust-bridge/compute.rs");
 let source="import {sum as calculate,identity} from './compute.rs';function main(){const values=[1,2,3,4,5];const total=calculate(values);console.log(total);}";
 let rust="// Native implementation\n/// Sum without consuming the caller's buffer; function sum retains no borrow.\npub fn sum(values:&[i64])->i64{values.iter().sum()}\npub fn identity(values:Vec<i64>)->Vec<i64>{values}\n";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(dependency.clone(),1,rust.into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let request=|needle:&str,method:&str|index.request(method,&json!({"position":{"line":0,"character":source.find(needle).unwrap()}}),&host_uri("/rust-bridge/main.tsr")).unwrap().unwrap();
 let hover=request("calculate(values)","textDocument/hover").to_string();assert!(hover.contains("ReadOnly<int[]>")&&hover.contains("int")&&hover.contains("Sum without consuming"),"{hover}");
 let definition=request("calculate(values)","textDocument/definition");assert_eq!(definition["uri"],host_uri("/rust-bridge/compute.rs"));assert_eq!(definition["range"]["start"]["line"],2);assert_eq!(definition["range"]["start"]["character"],7);
 let references=request("calculate(values)","textDocument/references");assert!(references.as_array().unwrap().iter().any(|item|item["uri"]==host_uri("/rust-bridge/compute.rs") && item["range"]["start"]["line"]==2),"{references}");
 let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":source.find("calculate(values)").unwrap()+10}}),&host_uri("/rust-bridge/main.tsr")).unwrap().unwrap().to_string();assert!(help.contains("ReadOnly<int[]>"),"{help}");
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
#[test]
fn binding_manifest_exposes_owned_data_and_tracks_rust_edits(){
 let entry=host_id("/rust-bindings/main.tsr");
 let source="import {Request,read} from './api.bindings.json';function main(){const request:Request={text:'hello'};const answer=read(request);console.log(answer.text);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());
 workspace.open(host_id("/rust-bindings/api.bindings.json"),1,r#"{"schemaVersion":2,"source":"./api.rs","dependencies":{}}"#.into());
 workspace.open(host_id("/rust-bindings/api.bindings.lock"),1,"version = 4".into());
 let rust=host_id("/rust-bindings/api.rs");
 workspace.open(rust.clone(),1,"pub struct Request{pub text:String}\n/// Read the owned request.\npub fn read(value:Request)->Request{value}".into());
 let mut session=tsrust_compiler::project::session::ProjectSession::default();
 let checked=session.check(&workspace.snapshot(),&entry.0).unwrap();let index=ProjectIndex::checked(&checked);
 let request=|needle:&str,method:&str|index.request(method,&json!({"position":{"line":0,"character":source.find(needle).unwrap()}}),&host_uri("/rust-bindings/main.tsr")).unwrap().unwrap();
 let hover=request("read(request)","textDocument/hover").to_string();assert!(hover.contains("Owned<Request>")&&hover.contains("Read the owned request"),"{hover}");
 let type_definition=request("Request={", "textDocument/definition");assert_eq!(type_definition["uri"],host_uri("/rust-bindings/api.rs"));assert_eq!(type_definition["range"]["start"]["line"],0);
 let definition=request("read(request)","textDocument/definition");assert_eq!(definition["uri"],host_uri("/rust-bindings/api.rs"));assert_eq!(definition["range"]["start"]["line"],2);
 let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":source.find("answer.text").unwrap()+7}}),&host_uri("/rust-bindings/main.tsr")).unwrap().unwrap().to_string();assert!(completion.contains("text"),"{completion}");
 workspace.open(rust,2,"pub struct Request{pub number:i64}pub fn read(value:Request)->Request{value}".into());
 assert!(session.check(&workspace.snapshot(),&entry.0).is_err());
}
#[test]
fn rust_record_hover_uses_readable_source_without_empty_doc_comments(){
 let entry=host_id("/pretty-rust/main.tsr");let source="import {Search} from './api.rs';function main(){const value:Search={pattern:'x',text:'text'};console.log(value.text);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(host_id("/pretty-rust/api.rs"),1,"pub struct Search{pub pattern:String,pub text:String}".into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find("Search={").unwrap()}}),&host_uri("/pretty-rust/main.tsr")).unwrap().unwrap().to_string();
 assert!(!hover.contains("/** */"),"{hover}");assert!(hover.contains("pattern: string;")&&hover.contains("text: string;"),"{hover}");
}
#[test]
fn managed_embedded_lock_changes_recheck_through_the_manifest(){
 let entry=host_id("/managed-rust/main.tsr");let manifest=host_id("/managed-rust/api.bindings.json");let native=host_id("/managed-rust/api.rs");
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,"import {answer} from './api.bindings.json';function main(){console.log(answer());}".into());workspace.open(native,1,"pub fn answer()->i64{42}".into());
 let config=|version:&str|serde_json::json!({"schemaVersion":1,"source":"./api.rs","dependencies":{},"lockfile":version}).to_string();workspace.open(manifest.clone(),1,config("version = 4"));
 let mut session=tsrust_compiler::project::session::ProjectSession::default();session.check(&workspace.snapshot(),&entry.0).unwrap();workspace.open(manifest,2,config("version = 4\n# updated"));session.check(&workspace.snapshot(),&entry.0).unwrap();assert!(session.stats().parsed>0);
}
#[test]
fn checked_width_adapter_help_preserves_borrow_and_error_contract(){
 let entry=host_id("/rust-width/main.tsr");let dependency=host_id("/rust-width/adapter.rs");
 let source="import {incrementWide} from './adapter.rs';function main(){const result=incrementWide('18446744073709551614',64);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(dependency,1,include_str!("../../../examples/rust-data-boundaries/adapter.rs").into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let answer=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.rfind("incrementWide").unwrap()}}),&host_uri("/rust-width/main.tsr")).unwrap().unwrap().to_string();
 for expected in ["ReadOnly<string>","Result<string, string>","width"]{assert!(answer.contains(expected),"{answer}");}
}

#[test]
fn tagged_owned_option_adapter_help_agrees_in_all_server_phases(){
 use support::session::Session;use std::time::Duration;
 let rust="/// Split shell words; invalid quoting returns the explicit none tag.\npub fn words(value:&str)->Option<Vec<String>>{Some(vec![value.into()])}";
 let before="import {words} from './words.rs';function main(){const result=";
 let after="('two words');console.log(result.kind);}";let source=format!("{before}words{after}");
 let entry=host_id("/adapter-options/main.tsr");let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.clone());workspace.open(host_id("/adapter-options/words.rs"),1,rust.into());assert!(workspace.snapshot().check(&entry).is_ok());
 for phase in ["immediate","paced","settled","broken"]{
  let broken=if phase=="broken"{" function unfinished( {"}else{""};let final_source=format!("{source}{broken}");
  let mut session=Session::start(&[("main.tsr",&final_source),("words.rs",rust)]);
  if phase=="immediate"||phase=="paced"{let after=format!("{after}{broken}");session.open("main.tsr",&format!("{before}{after}"));session.type_text("main.tsr",before,"words",&after,if phase=="paced"{Duration::from_millis(80)}else{Duration::ZERO});}else{session.open("main.tsr",&final_source);session.settle();}
  let hover=session.hover("main.tsr",&final_source,before.len()).unwrap_or_else(||panic!("{phase}: missing adapter hover"));for expected in ["ReadOnly<string>","RustAdapterOption<string[]>","invalid quoting"]{assert!(hover.contains(expected),"{phase}: {hover}");}
  let completion=session.completion("main.tsr",&final_source,final_source.find("result.kind").unwrap()+7).unwrap();assert!(completion.iter().any(|label|label=="kind"),"{phase}: {completion:?}");
  let uri=session.uri("main.tsr");let help=session.request("textDocument/signatureHelp",json!({"textDocument":{"uri":uri},"position":{"line":0,"character":before.len()+6}})).to_string();assert!(help.contains("value")&&help.contains("ReadOnly<string>"),"{phase}: {help}");
 }
}
