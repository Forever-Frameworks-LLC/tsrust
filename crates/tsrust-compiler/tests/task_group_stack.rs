mod support;
fn emitted(source:&str)->String{
    let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
    let output=support::run(support::cli().arg("emit").arg(path),30);support::success(&output);String::from_utf8(output.stdout).unwrap()
}
fn execute(source:&str){
    let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path),180));
}
#[test]
fn literal_all_stack_pins_follow_every_checked_arity(){
    for width in [1,2,5,17]{
        let tasks=(0..width).map(|index|format!("value({index})")).collect::<Vec<_>>().join(",");
        let source=format!("import {{all,yield}} from 'std:futures';import {{strictEqual}} from 'std:testing';async function value(n:int):Task<int>{{await yield();return n;}}async function main():Task<void>{{const values=await all([{tasks}]);strictEqual(values.length,{width});let index=0;for(const value of values){{strictEqual(value,index);index++;}}}}");
        let rust=emitted(&source);assert_eq!(rust.matches("std::pin::pin!(Some(all_task").count(),width);
        assert!(!rust.contains("Box::pin(async move"));assert!(!rust.contains("unsafe"));
        execute(&source);
    }
}
#[test]
fn literal_all_keeps_borrows_jobs_units_errors_and_dynamic_arrays(){
    let source=r#"import {all,yield} from 'std:futures';import {strictEqual} from 'std:testing';
async function read(value:ReadOnly<string>):Task<int>{await yield();return value.length;}
async function value(n:int):Task<int>{await yield();return n;}
let built=0;
function construct(n:int):Task<int>{built=built*10+n;return value(n);}
async function unit():Task<void>{await yield();}
async function fail():Task<int>{throw 'original';}
async function main():Task<void>{
 const owner='owned'.repeat(2);const values=await all([read(owner),value(7).start()]);strictEqual(values[0],10);strictEqual(values[1],7);strictEqual(owner.length,10);
 const once=all([construct(1),construct(2),construct(3)]);strictEqual(built,123);const constructed=await once;strictEqual(built,123);strictEqual(constructed[0],1);strictEqual(constructed[2],3);
 const units:void[]=await all([unit(),unit().start(),unit()]);strictEqual(units.length,3);strictEqual((await all([])).length,0);
 const tasks=[value(3),value(4)];const dynamic=await all(tasks);strictEqual(dynamic[0],3);strictEqual(dynamic[1],4);
 let failed=false;try{await all([value(1),fail(),value(3)]);}catch(error){if(typeof error.cause==='string'){strictEqual(error.cause,'original');failed=true;}else{throw 'wrong cause';}}strictEqual(failed,true);
}"#;
    let rust=emitted(source);assert!(rust.contains(".tsr_owned_items().map(|task|async move"));assert!(rust.contains("all_scope.drain(false).await;all_result"));execute(source);
}
#[test]
fn race_and_all_settled_keep_existing_owned_adapters(){
    let source="import {race,allSettled} from 'std:futures';async function value(n:int):Task<int>{return n;}async function main():Task<void>{await race([value(1),value(2)]);await allSettled([value(3),value(4)]);}";
    let rust=emitted(source);assert!(!rust.contains("all_inputs"));assert!(rust.contains("Box::pin(async move"));assert!(rust.contains("tsrust_native_futures::race(vec!"));assert!(rust.contains("tsrust_native_futures::all_settled(vec!"));
}
#[test]
fn emitted_stack_body_drops_results_then_pending_inputs_in_source_order(){
    let source="import {all} from 'std:futures';async function value(n:int):Task<int>{return n;}async function main():Task<void>{await all([value(0),value(1),value(2),value(3)]);}";
    let rust=emitted(source);let start=rust.find("let all_scope=tsrust_native_futures::Scope::group();").unwrap();let suffix="all_scope.drain(false).await;all_result}";
    let end=start+rust[start..].find(suffix).unwrap()+suffix.len();let body=&rust[start..end];
    let mut previous=0;for index in (0..4).rev(){let position=body.find(&format!("let mut all_future{index}=")).unwrap();assert!(position>previous);previous=position;}
    let mut previous=0;for index in 0..4{let position=body.find(&format!("if let Some(future)=all_future{index}")).unwrap();assert!(position>previous);previous=position;}
    let directory=support::TempDirectory::new();let group=directory.0.join("group.rs");
    let inputs=(0..4).map(|_|"ProbeFuture<'a>").collect::<Vec<_>>().join(",");
    // Test the actual emitted pin/poll/scope body with observable !Unpin inputs.
    std::fs::write(&group,format!("async fn probe<'a>(inputs:({inputs}))->Result<Vec<ProbeValue>,tsrust_native_futures::Failure<()>>{{let all_inputs=inputs;{body}")).unwrap();
    let runtime=include_str!("../src/stdlib/futures/runtime.rs");let closing=runtime.rfind('}').unwrap();let runtime_path=directory.0.join("runtime.rs");
    std::fs::write(&runtime_path,format!("{}pub(crate) fn probe_scope()->usize{{SCOPE.with(|scope|std::sync::Arc::as_ptr(scope) as usize)}}{}",&runtime[..closing],&runtime[closing..])).unwrap();
    let input=directory.0.join("witness.rs");std::fs::write(&input,include_str!("fixtures/task-group-stack/drop-order.rs")).unwrap();let binary=directory.0.join(format!("witness{}",std::env::consts::EXE_SUFFIX));
    let mut command=support::rustc();command.args(["--edition=2024","-F","unsafe_code","-C","overflow-checks=on"]).arg(&input).arg("-o").arg(&binary).env("TSRUST_STACK_RUNTIME",&runtime_path).env("TSRUST_STACK_GROUP",&group);
    support::success(&support::run(&mut command,120));support::success(&support::run(&mut std::process::Command::new(binary),30));
}
