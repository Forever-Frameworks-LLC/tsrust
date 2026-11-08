mod support;
#[path="support/io_suite_reports.rs"] mod io_suite_reports;
fn assert_safe_rust(source:&str){
 fn safe(tokens:proc_macro2::TokenStream)->bool{tokens.into_iter().all(|token|match token{proc_macro2::TokenTree::Ident(name)=>name!="unsafe",proc_macro2::TokenTree::Group(group)=>safe(group.stream()),_=>true})}
 assert!(safe(source.parse().expect("valid generated Rust tokens")));
}

use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
fn check(source:&str)->Result<String,String>{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();check_graph(graph).map(|checked|{assert!(checked.warnings().is_empty());let rust=checked.emit_rust();assert_safe_rust(&rust);rust}).map_err(|error|error.code.into())}
#[test]
fn explicit_results_and_two_input_task_loans_are_checked_together(){
 let prefix="import {reader,readInto,close} from 'std:io';import {fromUtf8,alloc,set} from 'std:bytes';";
 let positive=format!("{prefix}async function main():Task<void>{{const input=reader(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));const task=readInto(input,bytes);console.log(unwrap(await task));unwrap(set(bytes,0,1));unwrap(await close(input));}}");assert!(check(&positive).is_ok());
 for statement in ["unwrap(set(bytes,0,1));","const other=readInto(input,bytes);await other;","unwrap(await close(input));"]{
  let source=format!("{prefix}async function main():Task<void>{{const input=reader(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));const task=readInto(input,bytes);{statement}await task;}}");assert!(check(&source).is_err(),"loan accepted before settlement: {statement}");
 }
}
#[test]
fn readonly_destination_and_adapter_reuse_are_rejected(){
 for source in [
  "import {reader,readInto} from 'std:io';import {fromUtf8,view} from 'std:bytes';async function main():Task<void>{const bytes=unwrap(fromUtf8('abc'));const destination=unwrap(view(bytes,0,2));const input=reader(unwrap(fromUtf8('xy')));await readInto(input,destination);}",
  "import {reader} from 'std:io';import {fromUtf8,length} from 'std:bytes';function main(){const bytes=unwrap(fromUtf8('abc'));const input=reader(bytes);console.log(length(bytes));}",
 ]{assert!(check(source).is_err(),"accepted: {source}");}
}
#[test]
fn memory_reader_example_executes_with_explicit_results(){
 let source=include_str!("../../../examples/async-io/main.tsr");let _=check(source).unwrap();io_suite_reports::assert_registered_source("memory_reader_example_executes_with_explicit_results",source);
}
#[test]
fn consuming_buffered_adapters_preserves_unread_and_pending_bytes(){
 let source=r#"import {create,open,bufferReader,bufferWriter,write as syncWrite,read as syncRead} from 'std:fs';import {reader,writer,readAll,writeAll,flush,close} from 'std:io';import {fromUtf8,toUtf8} from 'std:bytes';async function main():Task<void>{const buffered=unwrap(bufferWriter(unwrap(create('buffered-io.txt')),8));unwrap(syncWrite(buffered,unwrap(fromUtf8('first'))));const output=writer(buffered);unwrap(await writeAll(output,unwrap(fromUtf8('-last'))));unwrap(await flush(output));unwrap(await close(output));const source=unwrap(bufferReader(unwrap(open('buffered-io.txt')),8));console.log(unwrap(toUtf8(unwrap(syncRead(source,2)))));const input=reader(source);console.log(unwrap(toUtf8(unwrap(await readAll(input)))));unwrap(await close(input));}"#;
 io_suite_reports::assert_registered_source("consuming_buffered_adapters_preserves_unread_and_pending_bytes",source);
}
#[test]
fn file_to_http_body_to_file_streams_without_whole_body_accumulation(){let source=include_str!("../../../examples/async-io/http-files.tsr");let _=check(source).unwrap();io_suite_reports::assert_registered_source("file_to_http_body_to_file_streams_without_whole_body_accumulation",source);}
#[test]
fn io_catalog_examples_agree_and_keep_results_explicit(){
 io_suite_reports::assert_sources();
 // Actual process stdout/stderr writes cannot share the NDJSON runner stream.
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 for name in ["stdout","stderr"]{let entry=catalog["std:io"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();let _=check(source).unwrap();let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let result=support::run(support::cli().arg("run").arg(path),180);support::success(&result);assert_eq!(String::from_utf8_lossy(&result.stdout),example["output"].as_str().unwrap());assert_eq!(String::from_utf8_lossy(&result.stderr),example["stderr"].as_str().unwrap_or(""));}}
}
#[test]
fn io_operation_values_use_checked_lazy_wrappers(){
 assert!(check("import {readInto} from 'std:io';function main(){const operation=readInto;}").is_ok());
 let source="import {reader,readAll,Reader,IoError} from 'std:io';import {Bytes} from 'std:bytes';async function read(input:Mutable<Reader>):Task<Result<Bytes,IoError>>{return await readAll(input);}function main(){}";assert!(check(source).is_ok());
}

