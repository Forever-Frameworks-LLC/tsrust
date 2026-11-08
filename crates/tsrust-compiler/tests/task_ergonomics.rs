mod support;
fn assert_registered(source:&str,expected:&str){
 let (original,exported)=if source.contains("async function main("){("async function main(","export async function run(")}else{("function main(","export function run(")};
 assert_eq!(source.matches(original).count(),1);
 let formatted=support::format_round_trip(&source.replacen(original,exported,1));
 assert!(REGISTERED_TASKS.iter().any(|(_,registered,output)|support::format_round_trip(registered)==formatted && *output==expected),"original task source or output lost from the registered suite");
}
fn execute_without_runtime(source:&str,expected:&str){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let fixture=root.join("crates/tsrust-compiler/tests/fixtures/task-ergonomics/outside-runtime.tsr");
 assert_eq!(support::format_round_trip(source),support::format_round_trip(&std::fs::read_to_string(&fixture).unwrap()));
 // This witness needs no enclosing async runtime; a registered suite has one.
 let output=support::run(support::cli().current_dir(&root).arg("run").arg(fixture),120);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}
#[test]
fn documented_task_timer_channel_examples_execute(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut seen=std::collections::BTreeSet::new();
 for family in ["Interval","CancellationToken"]{for entry in catalog[family].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source){assert_registered(source,example["output"].as_str().unwrap());}}}}
 for name in ["all","Interval","CancellationToken","MissedTickBehavior","SendError","ChannelError"]{let entry=catalog["std:futures"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source){assert_registered(source,example["output"].as_str().unwrap());}}}
 let entry=catalog["std:threads"].as_array().unwrap().iter().find(|entry|entry["name"]=="SendError").unwrap();for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source){assert_registered(source,example["output"].as_str().unwrap());}}
 let entry=catalog["ThreadPool"].as_array().unwrap().iter().find(|entry|entry["name"]=="taskAsync").unwrap();for example in entry["examples"].as_array().unwrap(){assert_registered(example["source"].as_str().unwrap(),example["output"].as_str().unwrap());}
}
#[test]
fn all_preserves_order_unit_arrays_and_original_typed_failure(){
 assert_registered("import {all,yield} from 'std:futures';async function value(n:int):Task<int>{await yield();return n;}async function unit():Task<void>{}async function fail():Task<int>{throw 'cause';}async function main():Task<void>{const values=await all([value(2),value(1).start()]);console.log(values[0],values[1]);const units:void[]=await all([unit(),unit().start()]);console.log(units.length,(await all([])).length);try{await all([value(3),fail()]);}catch(error){if(typeof error.cause==='string'){console.log(error.code,error.cause);}else{throw 'unexpected failure';}}}","2 1\n2 0\nworker cause\n");
}
#[test]
fn timer_constructor_context_and_range_are_checked_failures(){
 execute_without_runtime("import {Interval} from 'std:futures';function main(){try{const timer=new Interval(1);}catch(error){console.log(error.code);}try{const timer=new Interval(0);}catch(error){console.log(error.code);}}","runtime\nrange\n");
}
#[test]
fn task_async_admission_and_completion_errors_keep_distinct_types(){
 assert_registered("import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});console.log(await pool.taskAsync(():int=>7));try{await pool.taskAsync(():int=>{throw 'cause';});}catch(error){console.log(error.code);}unwrap(pool.shutdown());try{await pool.taskAsync(():int=>8);}catch(error){console.log(error.code);} }","7\nworker\nclosed\n");
}
#[test]
fn exclusive_tick_and_try_receive_loans_and_invalid_policies_are_rejected(){
 for source in [
 "import {Interval} from 'std:futures';async function main():Task<void>{const timer=new Interval(1);const pending=timer.tick();await timer.tick();await pending;}",
 "import {Interval} from 'std:futures';async function main():Task<void>{const timer=new Interval(1,'unknown');}",
 "import {Channel} from 'std:futures';async function main():Task<void>{const queue=new Channel<int>(1);const pending=queue.receive();queue.tryReceive();await pending;}",
 ]{let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(source)),30);assert!(!output.status.success(),"accepted {source}");}
}

