mod support;
use support::session::Session;
use std::time::Duration;
#[test]
fn network_address_dns_signature_and_docs_agree_in_all_server_phases(){
 for(before,typed,after,expected)in[
 ("import {parseIp} from 'std:net';function main(){const value=","parseIp","('::1');}",&["Result<IpAddress","NetError","text"][..]),
 ("import {lookup} from 'std:dns';async function main():Task<void>{const pending=","lookup","('localhost','ipv4');unwrap(await pending);}",&["Task<Result<IpAddress[]","NetError","family"][..]),
 ("import {lookup as resolve} from 'std:dns';async function main():Task<void>{const pending=","resolve","('::1');unwrap(await pending);}",&["Task<Result<IpAddress[]","NetError"][..]),
 ("import {NetError as Failure} from 'std:net';function inspect(error:ReadOnly<Failure>){console.log(error.","code",");}function main(){}",&["string","Failure category"][..]),
 ]{
 let mut baseline=None;
 for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
  let after=if broken{format!("{after}function broken(){{const = ;}}") }else{after.to_owned()};let source=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
  if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&source);session.settle();}
  let response=session.completion_items("main.tsr",&source,before.len()+typed.len());let items=if response.is_array(){response}else{response["items"].clone()};let items=items.as_array().unwrap();assert!(!items.is_empty(),"{mode}");let item=items.iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("missing network operation [{mode}]: {items:?}"));let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&source,before.len()+1).unwrap_or_default();let text=format!("{detail}\n{hover}");assert!(text.contains("Ownership:")&&expected.iter().all(|part|text.contains(*part)),"{typed}/{mode}: {text}");if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{mode}");}else{baseline=Some((detail,hover));}
 }
 }
}
#[test]
fn dns_module_exports_are_documented_and_private_pollers_hidden(){
 let source="import { } from 'std:dns';function main(){}";let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();let names=session.completion("main.tsr",source,"import { ".len()).unwrap();assert!(!names.is_empty());let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();for entry in catalog["std:dns"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(names.iter().any(|item|item==name),"missing {name}: {names:?}");}assert!(!names.iter().any(|name|name.starts_with("__")));
}
