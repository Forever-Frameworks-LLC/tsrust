use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};
#[test]
fn task_hover_and_await_result_use_checked_project_types(){
 let entry=SourceId("/async/main.tsr".into());let source="async function answer():Task<int>{return 42;}async function main():Task<void>{const task=answer();const result=await task;console.log(result);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 for(needle,expected)in [("task=","Task<int>"),("result=","int"),("answer()","Task<int>"),("main()","Task<void>")]{let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find(needle).unwrap()}}),"file:///async/main.tsr").unwrap().unwrap().to_string();assert!(hover.contains(expected),"{hover}");}
}
#[test]
fn unused_task_is_warning_and_invalid_await_is_error(){
 let source="async function answer():Task<int>{return 42;}async function main():Task<void>{const task=answer();}";
 let diagnostics=tsrust_lsp::diagnostics(source,"file:///async.tsr").unwrap();assert!(diagnostics.iter().any(|d|d.severity==Some(lsp_types::DiagnosticSeverity::WARNING)),"{diagnostics:?}");
 let invalid="function main(){await 1;}";let diagnostics=tsrust_lsp::diagnostics(invalid,"file:///async.tsr").unwrap();assert!(diagnostics.iter().any(|d|d.severity==Some(lsp_types::DiagnosticSeverity::ERROR)));
}
#[test]
fn imported_async_declaration_and_unused_diagnostic_have_source_locations(){
 let entry=SourceId("/async/main.tsr".into());let source="import {answer} from './work.tsr';async function main():Task<void>{const abandoned=answer();}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(SourceId("/async/work.tsr".into()),1,"export async function answer():Task<int>{return 42;}".into());
 let checked=workspace.snapshot().check(&entry).unwrap();let warnings=checked.warnings();assert!(warnings.iter().any(|warning|warning.location.source==entry && warning.code=="unused-task"),"{warnings:?}");
}
