mod support;
use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};
#[test]
fn aliased_cache_methods_keep_generic_help_and_checked_loans(){
 let entry=SourceId("/cache/main.tsr".into());let source="import {LruCache as Cache} from 'std:cache';function main(){const cache=new Cache<int,string>(2);unwrap(cache.set(1,'one'));console.log(cache.peek(1)!);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find("peek").unwrap()}}),"file:///cache/main.tsr").unwrap().unwrap().to_string();assert!(hover.contains("string")&&hover.contains("exclusive")&&hover.contains("TTL"),"{hover}");
 let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":source.find("set(1,").unwrap()+6}}),"file:///cache/main.tsr").unwrap().unwrap().to_string();assert!(help.contains("value: string")&&help.contains("ttlMs"),"{help}");
 workspace.open(entry.clone(),2,"import {LruCache} from 'std:cache';function main(){const c=new LruCache<int,string>(1);unwrap(c.set(1,'x'));const value=c.get(1);c.clear();console.log(value!);}".into());assert!(workspace.snapshot().check(&entry).is_err());
}

#[test]
fn cache_help_and_missing_value_reads_work_in_live_server(){
 use support::session::Session;
 let text="import {LruCache as Cache} from 'std:cache';function main(){const cache=new Cache<int,string>(2);unwrap(cache.set(1,'one'));const value=cache.get(1);if(value!==undefined){console.log(value);}cache.clear();}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 let offset=text.find("cache.get").unwrap()+6;let hover=session.hover("main.tsr",text,offset).unwrap_or_default();assert!(hover.contains("undefined")&&hover.contains("exclusive"),"{hover}");
 for fragment in ["Cache<int,string>(","cache.set("]{
  let position=text.find(fragment).unwrap()+fragment.len();let help=session.request("textDocument/signatureHelp",json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":position}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{fragment}: {help}");
 }
}
