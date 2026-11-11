//! Entry diagnostics borrow errors without expanding opaque retained values.
mod support;

#[test]
fn unhandled_opaque_payloads_and_typed_wrappers_report_without_inspection(){
 for(source,expected)in [
  ("function main(){throw {code:'visible',count:7};}","Uncaught { code: 'visible', count: 7 }\n"),
  ("import {fromUtf8} from 'std:bytes';function main(){throw unwrap(fromUtf8('private-bytes'));}","Uncaught <value not expanded>\n"),
  ("import {fromUtf8} from 'std:bytes';function main(){throw {value:unwrap(fromUtf8('private-bytes'))};}","Uncaught <value not expanded>\n"),
  ("import {fromUtf8} from 'std:bytes';function main(){throw [unwrap(fromUtf8('private-bytes'))];}","Uncaught <value not expanded>\n"),
  ("import {trackedResource} from 'std:testing';function main(){throw trackedResource();}","Uncaught <value not expanded>\n"),
  ("import {SendError} from 'std:futures';import {Bytes,fromUtf8} from 'std:bytes';function main(){const error:SendError<Bytes>={code:'closed',value:unwrap(fromUtf8('private-bytes'))};throw error;}","Uncaught SendError: closed\n"),
  ("import {ThreadError} from 'std:threads';import {Bytes,fromUtf8} from 'std:bytes';function main(){const error:ThreadError<Bytes>={code:'worker',error:unwrap(fromUtf8('private-bytes'))};throw error;}","Uncaught ThreadError: worker\n"),
  ("import {Channel,Sender} from 'std:futures';import {Bytes,fromUtf8} from 'std:bytes';async function sendOwned(sender:Sender<Bytes>):Task<void>{await sender.send(unwrap(fromUtf8('private-bytes')));}async function main():Task<void>{const queue=new Channel<Bytes>(1);const sender=queue.sender();queue.close();await sendOwned(sender).start();}","Uncaught AsyncError: worker\n"),
 ]{
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
  let result=support::run(support::cli().arg("run").arg(path),180);assert!(!result.status.success());
  assert!(result.stdout.is_empty(),"{}",String::from_utf8_lossy(&result.stdout));
  assert_eq!(String::from_utf8_lossy(&result.stderr),expected);
 }
}

#[test]
fn entry_fallback_does_not_authorize_explicit_opaque_inspection(){
 for source in [
  "import {fromUtf8} from 'std:bytes';function main(){console.log(unwrap(fromUtf8('private-bytes')));}",
  "import {SendError} from 'std:futures';import {Bytes,fromUtf8} from 'std:bytes';function main(){const error:SendError<Bytes>={code:'closed',value:unwrap(fromUtf8('private-bytes'))};console.log(error);}",
 ]{
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
  let result=support::run(support::cli().arg("check").arg(path),30);assert!(!result.status.success());
  assert!(String::from_utf8_lossy(&result.stderr).contains("error[native-capability]"),"{}",String::from_utf8_lossy(&result.stderr));
 }
}

#[test]
fn stream_channel_catalog_example_reports_only_supported_entry_payloads(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let entry=catalog["std:http"].as_array().unwrap().iter().find(|entry|entry["name"]=="streamChannel").unwrap();
 for example in entry["examples"].as_array().unwrap(){
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
  let result=support::run(support::cli().arg("run").arg(path),180);support::success(&result);
  assert_eq!(String::from_utf8_lossy(&result.stdout),example["output"].as_str().unwrap());
  assert!(result.stderr.is_empty(),"{}",String::from_utf8_lossy(&result.stderr));
 }
}

#[test]
fn implicit_test_callback_reports_do_not_inspect_owned_byte_payloads(){
 for callback in ["()=>{throw unwrap(fromUtf8('private-bytes'));}","async()=>{throw unwrap(fromUtf8('private-bytes'));}"]{
  let source=format!("import {{test,runTests}} from 'std:testing';import {{fromUtf8}} from 'std:bytes';function main(){{test('opaque',{callback});console.log(runTests());}}");
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
  let result=support::run(support::cli().arg("run").arg(path),180);support::success(&result);
  let text=String::from_utf8_lossy(&result.stdout);assert!(text.contains("<value not expanded>"),"{text}");
  assert!(!text.contains("private-bytes"),"{text}");assert!(result.stderr.is_empty(),"{}",String::from_utf8_lossy(&result.stderr));
 }
}

#[test]
fn inspectable_callback_cache_error_retains_full_field_formatting(){
 let source="import {test,runTests} from 'std:testing';import {LruCache} from 'std:cache';function main(){test('visible',()=>{const cache=new LruCache<int,int>(1);unwrap(cache.set(1,7,-1));});console.log(runTests());}";
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 let result=support::run(support::cli().arg("run").arg(path),180);support::success(&result);
 let text=String::from_utf8_lossy(&result.stdout);for field in ["code: 'ttl'","key: 1","value: 7"]{assert!(text.contains(field),"{field}: {text}");}
 assert!(result.stderr.is_empty(),"{}",String::from_utf8_lossy(&result.stderr));
}