#[test]
fn asynchronous_standard_output_and_error_writers_execute(){
 let source="import {stdout,stderr,writeAll,close} from 'std:io';import {fromUtf8} from 'std:bytes';async function main():Task<void>{const output=stdout();const errors=stderr();unwrap(await writeAll(output,unwrap(fromUtf8('output'))));unwrap(await writeAll(errors,unwrap(fromUtf8('errors'))));unwrap(await close(output));unwrap(await close(errors));}";let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let result=support::run(support::cli().arg("run").arg(path),180);support::success(&result);assert_eq!(result.stdout,b"output");assert_eq!(result.stderr,b"errors");
}
#[test]
fn synchronous_idle_stdin_blocks_only_its_process_and_exits_at_eof(){
 use std::io::Read;
 struct Child(std::process::Child);impl Drop for Child{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
 let source="import {stdin,readStdin} from 'std:io';import {length} from 'std:bytes';function main(){const input=stdin();console.log(length(unwrap(readStdin(input,1))));}";let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let binary=directory.0.join(format!("idle-stdin{}",std::env::consts::EXE_SUFFIX));let build=support::run(support::cli().arg("build").arg(path).arg("-o").arg(&binary),180);support::success(&build);
 let mut child=Child(std::process::Command::new(binary).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap());
 // A bounded process-isolated idle read must not report EOF while its input writer lives.
 std::thread::sleep(std::time::Duration::from_millis(100));assert!(child.0.try_wait().unwrap().is_none());drop(child.0.stdin.take());
 let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);loop{if child.0.try_wait().unwrap().is_some(){break;}assert!(std::time::Instant::now()<deadline,"stdin EOF exit deadline");std::thread::sleep(std::time::Duration::from_millis(5));}
 let mut output=String::new();child.0.stdout.take().unwrap().read_to_string(&mut output).unwrap();assert_eq!(output,"0\n");
}

