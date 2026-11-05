mod support;
use support::session::Session;
use std::time::Duration;
#[test]
fn io_explicit_result_signature_and_docs_agree_in_all_server_phases(){
 for(before,typed,after,expected)in[
 ("import {reader,readInto,Reader,IoError} from 'std:io';import {Bytes} from 'std:bytes';async function inspect(input:Mutable<Reader>,destination:Mutable<Bytes>):Task<void>{const pending=","readInto","(input,destination);const outcome=await pending;}function main(){}",&["Task<Result<int","IoError","destination"][..]),
 ("import {readInto as fill,Reader as Input,IoError} from 'std:io';import {Bytes as Buffer} from 'std:bytes';async function inspect(input:Mutable<Input>,destination:Mutable<Buffer>):Task<void>{const pending=","fill","(input,destination);const outcome=await pending;}function main(){}",&["Task<Result<int","IoError","destination"][..]),
 ("import {readInto,Reader,IoError} from 'std:io';import {Bytes} from 'std:bytes';async function inspect(input:Mutable<Reader>,destination:Mutable<Bytes>):Task<void>{const fill=readInto;const pending=","fill","(input,destination);const outcome=await pending;}function main(){}",&["Task<Result<int","IoError"][..]),
 ("import {IoError as Failure} from 'std:io';function inspect(error:ReadOnly<Failure>){console.log(error.","code",");}function main(){}",&["string","Failure category"][..]),
 ]{
 let mut baseline=None;
 for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
  let after=if broken{format!("{after}function broken(){{const = ;}}") }else{after.to_owned()};let source=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
  if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&source);session.settle();}
  let response=session.completion_items("main.tsr",&source,before.len()+typed.len());let items=if response.is_array(){response}else{response["items"].clone()};let items=items.as_array().unwrap();assert!(!items.is_empty(),"{mode}");let item=items.iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("missing IO operation [{mode}]: {items:?}"));let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&source,before.len()+1).unwrap_or_default();let text=format!("{detail}\n{hover}");assert!(text.contains("Ownership:")&&expected.iter().all(|part|text.contains(*part)),"{typed}/{mode}: {text}");if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{mode}");}else{baseline=Some((detail,hover));}
 }
 }
}
#[test]
fn io_module_exports_all_documented_operations_and_private_pollers_stay_hidden(){
 let source="import { } from 'std:io';function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let names=session.completion("main.tsr",source,"import { ".len()).unwrap();assert!(!names.is_empty());let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();for entry in catalog["std:io"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(names.iter().any(|item|item==name),"missing {name}: {names:?}");}assert!(!names.iter().any(|name|name.starts_with("__")));
}

#[test]
fn io_task_argument_loans_are_reported_and_clear_after_await(){
 for call in ["readInto(input,bytes)","fill(input,bytes)","(true?fill:fill)(input,bytes)"]{
  let prefix="import {reader,readInto,close} from 'std:io';import {fromUtf8,alloc,set} from 'std:bytes';async function main():Task<void>{const input=reader(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));const fill=readInto;";
  let invalid=tsrust_compiler::format(&format!("{prefix}const task={call};unwrap(set(bytes,0,1));await task;}}")).unwrap();
  assert_eq!(tsrust_compiler::format(&invalid).unwrap(),invalid);
  let mut session=Session::start(&[]);session.open("main.tsr",&invalid);
  let diagnostics=session.wait_for_diagnostics("main.tsr");
  assert!(diagnostics.as_array().unwrap().iter().any(|diagnostic|diagnostic["code"]=="capture-conflict"),"{call}: {diagnostics}");
  let valid=tsrust_compiler::format(&format!("{prefix}const task={call};await task;unwrap(set(bytes,0,1));unwrap(await close(input));}}")).unwrap();
  assert_eq!(tsrust_compiler::format(&valid).unwrap(),valid);
  session.change("main.tsr",&valid);let diagnostics=session.wait_for_diagnostics("main.tsr");
  assert!(diagnostics.as_array().unwrap().is_empty(),"{call}: {diagnostics}");
 }
}
