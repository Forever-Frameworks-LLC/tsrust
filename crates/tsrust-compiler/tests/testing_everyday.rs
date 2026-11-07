mod support;
fn native(source:&str){let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert!(output.stdout.is_empty(),"{}",String::from_utf8_lossy(&output.stdout));assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));}
#[test]fn structural_exact_data_borrowing_and_recursive_records_execute(){native(r#"
import {deepStrictEqual,notDeepStrictEqual,strictEqual} from 'std:testing';
type Row={label:string;values:number[];key:int;};type Node={value:int;children:Node[];};
function main(){
 const left:Row={label:'é🦀',values:[Number.NaN,-0.0,1.0],key:9007199254740993};const right:Row={key:9007199254740993,values:[Number.NaN,-0.0,1.0],label:'é🦀'};
 deepStrictEqual(left,right);deepStrictEqual(left,right);strictEqual(left.key,9007199254740993);strictEqual(right.label,'é🦀');
 notDeepStrictEqual([0.0],[-0.0]);notDeepStrictEqual([9007199254740992],[9007199254740993]);deepStrictEqual([[true,false]],[[true,false]]);
 const empty:undefined[]=[],other:undefined[]=[];deepStrictEqual(empty,other);const tuple:[int,string]=[7,'ready'];deepStrictEqual(tuple,clone(tuple));
 const a:int|string=7;const b:int|string='7';notDeepStrictEqual(a,b);deepStrictEqual(a,a);
 const first=new Map<string,int[]>();first.set('a',[1,2]);first.set('b',[3]);const second=new Map<string,int[]>();second.set('b',[3]);second.set('a',[1,2]);deepStrictEqual(first,second);strictEqual(first.size,2);
 const setA=new Set<int>([1,2,3]);const setB=new Set<int>([3,2,1]);deepStrictEqual(setA,setB);notDeepStrictEqual(setA,new Set<int>([1,2]));
 let tree:Node={value:0,children:[]};let i=0;while(i<10000){tree={value:i,children:[tree]};i++;}const copied=clone(tree);deepStrictEqual(tree,copied);strictEqual(tree.value,9999);
}
"#);}
#[test]fn rejects_checked_payloads_and_resource_cleanup_execute(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {rejects,trackedResource,resourceId,releaseCount} from 'std:testing';import {delay} from 'std:futures';import {compile} from 'std:regex';
async function main():Task<void>{
 await rejects(async()=>{unwrap(compile('('));});
 await rejects(async()=>{let id=0;try{const resource=trackedResource();id=resourceId(resource);await delay(0);throw {code:'expected',retry:false};}finally{console.log(releaseCount(id));}});
 await rejects(async()=>{const failure:int|string='typed';throw failure;});
}
"#);let path=directory.source(&source);let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);
 // The host checks cleanup independently: rejects must not hide a failed cleanup assertion.
 assert_eq!(output.stdout,b"1\n");assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
}
#[test]fn structural_assertions_reject_unsupported_nested_values_and_report_mismatch(){
 let directory=support::TempDirectory::new();for(body,code)in[("const callback=():int=>7;deepStrictEqual({callback},{callback});","unsupported"),("const value=trackedResource();deepStrictEqual([value],[trackedResource()]);","native-capability")]{let source=format!("import {{deepStrictEqual,trackedResource}} from 'std:testing';function main(){{{body}}}");let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));}
 let path=directory.source("import {deepStrictEqual} from 'std:testing';function main(){deepStrictEqual([1,2],[1,3],'nested mismatch');}");let output=support::run(support::cli().arg("run").arg(path),180);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("nested mismatch"));
}
#[test]fn cli_literal_filter_json_console_and_summary_are_machine_readable(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {describe,test,beforeAll,afterAll} from 'std:testing';
describe('selected',()=>{beforeAll(()=>{console.log('setup');});afterAll(()=>{console.log('teardown');});test('literal[.]',()=>{console.error('é\n\t"quoted"');});test('other',()=>{throw 'must be filtered';});});describe('excluded',()=>{beforeAll(()=>{throw 'must not run';});test('other',()=>{throw 'must not run';});});
"#);let path=directory.source(&source);let output=support::run(support::cli().arg("test").arg(&path).arg("--filter").arg("selected > literal[.]").arg("--reporter=json"),180);support::success(&output);let rows=std::str::from_utf8(&output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),1);assert_eq!(tests[0]["status"],"passed");let summary=rows.iter().find(|row|row["event"]=="summary").unwrap_or_else(||panic!("Missing test summary; stderr: {}",String::from_utf8_lossy(&output.stderr)));assert_eq!(summary["passed"],1);assert_eq!(summary["skipped"],2);assert!(rows.iter().any(|row|row["event"]=="output"&&row["stream"]=="stderr"&&row["text"]=="é\n\t\"quoted\"\n"));assert!(output.stderr.is_empty());
 let report=directory.0.join("actual-reporter.ndjson");std::fs::write(&report,&output.stdout).unwrap();
 let oracle=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/runtime/testing-everyday-reporter-oracle.mjs");
 support::success(&support::run(std::process::Command::new("node").arg(oracle).arg(report).arg(&path),30));
}
#[test]fn explicit_async_deadline_runs_teardown_and_later_tests(){
 let directory=support::TempDirectory::new();let path=directory.source(r#"
import {test,afterEach,afterAll,strictEqual} from 'std:testing';import {delay} from 'std:futures';let cleaned=0;afterEach(async()=>{await delay(0);cleaned++;console.log('afterEach');});afterAll(()=>{strictEqual(cleaned,2);console.log('afterAll');});test('deadline',async()=>{await delay(60000);throw 'late';});test('continues',async()=>{strictEqual(cleaned,1);});
"#);let output=support::run(support::cli().arg("test").arg(path).arg("--timeout=20").arg("--reporter=json"),180);assert!(!output.status.success());let rows=String::from_utf8(output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();let summary=rows.iter().find(|row|row["event"]=="summary").unwrap_or_else(||panic!("Missing test summary; stderr: {}",String::from_utf8_lossy(&output.stderr)));assert_eq!(summary["passed"],1);assert_eq!(summary["failed"],1);assert_eq!(summary["hookFailures"],0);assert!(rows.iter().any(|row|row["event"]=="test"&&row["name"]=="deadline"&&row["messages"].to_string().contains("TestTimeout")));assert!(rows.iter().any(|row|row["event"]=="output"&&row["text"]=="afterAll\n"));
}
#[test]fn cli_test_option_errors_precede_compilation(){let directory=support::TempDirectory::new();let path=directory.source("function main(){}");for option in["--timeout=-1","--timeout=18446744073709551616","--reporter=xml"]{let output=support::run(support::cli().arg("test").arg(&path).arg(option),30);assert!(!output.status.success());assert!(!output.stderr.is_empty());}}
#[test]fn timeout_drains_started_child_owner_before_teardown(){
 let directory=support::TempDirectory::new();let path=directory.source(r#"
import {test,afterEach,strictEqual} from 'std:testing';import {Channel,Sender,delay} from 'std:futures';
let sender:Sender<int>|undefined=undefined;
async function hold(queue:Owned<Channel<int>>):Task<void>{await queue.receive();}
afterEach(async()=>{if(sender!==undefined){let closed=false;try{await sender.send(7);}catch(error){strictEqual(error.code,'closed');closed=true;}strictEqual(closed,true);sender=undefined;console.log('child owner released');}});
test('timeout with child',async()=>{const queue=new Channel<int>(1);sender=queue.sender();const job=hold(queue).start();try{await delay(60000);}finally{job.cancel();await job;}});
test('next body',async()=>{strictEqual(sender===undefined,true);});
"#);let output=support::run(support::cli().arg("test").arg(path).arg("--timeout=20").arg("--reporter=json"),180);assert!(!output.status.success());let rows=String::from_utf8(output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();let summary=rows.iter().find(|row|row["event"]=="summary").unwrap_or_else(||panic!("Missing test summary; stderr: {}",String::from_utf8_lossy(&output.stderr)));assert_eq!(summary["passed"],1);assert_eq!(summary["failed"],1);assert_eq!(summary["hookFailures"],0);assert!(rows.iter().any(|row|row["event"]=="output"&&row["text"]=="child owner released\n"));
}
#[test]fn everyday_catalog_examples_execute(){let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();for name in["deepStrictEqual","notDeepStrictEqual","rejects"]{let entry=catalog["std:testing"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();for example in entry["examples"].as_array().unwrap(){let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes());}}}
#[test]fn transparent_native_snapshots_compare_without_expanding_owners(){native(r#"
import {compile} from 'std:regex';import {deepStrictEqual} from 'std:testing';
function main(){const regex=unwrap(compile('(é)'));const snapshots=regex.findAll('é é');deepStrictEqual(snapshots,clone(snapshots));}
"#);}
#[test]fn rejects_does_not_satisfy_itself_or_accept_native_panics(){
 let directory=support::TempDirectory::new();std::fs::write(directory.0.join("panic.rs"),"pub fn native_panic()->i64{panic!(\"native panic sentinel\")}").unwrap();
 for(body,message)in[("await rejects(async()=>{});","Expected task to reject"),("await rejects(async()=>{native_panic();});","native panic sentinel")]{let source=format!("import {{rejects}} from 'std:testing';import {{native_panic}} from './panic.rs';async function main():Task<void>{{{body}}}");let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("run").arg(path),180);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(message),"{}",String::from_utf8_lossy(&output.stderr));}
 native("import {rejects,fail} from 'std:testing';async function main():Task<void>{await rejects(async()=>{fail('checked assertion payload');});}");
}