#[test]
fn stored_io_calls_preserve_laziness_optional_arguments_views_and_two_loans(){
 let prefix="import {reader as adapt,readInto as fill,readAll,close,stdout,writeAll,IoError} from 'std:io';import {fromUtf8,alloc,viewMut,view,set,toUtf8} from 'std:bytes';function relay(work:Task<Result<int,IoError>>):Task<Result<int,IoError>>{return work;}";
 let source=format!("{prefix}async function main():Task<void>{{const make=adapt;const read=fill;const finish=close;const input=make(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));{{const destination=unwrap(viewMut(bytes,0,2));const task=read(input,destination);console.log(unwrap(await task));}}const rest=readAll;console.log(unwrap(toUtf8(unwrap(await rest(input)))));await finish(input);const output=stdout();const write=writeAll;{{const range=unwrap(view(bytes,0,2));unwrap(await write(output,range));}}unwrap(await finish(output));}}");
 assert!(check(&source).is_ok());io_suite_reports::assert_stored_stdio_source(&source);
 // This case deliberately writes through std:io stdout, outside reporter capture.
 // Keep its real process boundary and checked-in source/cache identity.
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
 let path=root.join("crates/tsrust-compiler/tests/fixtures/async-io/stored-operations.tsr");
 let result=support::run(support::cli().arg("run").arg(path).current_dir(root),180);
 support::success(&result);assert_eq!(result.stdout,b"2\nc\nab");
 for invocation in ["read(input,bytes)","(true?read:read)(input,bytes)","pass(read(input,bytes))"]{
  let setup=format!("{prefix}async function main():Task<void>{{const read=fill;const pass=relay;const input=adapt(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));const task={invocation};");
  assert!(check(&format!("{setup}await task;unwrap(set(bytes,0,1));}}")).is_ok(),"loan did not settle: {invocation}");
  for statement in ["unwrap(set(bytes,0,1));","const second=read(input,bytes);await second;"]{let source=format!("{setup}{statement}await task;}}");assert_eq!(check(&source).unwrap_err(),"capture-conflict","stored operation lost loan: {invocation}/{statement}");}
 }
}

#[test]
fn started_io_tasks_reject_caller_loans_and_owned_readers_transfer(){
 let prefix="import {reader,readInto,close,Reader} from 'std:io';import {fromUtf8,alloc,set} from 'std:bytes';";
 let negative=format!("{prefix}async function main():Task<void>{{const input=reader(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));const job=readInto(input,bytes).start();job.cancel();unwrap(set(bytes,0,1));try{{await job;}}catch(error){{}}}}");assert_eq!(check(&negative).unwrap_err(),"task-transfer","started I/O must not retain caller loans");
 let positive=format!("{prefix}async function run(input:Reader):Task<int>{{const bytes=unwrap(alloc(3));const count=unwrap(await readInto(input,bytes));unwrap(await close(input));return count;}}async function main():Task<void>{{const job=run(reader(unwrap(fromUtf8('abc')))).start();console.log(await job);}}");assert!(check(&positive).is_ok());io_suite_reports::assert_registered_source("started_io_tasks_reject_caller_loans_and_owned_readers_transfer",&positive);
}
#[test]
fn io_http_package_exports_and_builds_offline_after_relocation(){
 let directory=support::TempDirectory::new();let input=directory.source(&support::format_round_trip(include_str!("../../../examples/async-io/http-files.tsr")));let package=directory.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
 let manifest=std::fs::read_to_string(package.join("Cargo.toml")).unwrap();assert!(manifest.contains("\"io\""));assert!(manifest.contains("tokio/io-util"));let rust=std::fs::read_to_string(package.join("src/main.rs")).unwrap();assert!(!rust.contains("runtime_tests.rs"));assert_safe_rust(&rust);std::fs::write(package.join("src/main.rs"),format!("#![forbid(unsafe_code)]\n{rust}")).unwrap();
 std::fs::remove_file(input).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 let mut build=std::process::Command::new("rustup");build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);support::success(&support::run(&mut build,300));
 let output=support::run(std::process::Command::new(target.join(format!("debug/program{}",std::env::consts::EXE_SUFFIX))).current_dir(&relocated),30);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"20\nsplit-é-lines\r\nlast\n");
}

#[test]
fn io_error_field_types_and_alias_identity_are_checked(){
 let rust=check("import {IoError as Failure} from 'std:io';function inspect(error:ReadOnly<Failure>){console.log(error.code,error.operation,error.message,error.osCode);}function main(){}").unwrap();assert!(!rust.contains("mod tsrust_native_io")&&!rust.contains("mod tsrust_native_futures"),"error data alone must not activate I/O backends");
 assert!(check("import {IoError as Failure} from 'std:io';function inspect(error:ReadOnly<Failure>){error.code='other';}function main(){}").is_err());
}

