mod support;
fn check(source:&str)->std::process::Output{let dir=support::TempDirectory::new();support::run(support::cli().arg("check").arg(dir.source(source)),30)}
#[test]
fn cache_moves_evicted_owners_and_releases_retained_resources(){
 let dir=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/cache/owned-lru.tsr"));
 let result=support::run(support::cli().arg("run").arg(dir.source(&source)),180);support::success(&result);assert_eq!(result.stdout,b"cache ownership accepted\n");
}
#[test]
fn cache_reads_reject_overlapping_owner_access(){
 for operation in ["cache.size();","cache.peek(1);","cache.get(1);","cache.clear();","unwrap(cache.set(2,'two'));","cache.remove(1);","observe(cache);","const moved=cache;"]{
  let source=format!("import {{LruCache}} from 'std:cache';function observe(cache:ReadOnly<LruCache<int,string>>){{}}function main(){{const cache=new LruCache<int,string>(2);unwrap(cache.set(1,'one'));const value=cache.get(1);{operation}console.log(value!);}}");
  let result=check(&source);assert!(!result.status.success(),"accepted overlap: {operation}");
  let error=String::from_utf8_lossy(&result.stderr);assert!(error.contains("error[native-view-conflict]"),"{operation}: {error}");
 }
}
#[test]
fn cache_rejects_unsupported_keys_clone_and_readonly_mutation(){
 for body in ["const cache=new LruCache<int|string,int>(1);","const cache=new LruCache<int,string>(1);const copied=clone(cache);","const cache=new LruCache<int,string>(1);const value:ReadOnly<LruCache<int,string>>=cache;value.size();"]{
  let source=format!("import {{LruCache}} from 'std:cache';function main(){{{body}}}");
  assert!(!check(&source).status.success(),"accepted {body}");
 }
}
#[test]
fn cache_documentation_examples_execute_together(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
 for family in ["std:cache","LruCache","CacheEntry","CacheInsertError"] {for entry in catalog[family].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
  let (_,body)=example["source"].as_str().unwrap().split_once("function main()").unwrap();functions.push_str(&format!("function example{count}(){body}"));calls.push_str(&format!("example{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;
 }}}
 assert_eq!(count,14);let source=support::format_round_trip(&format!("import {{LruCache}} from 'std:cache';{functions}function main(){{{calls}}}"));let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(dir.source(&source)),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}

#[test]
fn cache_projection_cannot_escape_or_mutate_borrowed_values(){
 for body in [
  "const c=new LruCache<int,string>(1);unwrap(c.set(1,'x'));const v=c.get(1)!;c.clear();console.log(v);",
  "const c=new LruCache<int,Row>(1);unwrap(c.set(1,{value:1}));const v=c.get(1)!;v.value=2;",
  "const c=new LruCache<int,Row>(1);unwrap(c.set(1,{value:1}));const v=c.get(1)!;const n=v.value;c.clear();console.log(v.value);",
 ]{
  let source=format!("import {{LruCache}} from 'std:cache';type Row={{value:int;}};function main(){{{body}}}");assert!(!check(&source).status.success(),"accepted {body}");
 }
 let source="import {LruCache} from 'std:cache';function leak():string{const c=new LruCache<int,string>(1);unwrap(c.set(1,'x'));return c.get(1)!;}function main(){console.log(leak());}";
 assert!(!check(source).status.success(),"local cache borrow escaped");
 for declaration in ["LruCache<int,ByteView>","LruCache<int,Holder>","LruCache<int,Function<void>>"]{
  let source=format!("import {{LruCache}} from 'std:cache';import {{ByteView}} from 'std:bytes';type Holder={{view:ByteView;}};function invalid(cache:ReadOnly<{declaration}>){{}}function main(){{}}");let result=check(&source);assert!(!result.status.success());assert!(String::from_utf8_lossy(&result.stderr).contains(if declaration.contains("Function"){"error[unsupported]"}else{"error[cache-payload]"}),"{}",String::from_utf8_lossy(&result.stderr));
 }
}
#[test]
fn cache_capacity_and_owned_failure_have_checked_runtime_reports(){
 for (body,message) in [
  ("const c=new LruCache<int,int>(0);","Uncaught RangeError:"),
  ("const c=new LruCache<int,TrackedResource>(1);unwrap(c.set(1,trackedResource(),-1));","Uncaught CacheInsertError: ttl"),
 ]{
  let source=support::format_round_trip(&format!("import {{LruCache}} from 'std:cache';import {{TrackedResource,trackedResource}} from 'std:testing';function main(){{{body}}}"));let dir=support::TempDirectory::new();let result=support::run(support::cli().arg("run").arg(dir.source(&source)),180);assert_eq!(result.status.code(),Some(1));let stderr=String::from_utf8_lossy(&result.stderr);assert!(stderr.contains(message),"{stderr}");assert!(!stderr.contains("compiler defect"),"{stderr}");
 }
}

#[test]
fn cache_last_use_keeps_aliases_and_loop_backedges_live(){
 for tail in [
  "const alias=value;cache.clear();console.log(alias);",
  "while(true){console.log(value);cache.clear();}",
  "try{console.log(value);cache.clear();}finally{console.log(value);}",
 ]{
  let source=format!("import {{LruCache}} from 'std:cache';function main(){{const cache=new LruCache<int,string>(1);unwrap(cache.set(1,'x'));const value=cache.get(1);{tail}}}");
  let result=check(&source);assert!(!result.status.success(),"accepted {tail}");let error=String::from_utf8_lossy(&result.stderr);assert!(error.contains("error[native-view-conflict]"),"{tail}: {error}");
 }
}

#[test]
fn cache_serializes_owned_loads_and_retries_failures(){
 let dir=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/cache/serialized-loading.tsr"));
 let result=support::run(support::cli().arg("run").arg(dir.source(&source)),180);support::success(&result);assert_eq!(result.stdout,b"serialized cache loading accepted\n");
}

#[test]
fn cache_entry_cloning_checks_its_owned_payload(){
 let source="import {LruCache} from 'std:cache';import {trackedResource,TrackedResource} from 'std:testing';function main(){const cache=new LruCache<int,TrackedResource>(1);unwrap(cache.set(1,trackedResource()));const entry=cache.remove(1)!;const copy=clone(entry);}";
 assert!(!check(source).status.success(),"cloned a non-clone resource through CacheEntry");
}
