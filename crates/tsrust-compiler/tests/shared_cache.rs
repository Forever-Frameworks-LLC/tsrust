mod support;
#[test]
fn shared_cache_owns_copies_and_serves_concurrent_handlers(){
 let source=support::format_round_trip(include_str!("../../../examples/cache/shared-http.tsr"));let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(dir.source(&source)),240);support::success(&output);assert_eq!(output.stdout,b"shared HTTP cache accepted\n");
}
#[test]
fn shared_cache_rejects_implicit_clone_resources_endpoints_and_borrowed_reads(){
 for(body,code)in [
 ("const c=new SharedLruCache<int,string>(1);clone(c);","clone-type"),
 ("const c=new SharedLruCache<int,TrackedResource>(1);","shared-cache-payload"),
 ("const c=new SharedLruCache<int,Sender<int>>(1);","shared-cache-payload"),
 ("const c=new SharedLruCache<int,SharedLruCache<int,int>>(1);","shared-cache-payload"),
 ("const c=new SharedLruCache<int,string>(1);c.get(1);","cache-member"),
 ("const c=new SharedLruCache<int|string,string>(1);","cache-key"),
 ]{let source=format!("import {{SharedLruCache}} from 'std:cache';import {{TrackedResource}} from 'std:testing';import {{Sender}} from 'std:futures';function main(){{{body}}}");let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(dir.source(&support::format_round_trip(&source))),30);assert!(!output.status.success(),"{source}");assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),"{}",String::from_utf8_lossy(&output.stderr));}
}
#[test]
fn fallible_native_constructors_check_catch_bodies(){
 for name in ["LruCache","SharedLruCache"]{
  let source=format!("import {{{name}}} from 'std:cache';function main(){{try{{const cache=new {name}<int,int>(0);}}catch(error){{missingCatchFunction();}}}}");let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(dir.source(&support::format_round_trip(&source))),30);assert!(!output.status.success(),"unchecked catch: {source}");assert!(String::from_utf8_lossy(&output.stderr).contains("missingCatchFunction"),"{}",String::from_utf8_lossy(&output.stderr));
 }
}
