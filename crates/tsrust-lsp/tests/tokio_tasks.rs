use serde_json::{json,Value};
use tsrust_lsp::{intelligence::SourceIndex,workspace::Workspace,project_intelligence::ProjectIndex};
use tsrust_compiler::project::SourceId;
fn request(source:&str,needle:&str,method:&str)->Value{
 let offset=source.find(needle).unwrap()+needle.len()-usize::from(method=="textDocument/hover");
 SourceIndex::new(source).request(method,&json!({"position":{"line":0,"character":offset}}),"file:///tokio.tsr").unwrap()
}
#[test]
fn async_imports_and_capacity_help_explain_waiting_and_cleanup(){
 let source="import {delay,Channel} from 'std:futures';async function main():Task<void>{await delay(1);const queue=new Channel<int>(2);}";
 let hover=request(source,"await delay","textDocument/hover").to_string();assert!(hover.contains("lazy")&&hover.contains("31536000000"),"{hover}");
 let help=request(source,"new Channel<int>(","textDocument/signatureHelp").to_string();assert!(help.contains("queued messages")&&help.contains("without blocking"),"{help}");
}
#[test]
fn checked_job_and_await_result_have_types_and_abandonment_is_error(){
 let entry=SourceId("/tokio.tsr".into());let source="async function one():Task<int>{return 1;}async function main():Task<void>{const job=one().start();const value=await job;console.log(value);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 for(needle,expected)in[("job=","Job<int, undefined>"),("value=","int")]{let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find(needle).unwrap()}}),"file:///tokio.tsr").unwrap().unwrap();assert!(hover.to_string().contains(expected),"{hover}");}
 let abandoned=source.replace("const value=await job;console.log(value);", "job.cancel();");
 let diagnostics=tsrust_lsp::diagnostics(&abandoned,"file:///tokio.tsr").unwrap();assert!(diagnostics.iter().any(|error|error.severity==Some(lsp_types::DiagnosticSeverity::ERROR)),"{diagnostics:?}");
}

#[test]
fn yield_hover_uses_public_name(){
 let source="import {yield} from 'std:futures';async function main():Task<void>{await yield();}";
 let hover=request(source,"await yield","textDocument/hover").to_string();assert!(hover.contains("yield(): Task&lt;void&gt;") || hover.contains("yield(): Task<void>"),"{hover}");assert!(!hover.contains("yieldNow"));
}