#[test]fn cancelling_rejects_remains_cancelled(){native(r#"
import {rejects,strictEqual} from 'std:testing';import {delay} from 'std:futures';
async function waitForFailure():Task<void>{await rejects(async()=>{await delay(60000);throw 'eventual';});}
async function main():Task<void>{const job=waitForFailure().start();job.cancel();let cancelled=false;try{await job;}catch(error){strictEqual(error.code,'cancelled');cancelled=true;}strictEqual(cancelled,true);}
"#);}

#[test]fn borrowed_records_and_stored_structural_assertions_keep_inputs(){native(r#"
import {deepStrictEqual,notDeepStrictEqual,strictEqual} from 'std:testing';
type Row={id:int;text:string;values:number[];};
type Checks={equal:Function<void,[actual:ReadOnly<Row>,expected:ReadOnly<Row>,message?:ReadOnly<string>]>;different:Function<void,[actual:ReadOnly<Row>,expected:ReadOnly<Row>,message?:ReadOnly<string>]>;};
function inspect(left:ReadOnly<Row>,right:ReadOnly<Row>,different:ReadOnly<Row>):void{
 const equalAlias=deepStrictEqual;const differentAlias=notDeepStrictEqual;
 equalAlias(left,right,'borrowed alias');differentAlias(left,different,'borrowed alias mismatch');
 const checks:Checks={equal:deepStrictEqual,different:notDeepStrictEqual};
 checks.equal(left,right,'stored borrowed equality');checks.different(left,different,'stored borrowed mismatch');
 strictEqual(left.id,9007199254740993);strictEqual(right.text,'é');
}
function main(){const left:Row={id:9007199254740993,text:'é',values:[Number.NaN,-0.0]};const right:Row={id:9007199254740993,text:'é',values:[Number.NaN,-0.0]};const other:Row={id:9007199254740992,text:'é',values:[Number.NaN,-0.0]};inspect(left,right,other);inspect(left,right,other);strictEqual(left.id,9007199254740993);strictEqual(right.values.length,2);}
"#);}
#[test]fn structural_maps_and_sets_preserve_nan_and_zero_key_semantics(){native(r#"
import {deepStrictEqual,notDeepStrictEqual,strictEqual} from 'std:testing';
function main(){
 const left=new Map<number,number[]>();left.set(Number.NaN,[Number.NaN,-0.0]);left.set(-0.0,[1.0]);
 const right=new Map<number,number[]>();right.set(0.0,[1.0]);right.set(Number.NaN,[Number.NaN,-0.0]);
 deepStrictEqual(left,right);strictEqual(left.size,2);strictEqual(left.has(Number.NaN),true);strictEqual(left.has(0.0),true);
 const changed=new Map<number,number[]>();changed.set(0.0,[1.0]);changed.set(Number.NaN,[Number.NaN,0.0]);notDeepStrictEqual(left,changed);
 const setA=new Set<number>([Number.NaN,-0.0,0.0]);const setB=new Set<number>([0.0,Number.NaN]);deepStrictEqual(setA,setB);strictEqual(setA.size,2);strictEqual(setA.has(0.0),true);strictEqual(setA.has(-0.0),true);strictEqual(setA.has(Number.NaN),true);
}
"#);}
#[test]fn rejects_aliases_and_stored_calls_accept_checked_job_transport(){
 let directory=support::TempDirectory::new();let source=r#"
