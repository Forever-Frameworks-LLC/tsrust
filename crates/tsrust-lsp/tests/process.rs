mod support;
use support::session::Session;
use std::time::Duration;
#[test]
fn process_explicit_result_signature_and_docs_agree_in_all_server_phases(){
 for(before,typed,after,expected)in[
 ("import {run,Output,ProcessError} from 'std:process';async function main():Task<void>{const pending=","run","('missing',[]);const result=await pending;}",&["Task<Result<Output","ProcessError","options"][..]),
 ("import {run as launch} from 'std:process';async function main():Task<void>{const pending=","launch","('missing',[]);await pending;}",&["Task<Result<Output","ProcessError"][..]),
 ("import {Child} from 'std:process';async function work(child:Mutable<Child>):Task<void>{const pending=child.","wait","();await pending;}function main(){}",&["Task<Result<ExitStatus","ProcessError"][..]),
 ("import {ProcessError as Failure} from 'std:process';function inspect(error:ReadOnly<Failure>){console.log(error.","code",");}function main(){}",&["string","Failure category"][..]),
 ]{
 let mut baseline=None;
 for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
  let after=if broken{format!("{after}function broken(){{const = ;}}") }else{after.to_owned()};let source=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
  if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&source);session.settle();}
  let response=session.completion_items("main.tsr",&source,before.len()+typed.len());let items=if response.is_array(){response}else{response["items"].clone()};let items=items.as_array().unwrap();assert!(!items.is_empty(),"{mode}");let item=items.iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("missing process operation [{mode}]: {items:?}"));let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&source,before.len()+1).unwrap_or_default();let text=format!("{detail}\n{hover}");assert!(text.contains("Ownership:")&&expected.iter().all(|part|text.contains(*part)),"{typed}/{mode}: {text}");if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{mode}");}else{baseline=Some((detail,hover));}
 }
 }
}
#[test]
fn process_module_exports_all_documented_operations_and_private_pollers_stay_hidden(){
 let source="import { } from 'std:process';function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let names=session.completion("main.tsr",source,"import { ".len()).unwrap();assert!(!names.is_empty());let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();for entry in catalog["std:process"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(names.iter().any(|item|item==name),"missing {name}: {names:?}");}assert!(!names.iter().any(|name|name.starts_with("__")));
}

#[test]
fn process_child_completion_exports_every_documented_method(){
 let source="import {Child} from 'std:process';function inspect(child:Mutable<Child>){child.}function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let names=session.completion("main.tsr",source,source.find("child.}").unwrap()+6).unwrap();let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();for entry in catalog["Child"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(names.iter().any(|item|item==name),"missing {name}: {names:?}");}
}
