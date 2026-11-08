mod support;
use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
fn check(source:&str)->Result<String,String>{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).map_err(|error|format!("{}: {}",error.code,error.message))?;check_graph(graph).map(|checked|{assert!(checked.warnings().is_empty(),"{:?}",checked.warnings());let rust=checked.emit_rust();fn safe(tokens:proc_macro2::TokenStream)->bool{tokens.into_iter().all(|token|match token{proc_macro2::TokenTree::Ident(name)=>name!="unsafe",proc_macro2::TokenTree::Group(group)=>safe(group.stream()),_=>true})}assert!(safe(rust.parse().unwrap()));rust}).map_err(|error|format!("{}: {}",error.code,error.message))}
fn execute(source:&str,expected:&str){let _=check(source).unwrap();let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),180);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),expected);}
#[test]
fn process_examples_and_catalog_execute_explicit_results(){
 execute(include_str!("../../../examples/process/main.tsr"),"true true\ntrue\ntrue true\ntrue\n");
 execute(include_str!("../../../examples/process/pipes.tsr"),"hello note true\n");
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let mut seen=std::collections::HashSet::new();for group in ["std:process","Child"]{for entry in catalog[group].as_array().unwrap(){assert!(!entry["examples"].as_array().unwrap().is_empty(),"{} lacks example",entry["name"]);for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if seen.insert(source.to_owned()){eprintln!("process catalog: {}",entry["name"]);execute(source,example["output"].as_str().unwrap());}}}}
}
#[test]
fn run_aliases_optional_options_and_owned_inputs_preserve_explicit_result(){
 let source=r#"import {run as launch,CommandOptions,ProcessError,Output} from 'std:process';import {args,currentExe} from 'std:env';import {fromUtf8,toUtf8,length} from 'std:bytes';import {reader,stdin,readStdin,stdout,writeAll,close} from 'std:io';function relay(task:Task<Result<Output,ProcessError>>):Task<Result<Output,ProcessError>>{return task;}async function main():Task<void>{if(unwrap(args()).length>0){const input=stdin();while(length(unwrap(readStdin(input,8192)))>0){}const output=stdout();unwrap(await writeAll(output,unwrap(fromUtf8('complete'))));unwrap(await close(output));return;}const executable=unwrap(currentExe());const arguments=['--child'];const invoke=launch;const options:CommandOptions|undefined={input:unwrap(fromUtf8('bytes'))};const task=relay((true?invoke:invoke)(executable,arguments,options));console.log(unwrap(toUtf8(unwrap(await task).stdout)));arguments.push('after');arguments.pop();const input=reader(unwrap(fromUtf8('reader')));console.log(unwrap(toUtf8(unwrap(await invoke(executable,arguments,{input})).stdout)));}"#;
 execute(source,"complete\ncomplete\n");
}
#[test]
fn process_pending_tasks_retain_program_arguments_and_child_loans(){
 let prefix="import {run,Output,ProcessError} from 'std:process';function pass(task:Task<Result<Output,ProcessError>>):Task<Result<Output,ProcessError>>{return task;}";
 for invocation in ["operation(program,arguments)","(true?operation:operation)(program,arguments)","pass(operation(program,arguments))"]{
 let setup=format!("{prefix}async function main():Task<void>{{const operation=run;const program='tsrust-missing';const arguments:string[]=[];const task={invocation};");
 assert!(check(&format!("{setup}await task;arguments.push('settled');console.log(program);}}")).is_ok());
 assert!(check(&format!("{setup}arguments.push('pending');await task;}}")).is_err(),"lost args loan: {invocation}");
 assert!(check(&format!("{setup}const observed=program.length;await task;console.log(observed);}}")).is_ok(),"readonly observation should remain legal");
 }
 assert!(check("import {spawn} from 'std:process';async function main():Task<void>{const child=unwrap(spawn('missing',[]));const task=child.wait();child.tryWait();await task;}").is_err());
 assert!(check("import {spawn} from 'std:process';async function main():Task<void>{const child=unwrap(spawn('missing',[]));const task=child.wait();await task;child.tryWait();}").is_ok());
}
#[test]
fn process_owned_input_and_started_factory_contracts_are_checked(){
 for source in [
 "import {run} from 'std:process';import {fromUtf8,length} from 'std:bytes';async function main():Task<void>{const bytes=unwrap(fromUtf8('input'));const task=run('missing',[],{input:bytes});console.log(length(bytes));await task;}",
 "import {run} from 'std:process';async function main():Task<void>{const program='missing';const args:string[]=[];const job=run(program,args).start();await job;}",
 "import {spawn} from 'std:process';async function main():Task<void>{const child=unwrap(spawn('missing',[]));const duplicate=clone(child);}",
 ]{assert!(check(source).is_err(),"accepted ownership violation: {source}");}
 let source="import {run,Output,ProcessError} from 'std:process';async function work(program:Owned<string>,arguments:Owned<string[]>):Task<Result<Output,ProcessError>>{return await run(program,arguments);}async function main():Task<void>{const job=work('tsrust-missing',[]).start();match(await job){Ok(_)=>{},Err(error)=>console.log(error.code)}}";execute(source,"not-found\n");
}
#[test]
fn process_validation_precedes_spawn_and_nonzero_exit_is_not_failure(){
 let source=r#"import {run,spawn,CommandOptions} from 'std:process';import {args,currentExe} from 'std:env';import {fromUtf8,length,toUtf8} from 'std:bytes';async function main():Task<void>{if(unwrap(args()).length>0){throw 'child failure';}const executable=unwrap(currentExe());const result=unwrap(await run(executable,['--child']));console.log(result.status.success,result.status.code!==undefined,length(result.stderr)>0);for(const bound of [-1]){match(await run('missing',[],{maxOutputBytes:bound})){Ok(_)=>{throw 'unexpected';},Err(error)=>console.log(error.code)}}match(await run('missing',[],{deadlineMs:-1})){Ok(_)=>{throw 'unexpected';},Err(error)=>console.log(error.code)}match(await run('missing',[],{stdin:'pipe'})){Ok(_)=>{throw 'unexpected';},Err(error)=>console.log(error.code)}const input=unwrap(fromUtf8('kept'));const options:CommandOptions={input};match(spawn('missing',[],options)){Ok(_)=>{throw 'unexpected';},Err(error)=>console.log(error.code)}const restored=unwrap(await run(__HELPER__,['echo'],options));console.log(unwrap(toUtf8(restored.stdout)));}"#;
 execute_helper(source,"false true true\nrange\nrange\noption\noption\nkept\n");
}
#[test]
fn unused_process_imports_and_error_data_do_not_activate_backend(){
 for source in ["import {run,spawn,Child} from 'std:process';function main(){}","import {ProcessError} from 'std:process';function inspect(error:ReadOnly<ProcessError>){console.log(error.code,error.operation,error.message,error.osCode);}function main(){}"]{let rust=check(source).unwrap();assert!(!rust.contains("mod tsrust_native_process"));}
}
#[test]
fn private_process_poll_bindings_are_not_importable(){assert!(check("import {__run} from 'std:process';function main(){}").is_err());}

