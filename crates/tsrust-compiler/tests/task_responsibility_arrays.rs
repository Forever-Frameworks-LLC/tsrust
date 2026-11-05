//! Populated collection responsibilities survive insertion and settle at group await.
mod support;
use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
fn checked(source:&str)->Result<Vec<String>,String>{
 let directory=support::TempDirectory::new();let path=directory.source(source);let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
 check_graph(graph).map(|checked|checked.warnings().into_iter().map(|warning|warning.code.to_owned()).collect()).map_err(|error|error.code.to_owned())
}
#[test]
fn populated_homogeneous_task_job_and_unit_arrays_settle_without_false_warnings(){
 for operation in ["all","allSettled","race"]{for(kind,output,invocation)in[("Task<int>","int","value(3)"),("Job<int,undefined>","int","value(3).start()"),("Task<void>","void","unit()")]{
  let source=format!("import {{all,allSettled,race,Job}} from 'std:futures';async function value(n:int):Task<int>{{return n;}}async function unit():Task<void>{{}}async function main():Task<void>{{const inputs:{kind}[]=[];inputs.push({invocation});inputs.push({invocation});await {operation}(inputs);console.log('settled');}} ");
  let warnings=checked(&source).unwrap_or_else(|error|panic!("{operation}/{output}/{kind}: {error}"));assert!(warnings.is_empty(),"{operation}/{kind}: {warnings:?}");
 }}
}
#[test]
fn abandoned_job_arrays_and_throwing_push_arguments_still_require_cleanup(){
 let prefix="import {all,Job} from 'std:futures';async function value(n:int):Task<int>{return n;}function fail():Job<int,undefined>{throw 'argument failed';}";
 for body in [
  "const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());",
  "const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());jobs.push(fail());await all(jobs);",
  "const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());if(flag){await all(jobs);}",
 ]{let source=format!("{prefix}async function probe(flag:boolean):Task<void>{{{body}}}function main(){{}}");assert_eq!(checked(&source).unwrap_err(),"unsettled-job","{body}");}
 let source=format!("{prefix}async function main():Task<void>{{const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());try{{jobs.push(fail());}}finally{{await all(jobs);}}}}");
 assert!(checked(&source).unwrap().is_empty());
}
#[test]
fn abandoned_lazy_task_arrays_still_warn(){
 let source="async function value():Task<int>{return 3;}async function main():Task<void>{const tasks:Task<int>[]=[];tasks.push(value());}";
 assert!(checked(source).unwrap().iter().any(|code|code=="unused-task"));
}
#[test]
fn populated_arrays_execute_all_in_order_and_without_unused_task_warnings(){
 let source="import {all,Job} from 'std:futures';async function value(n:int):Task<int>{return n;}async function unit():Task<void>{}async function main():Task<void>{const tasks:Task<int>[]=[];tasks.push(value(3));tasks.push(value(4));const taskValues=await all(tasks);console.log(taskValues[0],taskValues[1]);const jobs:Job<int,undefined>[]=[];jobs.push(value(5).start());jobs.push(value(6).start());const jobValues=await all(jobs);console.log(jobValues[0],jobValues[1]);const units:Task<void>[]=[];units.push(unit());units.push(unit());console.log((await all(units)).length);}";
 let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let input=directory.source(&source);
 let result=support::run(support::cli().arg("run").arg(input),120);support::success(&result);assert_eq!(result.stdout,b"3 4\n5 6\n2\n");assert!(!String::from_utf8_lossy(&result.stderr).contains("unused-task"),"{}",String::from_utf8_lossy(&result.stderr));
}
