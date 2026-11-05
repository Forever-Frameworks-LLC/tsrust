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

#[test]
fn nested_awaited_group_members_and_indices_evaluate_their_responsibilities(){
 for(kind,invocation)in[("Task<int>","value(3)"),("Job<int,undefined>","value(3).start()"),("Task<void>","unit()")]{
  let source=format!("import {{all,allSettled,Job}} from 'std:futures';async function value(n:int):Task<int>{{return n;}}async function unit():Task<void>{{}}async function main():Task<void>{{const tasks:{kind}[]=[];tasks.push({invocation});console.log((await all(tasks)).length);const others:{kind}[]=[];others.push({invocation});console.log((await allSettled(others)).length);}}");assert!(checked(&source).unwrap().is_empty(),"{kind}");
 }
 let prefix="import {all,Job} from 'std:futures';async function value(n:int):Task<int>{return n;}function fail():int{throw 'argument failed';}";
 for expression in ["const pending=all(jobs);console.log((await pending).length);","console.log((await all(jobs))[0]);","console.log((await all(jobs))[fail()]);","console.log((await all(jobs)).length,fail());"]{
  let source=format!("{prefix}async function main():Task<void>{{const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());{expression}}}");assert!(checked(&source).unwrap().is_empty(),"{expression}");
 }
 let source=format!("{prefix}async function main():Task<void>{{const jobs:Job<int,undefined>[]=[];jobs.push(value(3).start());console.log(fail(),(await all(jobs)).length);}}");assert_eq!(checked(&source).unwrap_err(),"unsettled-job");
}

#[test]
fn named_place_indices_evaluate_awaits_mutations_and_failure_order_once(){
 let prefix="import {Job} from 'std:futures';async function index():Task<int>{return 0;}function fail():int{throw 'index failed';}";
 for body in [
  "const data:int[]=[7];const position=index();console.log(data[await position]);",
  "const data:int[]=[7];const position=index().start();console.log(data[await position]);",
  "const data:int[]=[7];const position=index().start();const selected=data[await position];console.log(selected);",
  "const data:int[]=[7];const position=index().start();data[await position]=8;",
  "const holder={rows:[[7]]};const position=index().start();console.log(holder.rows[await position][0]);",
  "const data:string[]=['seven'];const position=index().start();console.log(data[await position].toUpperCase());",
  "const data:int[]=[7];const position=index().start();console.log(data[(await position,fail())]);",
 ]{let source=format!("{prefix}async function main():Task<void>{{{body}}}");assert!(checked(&source).unwrap().is_empty(),"{body}");}
 for body in [
  "const data:int[]=[7];const jobs:Job<int,undefined>[]=[];console.log(data[(jobs.push(index().start()),0)]);",
  "const data:string[]=['seven'];const jobs:Job<int,undefined>[]=[];console.log(data[(jobs.push(index().start()),0)].toUpperCase());",
  "const data:int[]=[7];const jobs:Job<int,undefined>[]=[];const selected=data[(jobs.push(index().start()),0)];console.log(selected);",
  "const data:int[]=[7];const jobs:Job<int,undefined>[]=[];data[(jobs.push(index().start()),0)]=8;",
  "const data:int[][]=[[]];const jobs:Job<int,undefined>[]=[];data[(jobs.push(index().start()),0)].push(1);",
  "const data:int[]=[7];const position=index().start();console.log(data[(fail(),await position)]);",
  "const holder={rows:[[7]]};const position=index().start();console.log(holder.rows[fail()][await position]);",
 ]{let source=format!("{prefix}async function main():Task<void>{{{body}}}");assert_eq!(checked(&source).unwrap_err(),"unsettled-job","{body}");}
}
