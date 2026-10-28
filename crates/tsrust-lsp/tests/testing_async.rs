use serde_json::{json,Value};
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_diagnostics::file_uri,project_intelligence::ProjectIndex};

fn document()->(SourceId,String){
    let root=std::fs::canonicalize(std::env::temp_dir()).unwrap();
    let id=SourceId(root.join(format!("tsrust-async-editor-{}",std::process::id())).join("async.test.tsr").to_string_lossy().into_owned());
    let uri=file_uri(&id).unwrap().to_string();(id,uri)
}
fn ask(index:&ProjectIndex,uri:&str,source:&str,needle:&str,method:&str)->Value{
    let offset=source.rfind(needle).unwrap();let before=&source[..offset];
    index.request(method,&json!({"position":{"line":before.bytes().filter(|b|*b==b'\n').count(),"character":before.rsplit('\n').next().unwrap().encode_utf16().count()}}),uri).unwrap().unwrap()
}

#[test]
fn async_test_and_hook_editor_help_shows_both_callback_results(){
    let source="import {beforeAll,test,strictEqual} from 'std:testing';import {delay} from 'std:futures';let ready=false;beforeAll(async()=>{await delay(0);ready=true;});test('awaited',async()=>{await delay(0);strictEqual(ready,true);});";
    let (id,uri)=document();let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check_with_diagnostics(&id).unwrap();assert!(checked.warnings().is_empty());
    let index=ProjectIndex::checked(&checked);
    for (needle,method) in [("test('awaited'","textDocument/hover"),("beforeAll(async","textDocument/hover"),("async()=>{await delay(0);strictEqual","textDocument/signatureHelp"),("async()=>{await delay(0);ready","textDocument/signatureHelp")]{
        let result=ask(&index,&uri,source,needle,method);let text=result.to_string();
        assert!(text.contains("Task<void>"),"{result}");assert!(text.contains("Owned<Function<void>>"),"{result}");
        assert!(!text.contains("async callbacks are unsupported"),"{result}");
    }
    let complete=ask(&index,&uri,source,"st,strictEqual","textDocument/completion");
    let test=complete.as_array().unwrap().iter().find(|entry|entry["label"]=="test").expect("test import completion");
    assert!(test.to_string().contains("Task<void>"),"{test}");
}

#[test]
fn async_callback_diagnostics_follow_changed_and_repaired_buffers(){
    let valid="import {test} from 'std:testing';test('async',async()=>{});";
    let (id,_)=document();let mut workspace=Workspace::default();workspace.open(id.clone(),1,valid.into());
    assert!(workspace.snapshot().check_with_diagnostics(&id).is_ok());
    assert!(workspace.change(&id,2,valid.replace("async()=>{}","async()=>{return 7;}")));
    let errors=workspace.snapshot().check_with_diagnostics(&id).err().expect("non-void task rejection");
    assert!(errors.iter().any(|error|error.code=="test-callback-result"),"{errors:?}");
    assert!(workspace.change(&id,3,valid.into()));assert!(workspace.snapshot().check_with_diagnostics(&id).is_ok());
}

#[test]
fn immediate_suite_lowering_keeps_public_editor_help(){
    let source="import {describe,test} from 'std:testing';let value=1;describe('suite',()=>{console.log(value);test('async',async()=>{});});";
    let (id,uri)=document();let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());
    let checked=workspace.snapshot().check_with_diagnostics(&id).unwrap();assert!(checked.warnings().is_empty());
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,&uri,source,"describe('suite'","textDocument/hover");
    let text=hover.to_string();assert!(text.contains("describe"),"{hover}");assert!(text.contains("registration"),"{hover}");
    assert!(!text.contains("invokeSuite"),"{hover}");
}
