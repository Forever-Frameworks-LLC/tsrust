mod support;
use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_diagnostics::file_uri,project_intelligence::ProjectIndex};
fn position(source:&str,offset:usize)->serde_json::Value{let prefix=&source[..offset];json!({"line":prefix.bytes().filter(|b|*b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()})}
#[test]fn everyday_assertion_hover_signature_and_import_completion_agree(){
 let source="import {deepStrictEqual,notDeepStrictEqual,rejects} from 'std:testing';async function main():Task<void>{deepStrictEqual([[1]],[[1]]);notDeepStrictEqual([1],[2]);await rejects(async()=>{throw {code:'expected'};});}";
 let id=SourceId(std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("testing-everyday-editor-{}",std::process::id())).join("main.tsr").to_string_lossy().into_owned());let uri=file_uri(&id).unwrap().to_string();let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());let checked=workspace.snapshot().check_with_diagnostics(&id).unwrap();let index=ProjectIndex::checked(&checked);
 for name in["deepStrictEqual","notDeepStrictEqual","rejects"]{let offset=source.rfind(&format!("{name}(")).unwrap();let hover=index.request("textDocument/hover",&json!({"position":position(source,offset+2)}),&uri).unwrap().unwrap();assert!(hover.to_string().contains("Ownership:"),"{hover}");let signature=index.request("textDocument/signatureHelp",&json!({"position":position(source,offset+name.len()+1)}),&uri).unwrap().unwrap();assert!(signature.to_string().contains(name),"{signature}");}
 let completion=index.request("textDocument/completion",&json!({"position":position(source,source.find("deepStrictEqual").unwrap())}),&uri).unwrap().unwrap();for name in["deepStrictEqual","notDeepStrictEqual","rejects"]{assert!(completion.to_string().contains(name),"{completion}");}
}
#[test]fn nested_opaque_diagnostic_repairs_in_changed_buffer(){let valid="import {deepStrictEqual} from 'std:testing';function main(){deepStrictEqual([1],[1]);}";let id=SourceId("/tmp/testing-everyday-negative/main.tsr".into());let mut workspace=Workspace::default();workspace.open(id.clone(),1,valid.into());assert!(workspace.snapshot().check_with_diagnostics(&id).is_ok());let invalid="import {deepStrictEqual,trackedResource} from 'std:testing';function main(){deepStrictEqual([trackedResource()],[trackedResource()]);}";assert!(workspace.change(&id,2,invalid.into()));let errors=workspace.snapshot().check_with_diagnostics(&id).err().unwrap();assert!(errors.iter().any(|error|error.code=="native-capability"));assert!(workspace.change(&id,3,valid.into()));assert!(workspace.snapshot().check_with_diagnostics(&id).is_ok());}

#[test]
fn everyday_assertions_agree_in_all_real_server_phases(){
 use support::session::Session;use std::time::Duration;
 for(before,typed,after,expected)in[
  ("import {deepStrictEqual} from 'std:testing';function main(){","deepStrictEqual","([[1]],[[1]]);}","void"),
  ("import {notDeepStrictEqual} from 'std:testing';function main(){","notDeepStrictEqual","([1],[2]);}","void"),
  ("import {rejects} from 'std:testing';async function main():Task<void>{await ","rejects","(async()=>{throw {code:'expected'};});}","Task<void>"),
  ("import {rejects as expectRejection} from 'std:testing';async function main():Task<void>{await ","expectRejection","(async()=>{throw 'expected';});}","Task<void>"),
 ]{
  let mut baseline=None;let mut signature_baseline=None;
  for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
   let after=if broken{format!("{after}function broken(){{const = ;}}") }else{after.to_owned()};let source=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
   if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&source);session.settle();}
   let response=session.completion_items("main.tsr",&source,before.len()+typed.len());let items=if response.is_array(){response}else{response["items"].clone()};
   let item=items.as_array().unwrap().iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("Missing {typed} [{mode}]: {items}"));
   let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&source,before.len()+1).unwrap();assert!(format!("{detail}\n{hover}").contains("Ownership:"),"{typed} [{mode}]: {hover}");
   if let Some((old_detail,old_hover))=&baseline{assert_eq!((&detail,&hover),(old_detail,old_hover),"{typed} [{mode}]");}else{baseline=Some((detail,hover));}
   let help=session.request("textDocument/signatureHelp",json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));assert!(help.to_string().contains(expected),"{typed} [{mode}]: {help}");
   if let Some(previous)=&signature_baseline{assert_eq!(&help,previous,"{typed} [{mode}]");}else{signature_baseline=Some(help);}
   let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{typed} [{mode}]: {counters}");}
  }
 }
}