fn helper(directory:&support::TempDirectory)->std::path::PathBuf{let source=directory.0.join("process-helper.rs");std::fs::write(&source,include_str!("../../../benchmarks/runtime/process-helper.rs")).unwrap();let binary=directory.0.join(format!("process-helper{}",std::env::consts::EXE_SUFFIX));support::success(&support::run(support::rustc().arg("--edition=2024").arg(&source).arg("-o").arg(&binary),60));binary}
fn execute_helper(source:&str,expected:&str){let directory=support::TempDirectory::new();let helper=helper(&directory);let source=source.replace("__HELPER__",&serde_json::to_string(helper.to_str().unwrap()).unwrap());let _=check(&source).unwrap();let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),180);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),expected);}
#[test]
fn process_binary_duplex_reader_eof_and_both_output_streams(){
 execute_helper(r#"import {run} from 'std:process';import {alloc,set,at,length} from 'std:bytes';import {reader} from 'std:io';async function main():Task<void>{const executable=__HELPER__;const bytes=unwrap(alloc(1048576));for(let i:int=0;i<1048576;i++){unwrap(set(bytes,i,i%251));}for(const mode of ['echo','eof']){const output=unwrap(await run(executable,[clone(mode)],{input:clone(bytes)}));console.log(output.status.success,length(output.stdout),length(output.stderr));for(let i:int=0;i<1048576;i++){if(unwrap(at(output.stdout,i))!==i%251){throw 'binary mismatch';}}}const input=reader(clone(bytes));const streamed=unwrap(await run(executable,['eof'],{input}));console.log(length(streamed.stdout));for(let i:int=0;i<1048576;i++){if(unwrap(at(streamed.stdout,i))!==i%251){throw 'reader mismatch';}}const output=unwrap(await run(executable,['flood','1048576','7']));console.log(length(output.stdout),length(output.stderr));for(let i:int=0;i<1048576;i++){if(unwrap(at(output.stdout,i))!==(i+7)%251||unwrap(at(output.stderr,i))!==(i*3+24)%251){throw 'flood mismatch';}}}"#,"true 1048576 0\ntrue 1048576 0\n1048576\n1048576 1048576\n");
}
#[test]
fn process_explicit_limits_kill_reap_and_completed_status(){
 execute_helper(r#"import {run,spawn as startChild} from 'std:process';async function main():Task<void>{const executable=__HELPER__;match(await run(executable,['sleep'],{deadlineMs:10})){Ok(_)=>{throw 'deadline ignored';},Err(error)=>console.log(error.code)}match(await run(executable,['flood','65536','7'],{maxOutputBytes:0})){Ok(_)=>{throw 'limit ignored';},Err(error)=>console.log(error.code)}const empty=unwrap(await run(executable,['silent'],{maxOutputBytes:0}));console.log(empty.status.success);const nonzero=unwrap(await run(executable,['exit','23']));console.log(nonzero.status.success,nonzero.status.code===23);const launch=startChild;const child=unwrap(launch(executable,['sleep'],{stdout:'null',stderr:'null'}));console.log(unwrap(child.tryWait())===undefined);unwrap(await child.kill());console.log(child.id()===undefined,unwrap(await child.wait()).success);}"#,"timeout\noutput-limit\ntrue\nfalse true\ntrue\ntrue false\n");
}
#[test]
fn process_cwd_environment_and_literal_argument_boundaries_are_child_local(){
 let directory=support::TempDirectory::new();std::fs::create_dir(directory.0.join("child")).unwrap();let helper=helper(&directory);
 let source=format!(r#"import {{run}} from 'std:process';import {{toUtf8}} from 'std:bytes';import {{currentDir as cwd,get}} from 'std:env';import {{join}} from 'std:path';async function main():Task<void>{{const executable={};const before=unwrap(cwd());const childCwd=join(before,'child');
 const cleared=unwrap(await run(executable,['context','a b','"quoted"','é'],{{cwd:clone(childCwd),clearEnv:true,env:[{{name:'TSRUST_PROCESS_CHILD',value:'local'}}]}}));console.log(unwrap(toUtf8(cleared.stdout))===childCwd+'\nlocal\n\n612062\n2271756f74656422\nc3a9\n');
 const inherited=unwrap(await run(executable,['context','a b','"quoted"','é'],{{cwd:clone(childCwd),env:[{{name:'TSRUST_PROCESS_CHILD',value:'local'}}]}}));console.log(unwrap(toUtf8(inherited.stdout))===childCwd+'\nlocal\nparent\n612062\n2271756f74656422\nc3a9\n');
 console.log(unwrap(cwd())===before,unwrap(get('TSRUST_PROCESS_CHILD'))===undefined,unwrap(get('TSRUST_PROCESS_PARENT'))==='parent');}}"#,serde_json::to_string(helper.to_str().unwrap()).unwrap());
 let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0).env_remove("TSRUST_PROCESS_CHILD").env("TSRUST_PROCESS_PARENT","parent"),180);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"true\ntrue\ntrue true true\n");
}
#[test]
fn process_package_relocates_and_builds_offline(){
 let directory=support::TempDirectory::new();let input=directory.source(&support::format_round_trip(include_str!("../../../examples/process/main.tsr")));let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));let manifest=std::fs::read_to_string(package.join("Cargo.toml")).unwrap();assert!(manifest.contains("\"process\""));let rust=std::fs::read_to_string(package.join("src/main.rs")).unwrap();std::fs::write(package.join("src/main.rs"),format!("#![forbid(unsafe_code)]\n{rust}")).unwrap();std::fs::remove_file(input).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let mut build=std::process::Command::new("rustup");build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);support::success(&support::run(&mut build,300));let output=support::run(std::process::Command::new(target.join(format!("debug/program{}",std::env::consts::EXE_SUFFIX))).current_dir(&relocated),30);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"true true\ntrue\ntrue true\ntrue\n");
}

