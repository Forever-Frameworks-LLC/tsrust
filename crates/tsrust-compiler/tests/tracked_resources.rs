mod support;
#[test]
fn tracked_resource_storage_capture_return_and_close(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/resource-lifetimes/release.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);
 support::success(&output);assert_eq!(output.stdout,b"resources released once\n");
}
#[test]
fn tracked_resource_observer_negative_controls(){
 for body in [
  "const r=trackedResource();const key=resourceId(r);strictEqual(releaseCount(key),1);",
  "const r=trackedResource();const key=resourceId(r);closeResource(r);strictEqual(releaseCount(key),0);",
  "const r=trackedResource();const key=resourceId(r);closeResource(r);strictEqual(releaseCount(key),2);",
  "const a=trackedResource();const b=trackedResource();const x=resourceId(a);const y=resourceId(b);closeResource(a);closeResource(b);ok(releaseOrder(y)<releaseOrder(x));",
 ] {
  let directory=support::TempDirectory::new();
  let source=support::format_round_trip(&format!("import {{trackedResource,resourceId,closeResource,releaseCount,releaseOrder}} from 'std:testing';import {{strictEqual,ok}} from 'std:testing';function main(){{{body}}}"));
  let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);
  assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("AssertionError"),"{}",String::from_utf8_lossy(&output.stderr));
 }
}
#[test]
fn tracked_resources_reject_projection_clone_transfer_and_reuse(){
 for (body,code) in [
  ("const r=trackedResource();closeResource(r);resourceId(r);","transferred"),
  ("const r=trackedResource();const copy=clone(r);","clone-type"),
  ("const r=trackedResource();console.log(r.storage);","type"),
  ("const r=trackedResource();const worker=unwrap(spawn(():int=>resourceId(r)));worker.join();","thread-transfer"),
  ("const inbox=new Channel<TrackedResource>(1);","channel-transfer"),
 ] {
  let directory=support::TempDirectory::new();
  let source=format!("import {{trackedResource,resourceId,closeResource,TrackedResource}} from 'std:testing';import {{spawn,Channel}} from 'std:threads';function main(){{{body}}}");
  let path=directory.source(&source);
  for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"accepted {body}");let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains(&format!("error[{code}]")),"{error}");}
 }

}

#[test]
fn tracked_resource_local_async_ownership_survives_suspension(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/resource-lifetimes/async.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);
 support::success(&output);assert_eq!(output.stdout,b"local async release settled\n");
}

#[test]
fn tracked_resources_error_abandonment_and_repeated_balance(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(r#"import {trackedResource,resourceId,closeResource,releaseCount,liveCount,TrackedResource,strictEqual} from 'std:testing';
function fail(resource:Owned<TrackedResource>):void {throw 'failure';}
async function later(resource:Owned<TrackedResource>):Task<void>{closeResource(resource);}
function main(){
 const baseline=liveCount();
 const r=trackedResource();const key=resourceId(r);
 try {fail(r);} catch(error){strictEqual(error,'failure');}
 strictEqual(releaseCount(key),1);
 let abandoned=0;
 {const r=trackedResource();abandoned=resourceId(r);const pending=later(r);strictEqual(releaseCount(abandoned),0);}
 strictEqual(releaseCount(abandoned),1);
 for(let i=0;i<1000;i=i+1){const r=trackedResource();const key=resourceId(r);closeResource(r);strictEqual(releaseCount(key),1);}
 strictEqual(liveCount(),baseline);
 console.log('errors and abandoned tasks release');
}
"#);
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);
 support::success(&output);assert_eq!(output.stdout,b"errors and abandoned tasks release\n");
 assert!(String::from_utf8_lossy(&output.stderr).contains("unused-task"));
}

#[test]
fn resources_reject_started_task_transport_and_private_runtime_imports(){
 for (source,code) in [
 ("import {trackedResource,resourceId,TrackedResource} from 'std:testing';async function inspect(r:Owned<TrackedResource>):Task<int>{return resourceId(r);}async function main():Task<void>{const r=trackedResource();const job=inspect(r).start();await job.join();}","task-transfer"),
 ("import {trackedResource} from 'std:testing-runtime';function main(){trackedResource();}","module-native"),
 ] {
  let directory=support::TempDirectory::new();let path=directory.source(source);
  for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")));}
 }
}