#[test]
fn task_async_typed_causes_survive_direct_started_and_all_transports(){
 let source="import {ThreadPool} from 'std:threads';import {all} from 'std:futures';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:2});try{await pool.taskAsync(():int=>{throw 'cause';});}catch(error){console.log(JSON.stringify(error));}try{await pool.taskAsync(():int=>{throw 'cause';}).start();}catch(error){console.log(JSON.stringify(error));}try{await all([pool.taskAsync(():int=>{throw 'cause';}).start()]);}catch(error){console.log(JSON.stringify(error));}unwrap(pool.shutdown());}";
 assert_registered(source, "");
}

#[test]
fn cancellation_dependency_is_optional_and_async_export_builds_offline(){
 for manifest in [include_str!("../backend-dependencies/Cargo.toml"),include_str!("../native-dependencies/Cargo.toml")]{let doc=manifest.parse::<toml_edit::DocumentMut>().unwrap();assert_eq!(doc["dependencies"]["tokio-util"]["version"].as_str(),Some("=0.7.20"));assert_eq!(doc["dependencies"]["tokio-util"]["default-features"].as_bool(),Some(false));assert_eq!(doc["dependencies"]["tokio-util"]["optional"].as_bool(),Some(true));}
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip("import {CancellationToken,yield} from 'std:futures';import {ThreadPool} from 'std:threads';async function main():Task<void>{const token=new CancellationToken();token.cancel();await token.cancelled();await yield();const pool=new ThreadPool({workers:1,capacity:1});console.log(await pool.taskAsync(():int=>7),token.isCancelled());unwrap(pool.shutdown());}"));
 support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
 let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));
 std::fs::remove_file(path).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap().parse::<toml_edit::DocumentMut>().unwrap();for feature in ["threads","async","cancellation"]{assert!(manifest["features"]["default"].as_array().unwrap().iter().any(|value|value.as_str()==Some(feature)),"{feature}");}
 assert!(relocated.join("vendor/tokio-util-0.7.20/LICENSE").is_file());let file=relocated.join("src/main.rs");let source=std::fs::read_to_string(&file).unwrap();assert!(!source.contains("unsafe"));std::fs::write(file,format!("#![forbid(unsafe_code)]\n{source}")).unwrap();
 let target=std::path::PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("shared target"));assert!(target.is_absolute());let home=directory.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 for release in [false,true]{let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");if release{command.arg("--release");}support::success(&support::run(&mut command,180));let result=support::run(&mut std::process::Command::new(target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX))),30);support::success(&result);assert_eq!(result.stdout,b"7 true\n");}
}

#[test]
fn task_async_moves_owned_captures_without_making_the_pool_transferable(){
 assert_registered("import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const payload='owned'.repeat(2);const task=pool.taskAsync(():int=>payload.length);console.log(await task.start());unwrap(pool.shutdown());}","10\n");
 let directory=support::TempDirectory::new();let source="import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});await pool.taskAsync(():int=>pool.stats().outstanding);}";
 let output=support::run(support::cli().arg("check").arg(directory.source(source)),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("thread-transfer"),"{:?}",output);
}

#[test]
fn synchronous_try_operations_prepare_channel_error_types(){
 assert_registered("import {Channel} from 'std:futures';function main(){const channel=new Channel<string>(1);unwrap(channel.trySend('owned'));console.log(unwrap(channel.tryReceive()));try{unwrap(channel.tryReceive());}catch(error){console.log(error.code);}}","owned\nempty\n");
}

#[test]
fn all_materializes_scalar_record_and_nested_array_carriers(){
 assert_registered("import {all} from 'std:futures';type Item={value:int};async function text():Task<string>{return 'owned'.repeat(2);}async function record():Task<Item>{return {value:7};}async function array():Task<int[]>{return [3,4];}async function main():Task<void>{const texts=await all([text(),text().start()]);const records=await all([record(),record().start()]);const arrays=await all([array(),array().start()]);console.log(texts[0],texts[1],records[0].value,records[1].value,arrays[0][0],arrays[1][1]);}","ownedowned ownedowned 7 7 3 4\n");
}