import {rejects,fail} from 'std:testing';import {delay} from 'std:futures';
async function crash():Task<void>{fail('worker panic transported as checked data');}
async function waitForever():Task<void>{await delay(60000);}
type Reject=Function<Task<void>,[callback:Once<Function<Task<void>>>,message?:ReadOnly<string>]>;
async function main():Task<void>{
 const alias=rejects;await alias(async()=>{throw {code:'user-payload'};},'alias must observe a checked record');
 const stored:Reject=rejects;await stored(async()=>{throw 'stored checked payload';},'stored rejects must await');
 await rejects(async()=>{const job=crash().start();await job;});
 await rejects(async()=>{const job=waitForever().start();job.cancel();await job;});
 console.log('checked transport observed');
}
"#;let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert_eq!(output.stdout,b"checked transport observed\n");
 // The backend may print the worker's panic diagnostic before transporting its
 // checked AsyncError. Such stderr does not change the successful result.
 if !output.stderr.is_empty(){assert!(String::from_utf8_lossy(&output.stderr).contains("worker panic transported as checked data"));}
}

#[test]
fn deep_mismatch_diagnostics_preserve_exact_original_call_sites(){
 let directory=support::TempDirectory::new();let path=directory.0.join("deep-range.tsr");
 for(operation,actual,expected,message)in[("deepStrictEqual","[1,2]","[1,3]","nested mismatch"),("notDeepStrictEqual","[[1]]","[[1]]","unexpected deep equality")]{
  let source=format!("import {{deepStrictEqual,notDeepStrictEqual}} from 'std:testing';\nfunction main(){{\n  {operation}({actual},{expected},'{message}');\n}}\n");
  let source=support::format_round_trip(&source);let offset=source.rfind(&format!("{operation}(" )).unwrap();let prefix=&source[..offset];let line=prefix.bytes().filter(|byte|*byte==b'\n').count()+1;let column=prefix.rsplit('\n').next().unwrap().encode_utf16().count()+1;let site=format!("deep-range.tsr:{line}:{column}");
  std::fs::write(&path,&source).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);assert!(!output.status.success());
  let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains(message),"{stderr}");assert!(stderr.match_indices(&site).any(|(offset,_)|stderr.as_bytes().get(offset+site.len()).is_none_or(|next|!next.is_ascii_digit())),"Original assertion call site {site} missing: {stderr}");
 }
}

