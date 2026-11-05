mod support;
fn execute(source:&str,expected:&str){let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let input=directory.source(&source);let output=support::run(support::cli().arg("run").arg(input),120);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());}
#[test]
fn documented_task_timer_channel_examples_execute(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut seen=std::collections::BTreeSet::new();
 for family in ["Interval","CancellationToken"]{for entry in catalog[family].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source){execute(source,example["output"].as_str().unwrap());}}}}
 for name in ["all","Interval","CancellationToken","MissedTickBehavior","SendError","ChannelError"]{let entry=catalog["std:futures"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source){execute(source,example["output"].as_str().unwrap());}}}
 let entry=catalog["ThreadPool"].as_array().unwrap().iter().find(|entry|entry["name"]=="taskAsync").unwrap();for example in entry["examples"].as_array().unwrap(){execute(example["source"].as_str().unwrap(),example["output"].as_str().unwrap());}
}
#[test]
fn all_preserves_order_unit_arrays_and_original_typed_failure(){
 execute("import {all,yield} from 'std:futures';async function value(n:int):Task<int>{await yield();return n;}async function unit():Task<void>{}async function fail():Task<int>{throw 'cause';}async function main():Task<void>{const values=await all([value(2),value(1).start()]);console.log(values[0],values[1]);const units:void[]=await all([unit(),unit().start()]);console.log(units.length,(await all([])).length);try{await all([value(3),fail()]);}catch(error){if(typeof error.cause==='string'){console.log(error.code,error.cause);}else{throw 'unexpected failure';}}}","2 1\n2 0\nworker cause\n");
}
#[test]
fn timer_constructor_context_and_range_are_checked_failures(){
 execute("import {Interval} from 'std:futures';function main(){try{const timer=new Interval(1);}catch(error){console.log(error.code);}try{const timer=new Interval(0);}catch(error){console.log(error.code);}}","runtime\nrange\n");
}
#[test]
fn task_async_admission_and_completion_errors_keep_distinct_types(){
 execute("import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});console.log(await pool.taskAsync(():int=>7));try{await pool.taskAsync(():int=>{throw 'cause';});}catch(error){console.log(error.code);}unwrap(pool.shutdown());try{await pool.taskAsync(():int=>8);}catch(error){console.log(error.code);} }","7\nworker\nclosed\n");
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
 execute("import {ThreadPool} from 'std:threads';import {all} from 'std:futures';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:2});try{await pool.taskAsync(():int=>{throw 'cause';});}catch(error){console.log(JSON.stringify(error));}try{await pool.taskAsync(():int=>{throw 'cause';}).start();}catch(error){console.log(JSON.stringify(error));}try{await all([pool.taskAsync(():int=>{throw 'cause';}).start()]);}catch(error){console.log(JSON.stringify(error));}unwrap(pool.shutdown());}","{\"code\":\"worker\",\"cause\":\"cause\"}\n{\"code\":\"worker\",\"cause\":{\"code\":\"worker\",\"cause\":\"cause\"}}\n{\"code\":\"worker\",\"cause\":{\"code\":\"worker\",\"cause\":\"cause\"}}\n");
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
 execute("import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});const payload='owned'.repeat(2);const task=pool.taskAsync(():int=>payload.length);console.log(await task.start());unwrap(pool.shutdown());}","10\n");
 let directory=support::TempDirectory::new();let source="import {ThreadPool} from 'std:threads';async function main():Task<void>{const pool=new ThreadPool({workers:1,capacity:1});await pool.taskAsync(():int=>pool.stats().outstanding);}";
 let output=support::run(support::cli().arg("check").arg(directory.source(source)),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("thread-transfer"),"{:?}",output);
}

#[test]
fn synchronous_try_operations_prepare_channel_error_types(){
 execute("import {Channel} from 'std:futures';function main(){const channel=new Channel<string>(1);unwrap(channel.trySend('owned'));console.log(unwrap(channel.tryReceive()));try{unwrap(channel.tryReceive());}catch(error){console.log(error.code);}}","owned\nempty\n");
}

#[test]
fn all_materializes_scalar_record_and_nested_array_carriers(){
 execute("import {all} from 'std:futures';type Item={value:int};async function text():Task<string>{return 'owned'.repeat(2);}async function record():Task<Item>{return {value:7};}async function array():Task<int[]>{return [3,4];}async function main():Task<void>{const texts=await all([text(),text().start()]);const records=await all([record(),record().start()]);const arrays=await all([array(),array().start()]);console.log(texts[0],texts[1],records[0].value,records[1].value,arrays[0][0],arrays[1][1]);}","ownedowned ownedowned 7 7 3 4\n");
}