#[test]
fn send_error_clone_is_structural_through_records_void_results_and_settled_jobs(){
 assert_registered(r#"import {SendError as AsyncSendError,Channel,allSettled} from 'std:futures';
import {SendError as SyncSendError} from 'std:threads';
import {strictEqual} from 'std:testing';
type ErrorRecord={error:AsyncSendError<int[]>;};
async function rejected():Task<void>{const queue=new Channel<int>(1);const sender=queue.sender();queue.close();await sender.send(7);}
async function rejectedValue():Task<void>{const error:AsyncSendError<int>={code:'closed',value:7};throw error;}
async function main():Task<void>{
 const plain:SyncSendError<int[]>={code:'closed',value:[1]};const plainCopy=clone(plain);plainCopy.value.push(2);strictEqual(plain.value.length,1);strictEqual(plainCopy.value.length,2);
 const record:ErrorRecord={error:{code:'closed',value:[3]}};const recordCopy=clone(record);recordCopy.error.value.push(4);strictEqual(record.error.value.length,1);strictEqual(recordCopy.error.value.length,2);
 const unit:Result<void,AsyncSendError<int>>=Err({code:'closed',value:8});match(clone(unit)){Ok(_)=>{throw 'unexpected success';},Err(error)=>{strictEqual(error.value,8);}}
 const settled=await allSettled([rejected().start()]);match(clone(settled[0])){Ok(_)=>{throw 'unexpected success';},Err(error)=>{strictEqual(error.code,'worker');strictEqual(error.cause!.code,'closed');}}
 const payloads=await allSettled([rejectedValue().start()]);match(clone(payloads[0])){Ok(_)=>{throw 'unexpected success';},Err(error)=>{strictEqual(error.code,'worker');strictEqual(error.cause!.code,'closed');strictEqual(error.cause!.value,7);}}
 console.log('send error clones preserve owned payloads');
}
"#,"send error clones preserve owned payloads\n");
}

#[test]
fn send_error_clone_rejects_resource_and_callable_payloads_in_every_wrapper(){
 for payload in ["TrackedResource","CallablePayload"]{
  for shape in [format!("SendError<{payload}>"),format!("{{error:SendError<{payload}>;}}"),format!("Result<void,SendError<{payload}>>"),format!("AsyncError<SendError<{payload}>>")]{
   let source=format!("import {{SendError,AsyncError}} from 'std:futures';import {{TrackedResource}} from 'std:testing';type CallablePayload={{call:Function<int>;}};type Wrapped={shape};function inspect(error:ReadOnly<Wrapped>){{const copied=clone(error);}}function main(){{}}");
   let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
   let result=support::run(support::cli().arg("check").arg(path),30);assert!(!result.status.success(),"accepted {shape}");
   assert!(String::from_utf8_lossy(&result.stderr).contains("error[clone-type]"),"{shape}: {}",String::from_utf8_lossy(&result.stderr));
  }
 }
}

#[test]
fn nested_async_sender_calls_keep_payload_context_without_a_sender_type_import(){
 use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
 for source in [
  "import {Channel} from 'std:futures';async function main():Task<void>{const channel=new Channel<int>(1);unwrap(channel.sender().trySend(1));channel.close();}",
  "import {Channel} from 'std:futures';import {SseEvent} from 'std:http';async function main():Task<void>{const channel=new Channel<SseEvent>(1);unwrap(channel.sender().trySend({data:''}));channel.close();}",
 ]{
  let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
  let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
  let checked=check_graph(graph).unwrap_or_else(|error|panic!("nested sender: {error:?}"));drop(checked.emit_rust());
 }
 let source="import {Channel} from 'std:futures';import {SseEvent} from 'std:http';async function main():Task<void>{const channel=new Channel<SseEvent>(1);unwrap(channel.sender().trySend({data:1}));channel.close();}";
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
 let error=check_graph(graph).err().expect("nested sender must retain the endpoint payload's field type");assert_eq!(error.code,"type");
}

const REGISTERED_TASKS:&[(&str,&str,&str)]=&[
    ("all_preserves_order_unit_arrays_and_original_typed_failure",include_str!("fixtures/task-ergonomics/runtime-cases/all-preserves-order-unit-arrays-and-original-typed-failure.tsr"),"2 1\n2 0\nworker cause\n"),
    ("task_async_admission_and_completion_errors_keep_distinct_types",include_str!("fixtures/task-ergonomics/runtime-cases/task-async-admission-and-completion-errors-keep-distinct-types.tsr"),"7\nworker\nclosed\n"),
    ("task_async_moves_owned_captures_without_making_the_pool_transferable",include_str!("fixtures/task-ergonomics/runtime-cases/task-async-moves-owned-captures-without-making-the-pool-transferable.tsr"),"10\n"),
    ("synchronous_try_operations_prepare_channel_error_types",include_str!("fixtures/task-ergonomics/runtime-cases/synchronous-try-operations-prepare-channel-error-types.tsr"),"owned\nempty\n"),
    ("all_materializes_scalar_record_and_nested_array_carriers",include_str!("fixtures/task-ergonomics/runtime-cases/all-materializes-scalar-record-and-nested-array-carriers.tsr"),"ownedowned ownedowned 7 7 3 4\n"),
    ("send_error_clone_is_structural_through_records_void_results_and_settled_jobs",include_str!("fixtures/task-ergonomics/runtime-cases/send-error-clone-is-structural-through-records-void-results-and-settled-jobs.tsr"),"send error clones preserve owned payloads\n"),
    ("catalog-interval-tick",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-interval-tick.tsr"),"tick\n"),
    ("catalog-cancellationtoken-share",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-cancellationtoken-share.tsr"),"false true\ntrue\n"),
    ("catalog-futures-all",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-futures-all.tsr"),"2 1\n2 0\n"),
    ("catalog-futures-senderror",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-futures-senderror.tsr"),"7\nclosed 9\n"),
    ("catalog-threads-senderror",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-threads-senderror.tsr"),"full second\n"),
    ("catalog-threadpool-taskasync",include_str!("fixtures/task-ergonomics/runtime-cases/catalog-threadpool-taskasync.tsr"),"0\n42\n0\n"),
    ("task_async_typed_causes_survive_direct_started_and_all_transports",include_str!("fixtures/task-ergonomics/runtime-cases/task-async-typed-causes-survive-direct-started-and-all-transports.tsr"),""),
];

#[test]
fn registered_task_runtime_examples_share_one_profiled_runner(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let suite=root.join("crates/tsrust-compiler/tests/fixtures/task-ergonomics/runtime.test.tsr");
 let output=support::run(support::cli().current_dir(&root).arg("test").arg(suite).args(["--reporter=json","--profile","--timeout=10000"]),600);support::success(&output);
 let stderr=String::from_utf8(output.stderr).unwrap();
 if !stderr.is_empty(){
  // The original catalog launcher permitted this task-responsibility warning.
  eprint!("{stderr}");assert_eq!(stderr.matches("warning[").count(),1,"{stderr}");assert!(stderr.starts_with("warning[unused-task]:") && stderr.contains("catalog-threadpool-taskasync.tsr") && !stderr.contains("error["),"{stderr}");
 }
 let text=String::from_utf8(output.stdout).unwrap();print!("{text}");
 let rows=text.lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),REGISTERED_TASKS.len());
 let mut expected=std::collections::BTreeSet::new();
 for(name,_,stdout)in REGISTERED_TASKS {
  let name=format!("task ergonomics > {name}");assert!(expected.insert(name.clone()));
  let case=tests.iter().filter(|row|row["name"]==name).collect::<Vec<_>>();assert_eq!(case.len(),1);assert_eq!(case[0]["status"],"passed","{name}");
  let mut actual=String::new();for row in rows.iter().filter(|row|row["event"]=="output" && row["name"]==name){assert_eq!(row["stream"],"stdout","{name}: unexpected stderr");actual.push_str(row["text"].as_str().unwrap());}
  if name=="task ergonomics > task_async_typed_causes_survive_direct_started_and_all_transports" {
   let actual=actual.lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
   let direct=serde_json::json!({"code":"worker","error":"cause"});let wrapped=serde_json::json!({"code":"worker","cause":direct});assert_eq!(actual,vec![direct,wrapped.clone(),wrapped]);
  }else{assert_eq!(actual,*stdout,"{name}");}
 }
 let actual=tests.iter().map(|row|row["name"].as_str().unwrap().to_owned()).collect::<std::collections::BTreeSet<_>>();assert_eq!(actual,expected);
 for row in rows.iter().filter(|row|row["event"]=="output"){assert!(row["name"].as_str().is_some_and(|name|expected.contains(name)),"unattributed output: {row}");assert_eq!(row["stream"],"stdout");}
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);
 for(field,value)in[("passed",13),("failed",0),("skipped",0),("hookFailures",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summaries[0][field],value);}
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);
}