#[test]
fn structural_assertions_keep_native_ordered_maps_opaque(){
 for body in ["deepStrictEqual(left,right);","deepStrictEqual([left],[right]);"]{let directory=support::TempDirectory::new();let source=support::format_round_trip(&format!("import {{deepStrictEqual}} from 'std:testing';import {{OrderedMap}} from 'std:collections';function main(){{const left=new OrderedMap<int,int[]>();const right=new OrderedMap<int,int[]>();{body}}}"));let path=directory.source(&source);let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("native-capability"),"{}",String::from_utf8_lossy(&output.stderr));}
}

#[test]
fn timeout_waits_for_admitted_blocking_work_before_teardown() {
    use std::{path::PathBuf,process::{Child,Stdio},time::{Duration,Instant}};
    // Release first on every failure path; a blocked worker must never strand
    // either the generated process or this integration-test process.
    struct Running { child:Option<Child>, release:PathBuf }
    impl Running {
        fn release(&self) { let _=std::fs::write(&self.release,b"release"); }
        fn wait(&mut self,seconds:u64)->std::process::Output {
            let deadline=Instant::now()+Duration::from_secs(seconds);
            while self.child.as_mut().unwrap().try_wait().unwrap().is_none() {
                assert!(Instant::now()<deadline,"generated blocking cleanup did not settle");
                std::thread::sleep(Duration::from_millis(5));
            }
            self.child.take().unwrap().wait_with_output().unwrap()
        }
        fn marker(&mut self,path:&std::path::Path,seconds:u64) {
            let deadline=Instant::now()+Duration::from_secs(seconds);
            while !path.exists() {
                if self.child.as_mut().unwrap().try_wait().unwrap().is_some() {
                    let output=self.child.take().unwrap().wait_with_output().unwrap();
                    panic!("generated test exited before {}: {}",path.display(),String::from_utf8_lossy(&output.stderr));
                }
                assert!(Instant::now()<deadline,"missing worker marker {}",path.display());
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
    impl Drop for Running {
        fn drop(&mut self) {
            self.release();
            if let Some(child)=self.child.as_mut() {
                let deadline=Instant::now()+Duration::from_secs(5);
                while matches!(child.try_wait(),Ok(None)) && Instant::now()<deadline {std::thread::sleep(Duration::from_millis(5));}
                if !matches!(child.try_wait(),Ok(Some(_))) {let _=child.kill();}
                let _=child.wait();
            }
        }
    }
    let directory=support::TempDirectory::new();
    // Scalar-only source adapter. Its private helpers observe production worker
    // cancellation, then leave noninterruptible work behind an explicit gate.
    std::fs::write(directory.0.join("gate.rs"),r#"
fn marker(name:&str){std::fs::write(name,b"observed").unwrap();}
pub fn blocked()->i64{
    marker("started");
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
    while !crate::tsrust_native_threads::cancellation_requested(){
        assert!(std::time::Instant::now()<deadline,"worker cancellation deadline");
        std::thread::yield_now();
    }
    marker("cancelled");
    while !std::path::Path::new("release").exists(){
        assert!(std::time::Instant::now()<deadline,"worker release deadline");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    marker("finished");7
}
pub fn finished()->bool{std::path::Path::new("finished").exists()}
pub fn teardown(){marker("afterEach");}
"#).unwrap();
    let source=support::format_round_trip(r#"
import {test,afterEach,afterAll,trackedResource,resourceId,releaseCount,closeResource,strictEqual} from 'std:testing';
import {ThreadPool} from 'std:threads';
import {blocked,finished,teardown} from './gate.rs';
let owner=0;let cleaned=0;
afterEach(()=>{strictEqual(finished(),true);strictEqual(releaseCount(owner),1);teardown();cleaned++;});
afterAll(()=>{strictEqual(cleaned,2);console.log('all cleanup settled');});
test('blocking deadline',async()=>{
    const pool=new ThreadPool({workers:1,capacity:1});
    // The tracker stays in the local async task; worker settlement uses the gate.
    const resource=trackedResource();owner=resourceId(resource);
    await pool.taskAsync(():int=>blocked());
    closeResource(resource);
    throw 'deadline failed to interrupt admitted work';
});
test('continues after cleanup',()=>{strictEqual(cleaned,1);strictEqual(finished(),true);});
"#);
    let path=directory.source(&source);
    let child=support::cli().arg("test").arg(&path).arg("--timeout=1000").arg("--reporter=json")
        .current_dir(&directory.0).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut running=Running{child:Some(child),release:directory.0.join("release")};
    running.marker(&directory.0.join("started"),180);
    running.marker(&directory.0.join("cancelled"),5);
    assert!(!directory.0.join("finished").exists(),"admitted worker passed its unreleased gate");
    assert!(!directory.0.join("afterEach").exists(),"teardown began before admitted worker settlement");
    assert!(running.child.as_mut().unwrap().try_wait().unwrap().is_none(),"runner completed before worker release");
    running.release();
    let output=running.wait(10);
    assert!(!output.status.success(),"timed-out test unexpectedly passed");
    assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
    let rows=String::from_utf8(output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
    let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();
    assert_eq!(tests.len(),2);
    assert_eq!(tests[0]["name"],"blocking deadline");assert_eq!(tests[0]["status"],"failed");
    assert!(tests[0]["messages"].to_string().contains("TestTimeout"));
    assert_eq!(tests[1]["name"],"continues after cleanup");assert_eq!(tests[1]["status"],"passed");
    let summary=rows.iter().find(|row|row["event"]=="summary").unwrap_or_else(||panic!("Missing test summary; stderr: {}",String::from_utf8_lossy(&output.stderr)));
    assert_eq!(summary["passed"],1);assert_eq!(summary["failed"],1);assert_eq!(summary["hookFailures"],0);
    assert!(rows.iter().any(|row|row["event"]=="output"&&row["text"]=="all cleanup settled\n"));
}


#[test]
fn assertion_aliases_stored_and_erased_calls_preserve_original_sites(){
 let directory=support::TempDirectory::new();let path=directory.0.join("assertion-sites.tsr");
 for(setup,call,needle)in[
  ("", "strictEqual(1,2,'site mismatch');", "strictEqual("),
  ("const alias=deepStrictEqual;", "alias([1],[2],'site mismatch');", "alias("),
  ("const stored:Check=deepStrictEqual;", "stored([1],[2],'site mismatch');", "stored("),
  ("", "relay(deepStrictEqual);", "callback("),
  ("let mixed=deepStrictEqual;if(Boolean(true)){mixed=notDeepStrictEqual;}", "mixed([1],[1],'site mismatch');", "mixed("),
  ("", "strictEqual(inner(),0,'outer mismatch');", "deepStrictEqual("),
 ]{
  let source=support::format_round_trip(&format!("import {{strictEqual,deepStrictEqual,notDeepStrictEqual}} from 'std:testing';type Check=Function<void,[actual:ReadOnly<int[]>,expected:ReadOnly<int[]>,message?:ReadOnly<string>]>;function relay(callback:Check){{callback([1],[2],'site mismatch');}}function inner():int{{deepStrictEqual([1],[2],'site mismatch');return 0;}}function main(){{{setup}{call}}}"));
  let offset=source.rfind(needle).unwrap();let prefix=&source[..offset];let line=prefix.bytes().filter(|byte|*byte==b'\n').count()+1;let column=prefix.rsplit('\n').next().unwrap().encode_utf16().count()+1;let site=format!("assertion-sites.tsr:{line}:{column}");
  std::fs::write(&path,&source).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);assert!(!output.status.success());let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("site mismatch"),"{stderr}");assert!(stderr.match_indices(&site).any(|(offset,_)|stderr.as_bytes().get(offset+site.len()).is_none_or(|next|!next.is_ascii_digit())),"Missing original call {site}: {stderr}");
 }
}

#[test]
fn checked_assertion_errors_keep_payload_sites_order_and_restore(){
 let directory=support::TempDirectory::new();let path=directory.0.join("caught-assertion.tsr");
 let source=support::format_round_trip(r#"
import {strictEqual,deepStrictEqual} from 'std:testing';
let sequence=0;
function item(expected:int):int{sequence++;strictEqual(sequence,expected);return expected;}
function userFailure():int{return 1/zero();}function zero():int{return 0;}
function main(){
 strictEqual(7,7);const alias=deepStrictEqual;alias([1],[1]);
 try{strictEqual(item(1),item(2),'caught payload');}catch(error){console.log(error.name);console.log(error.code);console.log(error.message.startsWith('caught payload'));console.log(error.source);console.log(error.start);console.log(error.end);}
 strictEqual(sequence,2);
 try{userFailure();}catch(error){console.log(error.source);console.log(error.start);console.log(error.end);}
}
"#);
 std::fs::write(&path,&source).unwrap();
 // The checked generic-call fact retains the actual parsed invocation extent,
 // including formatter-inserted whitespace and argument line wrapping.
 let graph=tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,path.to_str().unwrap()).unwrap();let checked=tsrust_compiler::project::check_graph(graph).unwrap();
 let call=checked.generic_calls().into_iter().find(|call|call.location.source.0.ends_with("/caught-assertion.tsr")&&source[call.location.span.start..call.location.span.end].contains("caught payload")).expect("checked assertion invocation");let start=call.location.span.start;let end=call.location.span.end;
 let divisions=source.match_indices('/').map(|(offset,_)|offset).collect::<Vec<_>>();assert_eq!(divisions.len(),1);let division=divisions[0];
 let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 let stdout=String::from_utf8(output.stdout).unwrap();let lines=stdout.lines().collect::<Vec<_>>();assert_eq!(lines.len(),9,"{stdout}");assert_eq!(lines[0],"AssertionError");assert_eq!(lines[1],"assertion");assert_eq!(lines[2],"true");assert!(lines[3].ends_with("/caught-assertion.tsr"),"{stdout}");assert_eq!(lines[4],start.to_string());assert_eq!(lines[5],end.to_string());assert_eq!(lines[6],lines[3]);assert_eq!(lines[7],division.to_string());assert_eq!(lines[8],(division+1).to_string());
}


#[test]
fn assertion_provenance_does_not_wrap_known_user_functions(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {strictEqual as assertEqual} from 'std:testing';
function strictEqual(value:int):int{return value;}
function ordinary(value:int):int{return strictEqual(value);}
function main(){assertEqual(ordinary(7),7);}
"#);let path=directory.source(&source);let output=support::run(support::cli().arg("emit").arg(&path),30);support::success(&output);let rust=String::from_utf8(output.stdout).unwrap();assert_eq!(rust.matches("TsrAssertionGuard::new(").count(),1,"Only the trusted assertion invocation receives provenance");let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}


#[test]
fn assertion_provenance_excludes_task_invocation_but_keeps_sync_factories(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {strictEqual} from 'std:testing';import {delay} from 'std:futures';
type Work={task:Task<void>;};
function factory(callback:Once<Function<Task<void>>>):Task<void>{return callback();}
async function settle(task:Task<void>):Task<void>{await task;}
async function main():Task<void>{
 const concrete=delay(0);await concrete;
 await settle(delay(0));
 const work:Work={task:delay(0)};await work.task;
 const optional:Task<void>|undefined=delay(0);if(optional!==undefined){await optional;}
 await factory(():Task<void>=>delay(0));strictEqual(7,7);
}
"#);let path=directory.source(&source);let output=support::run(support::cli().arg("emit").arg(&path),30);support::success(&output);let rust=String::from_utf8(output.stdout).unwrap();
 // This factory specializes to a known non-asserting concrete closure, so
 // it needs no site. Actual Task invocation emits await internally and must
 // never hold TLS. The trusted assertion still exercises the guarded path.
 assert!(rust.contains("OnceCallback::invoke_once(value0, ())"),"Concrete synchronous factory still returns its Task without an internal await");
 assert!(rust.matches(".await").count()>=4,"Task invocation paths remain present");
 assert!(rust.contains("self.call().await.map_err("),"Task family Result bridge awaits before mapping its checked failure");
 assert!(!rust.contains("self.call().map_err("),"A Task family bridge must not map a Future as a Result");
 let marker="let _assertion_site=TsrAssertionGuard::new(";let mut guards=0;
 for(site,_)in rust.match_indices(marker){let start=rust[..site].rfind("{ ").expect("provenance block");let mut depth=0;let mut end=None;for(index,byte)in rust.as_bytes()[start..].iter().enumerate(){if *byte==b'{'{depth+=1;}else if *byte==b'}'{depth-=1;if depth==0{end=Some(start+index+1);break;}}}let block=&rust[start..end.expect("balanced provenance block")];assert!(!block.contains(".await"),"Assertion provenance crossed suspension: {block}");guards+=1;}
 assert_eq!(guards,1,"Only the trusted assertion receives a guard for these concrete origins");
 let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn rejects_started_and_cancelled_jobs_do_not_depend_on_prior_generic_aliases(){
 let source=support::format_round_trip(r#"
import {rejects,fail} from 'std:testing';import {delay} from 'std:futures';
async function crash():Task<void>{fail('worker panic transported as checked data');}
async function waitForever():Task<void>{await delay(60000);}
async function main():Task<void>{
 await rejects(async()=>{const job=crash().start();await job;});
 await rejects(async()=>{const job=waitForever().start();job.cancel();await job;});
 console.log('checked transport observed');
}
"#);
 let directory=support::TempDirectory::new();let path=directory.source(&source);
 let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);
 assert_eq!(output.stdout,b"checked transport observed\n");
 if !output.stderr.is_empty(){assert!(String::from_utf8_lossy(&output.stderr).contains("worker panic transported as checked data"));}
}