#[test]
fn generated_process_tasks_keep_literal_context_and_protected_pipe_guards(){
 check("import {run,CommandOptions} from 'std:process';async function main():Task<void>{unwrap(await run('missing',[]));unwrap(await run('missing',[],{}));unwrap(await run('missing',[],{deadlineMs:10}));unwrap(await run('missing',[],undefined));const options:CommandOptions|undefined=undefined;unwrap(await run('missing',[],options));}").unwrap();
 let prefix="import {spawn} from 'std:process';import {writeAll,close} from 'std:io';import {fromUtf8} from 'std:bytes';async function main():Task<void>{const child=unwrap(spawn('missing',[],{stdin:'pipe'}));const input=child.takeStdin();if(input===undefined){throw 'no pipe';}const task=writeAll(input,unwrap(fromUtf8('x')));";
 check(&format!("{prefix}unwrap(await task);unwrap(await close(input));unwrap(await child.kill());}}")).unwrap();
 assert!(check(&format!("{prefix}unwrap(await close(input));unwrap(await task);}}")).is_err(),"guard transport must not release the pending input loan");
 assert!(check("import {Writer,writeAll} from 'std:io';import {fromUtf8} from 'std:bytes';function inspect(input:Writer|undefined):void{if(input===undefined){return;}const action=()=>writeAll(input,unwrap(fromUtf8('x')));action();}function main(){}").is_err(),"ordinary closures must not inherit caller guard facts");
}
