mod support;
use serde_json::json;
#[test]
fn shared_cache_aliases_expose_owned_reads_and_explicit_sharing(){
 use support::session::Session;
 let text="import {SharedLruCache as Cache} from 'std:cache';function main(){const cache=new Cache<int,string>(2);unwrap(cache.set(1,'one'));const other=cache.share();const value=other.getCloned(1);cache.clear();if(value!==undefined){console.log(value);}}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();let offset=text.find("other.getCloned").unwrap()+7;
 let hover=session.hover("main.tsr",text,offset).unwrap_or_default();assert!(hover.contains("string")&&hover.contains("owned")&&hover.contains("undefined"),"{hover}");
 let completion=session.completion("main.tsr",text,text.find("cache.share").unwrap()+6).unwrap();assert!(completion.iter().any(|value|value=="share"),"{completion:?}");
 for fragment in ["Cache<int,string>(","cache.set(","other.getCloned("]{let position=text.find(fragment).unwrap()+fragment.len();let help=session.request("textDocument/signatureHelp",json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":position}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{fragment}: {help}");}
}