#[test]
fn existing_lazy_sql_and_http_calls_keep_owned_argument_snapshots(){
 for source in [
 "import {connect,Pool} from 'std:sql';function opening(url:Owned<string>):Task<Pool>{return connect(url);}async function main():Task<void>{const pool=await opening('sqlite::memory:');await pool.close();}",
 "import {App,Server} from 'std:http';function listening(app:Owned<App>,address:Owned<string>):Task<Server>{return app.listen(address);}async function main():Task<void>{const server=await listening(new App(),'127.0.0.1:0');await server.shutdown();}",
 ]{assert!(check(source).is_ok(),"owned snapshot rejected: {source}");}
}

#[test]
fn consumed_io_task_relays_preserve_outer_and_invariant_view_lifetimes_at_o0_and_o3(){
 let source="import {reader,readInto,close,IoError} from 'std:io';import {fromUtf8,alloc,viewMut,set,toUtf8} from 'std:bytes';function relay(work:Task<Result<int,IoError>>):Task<Result<int,IoError>>{return work;}function again(work:Task<Result<int,IoError>>):Task<Result<int,IoError>>{const pass=relay;return pass(work);}async function main():Task<void>{const input=reader(unwrap(fromUtf8('abc')));const bytes=unwrap(alloc(3));{const destination=unwrap(viewMut(bytes,0,3));const read=readInto;const pending=again(relay(read(input,destination)));console.log(unwrap(await pending));unwrap(set(destination,0,120));}console.log(unwrap(toUtf8(bytes)));unwrap(await close(input));}";
 let _=check(source).unwrap();for statement in["unwrap(set(destination,0,120));","const other=readInto(input,destination);await other;"]{let invalid=source.replace("console.log(unwrap(await pending));",&format!("{statement}console.log(unwrap(await pending));"));assert_eq!(check(&invalid).unwrap_err(),"capture-conflict");}
 io_suite_reports::assert_registered_source("consumed_io_task_relays_preserve_outer_and_invariant_view_lifetimes_at_o0_and_o3",source);
}

#[test]
fn registered_async_io_runtime_family_executes_at_both_required_profiles(){
 io_suite_reports::assert_sources();let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();let suite=root.join("crates/tsrust-compiler/tests/fixtures/async-io/runtime.test.tsr");
 // Only runtime data is temporary. Absolute suite paths and an explicit shared
 // native cache preserve executable identity without changing this host's cwd.
 let data=support::TempDirectory::new();let workspace=&data.0;
 let cache=root.join("target/tsrust-native");
 for optimized in [false,true]{
  let mut command=support::cli();command.current_dir(workspace).env("TSRUST_CACHE_DIR",&cache).args(["test",suite.to_str().unwrap(),"--reporter=json","--profile","--timeout=10000"]);if optimized{command.arg("--release");}
  io_suite_reports::assert_report(support::run(&mut command,900));
  // The buffered fixture originally relied on its temporary directory cleanup.
  std::fs::remove_file(workspace.join("buffered-io.txt")).unwrap();
  for name in ["io-source.txt","io-destination.txt","pipeline.z"]{assert!(!workspace.join(name).exists(),"fixture leaked {name}");}
 }
}

#[test]
fn heavy_build_classifier_keeps_frontend_checks_and_native_runs_concurrent(){
 for (program,args,expected) in [
  ("/tool/tsrust",vec!["build","input.tsr"],true),
  ("/tool/tsrust",vec!["run","input.tsr"],true),
  ("C:\\tool\\TSRUST.EXE",vec!["test","runtime.test.tsr"],true),
  ("cargo",vec!["build","--offline"],true),
  ("rustup",vec!["run","1.96.1","cargo","build"],true),
  ("rustc",vec!["input.rs"],true),
  ("/tool/tsrust",vec!["check","input.tsr"],false),
  ("/tool/tsrust",vec!["emit","input.tsr"],false),
  ("/tool/tsrust",vec!["format","input.tsr"],false),
  ("/data/idle-stdin",vec![],false),
  ("/data/program.exe",vec![],false),
 ]{let mut command=std::process::Command::new(program);command.args(args);assert_eq!(support::is_heavy_build(&command),expected,"{command:?}");}
}
