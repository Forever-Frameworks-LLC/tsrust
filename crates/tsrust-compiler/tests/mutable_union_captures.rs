mod support;
use std::fs;

#[test]
fn owned_callbacks_reborrow_captured_unions_into_mutable_parameters(){
 let source=r#"import {strictEqual,trackedResource,resourceId,releaseCount,TrackedResource} from 'std:testing';
interface Counter{count:int;}
type Value=Counter|string|undefined;
function advance(value:Mutable<Value>):int{if(typeof value==='object'){value.count++;return value.count;}return 0;}
function counter(value:Value):Owned<Function<int>>{return ()=>advance(value);}
interface ResourceReader{read:Owned<Function<int>>;}
type Reader=ResourceReader|string|undefined;
function read(value:Mutable<Reader>):int{if(typeof value==='object'){return value.read();}return 0;}
function reader(resource:TrackedResource):Owned<Function<int>>{const value:Reader={read:()=>resourceId(resource)};return ()=>read(value);}
function main(){const next=counter({count:0});strictEqual(next(),1);strictEqual(next(),2);const absent=counter(undefined);strictEqual(absent(),0);const text=counter('Text');strictEqual(text(),0);const resource=trackedResource();const id=resourceId(resource);{const callback=reader(resource);strictEqual(callback(),id);strictEqual(callback(),id);strictEqual(releaseCount(id),0);}strictEqual(releaseCount(id),1);console.log('Mutable union captures passed');}"#;
 let directory=support::TempDirectory::new();let path=directory.0.join("main.tsr");fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("run").arg(path),120);support::success(&output);assert_eq!(output.stdout,b"Mutable union captures passed\n");
}
