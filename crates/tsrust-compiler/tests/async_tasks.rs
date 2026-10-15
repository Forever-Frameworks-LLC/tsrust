mod support;
use tsrust_compiler::check;

#[test]
fn lazy_owned_tasks_and_nested_await_execute() {
    support::execute("async function work(text:string):Task<string>{console.log('running');return text;}async function nested():Task<string>{return await work('done');}async function main():Task<void>{const task=nested();console.log('created');console.log(await task);}","created\nrunning\ndone\n",None);
}
#[test]
fn borrow_remains_live_until_await_completes() {
    let source="async function size(text:ReadOnly<string>):Task<int>{return text.length;}async function main():Task<void>{let text='hello';const task=size(text);console.log(await task);text='ok';console.log(text);}";
    support::execute(source,"5\nok\n",None);
    let invalid=source.replace("console.log(await task);text='ok';","text='ok';console.log(await task);");
    assert!(check(&invalid).is_err());
}
#[test]
fn task_errors_propagate_at_await_and_finally_runs_normally() {
    support::execute("async function fail():Task<int>{try{throw 'failed';}finally{console.log('worker cleanup');}}async function main():Task<void>{const task=fail();console.log('created');try{console.log(await task);}catch(error){console.log(error);}finally{console.log('caller cleanup');}}","created\nworker cleanup\nfailed\ncaller cleanup\n",None);
}
#[test]
fn task_parameters_moves_and_records_execute() {
    support::execute("type Work={task:Task<int>;};async function one():Task<int>{return 1;}async function use(task:Task<int>):Task<int>{return await task;}async function main():Task<void>{const owner:Work={task:one()};console.log(await use(owner.task));}","1\n",None);
}
#[test]
fn task_ownership_and_context_rejections_are_source_diagnostics() {
    for source in [
        "async function one():Task<int>{return 1;}async function main():Task<void>{const task=one();console.log(await task);console.log(await task);}",
        "async function one():Task<int>{return 1;}function main(){console.log(await one());}",
        "async function one():Task<int>{return 1;}async function main():Task<void>{console.log(await 3);}",
        "async function one():Task<int>{return 1;}async function main():Task<void>{const task=one();task();}",
        "function fake():Task<int>{return ():int=>3;}function main(){}",
        "async function one():Task<int>{return 1;}async function main():Task<void>{const task=one();const other=task;console.log(await task);console.log(await other);}",
    ] {let source=support::format_round_trip(source);assert!(check(&source).is_err(),"accepted {source}");}
}
#[test]
fn unused_task_warnings_follow_reachable_paths_and_transfers() {
    let prefix="async function one():Task<int>{return 1;}";
    for body in ["one();","const task=one();","let task=one();task=one();await task;","const task=one();if(flag){await task;}","const task=one();if(flag){return; }await task;","const task=one();throw 'failure';","while(flag){const task=one();break;}","{const task=one();}{const task=one();await task;}"] {
        let source=format!("{prefix}async function probe(flag:boolean):Task<void>{{{body}}}function main(){{}}");
        let analysis=check(&support::format_round_trip(&source)).unwrap();assert!(!analysis.warnings().is_empty(),"missing warning: {source}");
    }
    for body in ["const task=one();await task;","const task=one();if(flag){await task;}else{await task;}","const task=one();const moved=task;await moved;","const task=one();try{throw 'recover';}catch(error){console.log(error);}finally{await task;}"] {
        let source=format!("{prefix}async function probe(flag:boolean):Task<void>{{{body}}}function main(){{}}");
        assert!(check(&support::format_round_trip(&source)).unwrap().warnings().is_empty(),"false warning: {source}");
    }
}

#[test]
fn generated_future_retains_allocation_and_drops_once_across_real_suspension() {
    let source=support::format_round_trip("async function hold(text:string):Task<string>{return text;}function main(){}");
    let mut rust=check(&source).unwrap().emit_rust();
    assert!(rust.contains("return value0;"),"generated owned return witness changed");
    rust=rust.replace("return value0;","YieldOnce(false).await;return value0;");
    rust=rust.replace("struct Closure0 {","struct Closure0 { local_only: std::rc::Rc<i64>,").replace("Closure0 { capture0:","Closure0 { local_only:std::rc::Rc::new(1),capture0:");
    let opening=rust.find("fn main() -> () {").expect("native main")+"fn main() -> () {".len();
    rust.insert_str(opening,"generated_task_witness();");
    rust.push_str(r#"
use std::sync::atomic::{AtomicUsize,Ordering};
static TRACKED:AtomicUsize=AtomicUsize::new(0);
static DROPS:AtomicUsize=AtomicUsize::new(0);
struct AllocationTracker;
unsafe impl std::alloc::GlobalAlloc for AllocationTracker {
 unsafe fn alloc(&self,layout:std::alloc::Layout)->*mut u8 {unsafe{std::alloc::System.alloc(layout)}}
 unsafe fn dealloc(&self,pointer:*mut u8,layout:std::alloc::Layout){
  if TRACKED.compare_exchange(pointer as usize,0,Ordering::SeqCst,Ordering::SeqCst).is_ok(){DROPS.fetch_add(1,Ordering::SeqCst);}
  unsafe{std::alloc::System.dealloc(pointer,layout)}
 }
}
#[global_allocator]static ALLOCATOR:AllocationTracker=AllocationTracker;
struct YieldOnce(bool);
impl std::future::Future for YieldOnce {
 type Output=();
 fn poll(mut self:std::pin::Pin<&mut Self>,context:&mut std::task::Context<'_>)->std::task::Poll<()> {
  if self.0 {std::task::Poll::Ready(())}else{self.0=true;context.waker().wake_by_ref();std::task::Poll::Pending}
 }
}
struct WakeCount(AtomicUsize);
impl std::task::Wake for WakeCount {fn wake(self:std::sync::Arc<Self>){self.0.fetch_add(1,Ordering::SeqCst);}}
fn generated_task_witness(){
 use std::future::Future;
 let wake=std::sync::Arc::new(WakeCount(AtomicUsize::new(0)));let waker=std::task::Waker::from(wake.clone());let mut context=std::task::Context::from_waker(&waker);
 for phase in 0..3 {
  DROPS.store(0,Ordering::SeqCst);
  let text=std::string::String::from("task-owned allocation");let pointer=text.as_ptr() as usize;TRACKED.store(pointer,Ordering::SeqCst);
  let task=function0(String::Owned(text));let mut future=Box::pin(task.call());
  if phase>0 {assert!(future.as_mut().poll(&mut context).is_pending());assert_eq!(DROPS.load(Ordering::SeqCst),0);}
  if phase==2 {
   let output=match future.as_mut().poll(&mut context){std::task::Poll::Ready(value)=>value,_=>panic!("future did not complete")};
   assert_eq!(output.as_ref().as_ptr() as usize,pointer);drop(future);assert_eq!(DROPS.load(Ordering::SeqCst),0);drop(output);
  }else{drop(future);}
  assert_eq!(DROPS.load(Ordering::SeqCst),1);
 }
 assert_eq!(wake.0.load(Ordering::SeqCst),2);
}
"#);
    let directory=support::TempDirectory::new();let input=directory.0.join("task.rs");let binary=directory.0.join("task");std::fs::write(&input,rust).unwrap();
    let compiled=support::run(support::rustc().args(["--edition=2024"]).arg(&input).arg("-o").arg(&binary),60);support::success(&compiled);
    support::success(&support::run(&mut std::process::Command::new(binary),10));
}

#[test]
fn defaults_evaluate_at_construction_and_generic_tasks_keep_their_type() {
    support::execute("function label():string{console.log('default');return 'world';}async function greet(text:string=label()):Task<string>{console.log('body');return text;}async function identity<T>(value:T):Task<T>{return value;}async function main():Task<void>{const task=greet();console.log('constructed');console.log(await task);console.log(await identity(42));}","default\nconstructed\nbody\nworld\n42\n",None);
}
#[test]
fn fallible_task_fields_keep_original_errors() {
    support::execute("type Work={task:Task<int>;};async function fail():Task<int>{throw 'field failure';}async function main():Task<void>{const work:Work={task:fail()};try{console.log(await work.task);}catch(error){console.log(error);}}","field failure\n",None);
}
#[test]
fn unused_task_cli_warning_does_not_fail_and_suppression_only_hides_warning() {
    let source="async function one():Task<int>{return 1;}async function main():Task<void>{one();}";
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("check").arg(&path),30);support::success(&output);assert!(String::from_utf8_lossy(&output.stderr).contains("warning[unused-task]"));
    let output=support::run(support::cli().arg("run").arg(&path),30);support::success(&output);assert!(String::from_utf8_lossy(&output.stderr).contains("warning[unused-task]"));assert!(output.stdout.is_empty());
    let invalid="async function one():Task<int>{return 1;}async function main():Task<void>{const task=one();await task;\n// @tsrust ignore - intentional ownership rejection\nawait task;}";
    assert!(check(invalid).is_err());
}

#[test]
fn mutable_loans_conditional_tasks_and_awaited_cleanup_execute() {
    support::execute("async function add(values:Mutable<int[]>):Task<void>{values.push(2);}async function one():Task<int>{return 1;}async function two():Task<int>{return 2;}async function pick(flag:boolean):Task<int>{const task=flag?one():two();try{return await task;}finally{console.log(await one());}}async function main():Task<void>{const values=[1];const pending=add(values);await pending;console.log(values.length);console.log(await pick(false));}","2\n1\n2\n",None);
    assert!(check("async function one():Task<int>{return 1;}async function invalid(value:int=await one()):Task<int>{return value;}function main(){}").is_err());
}
#[test]
fn named_and_default_async_imports_execute_with_retained_project_spans() {
    let directory=support::TempDirectory::new();
    std::fs::write(directory.0.join("work.tsr"),support::format_round_trip("export async function one():Task<int>{return 1;}export default async function two():Task<int>{return 2;}")).unwrap();
    let source=support::format_round_trip("import two,{one} from './work.tsr';async function main():Task<void>{console.log(await one());console.log(await two());}");
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);support::success(&output);assert_eq!(output.stdout,b"1\n2\n");
}

#[test]
fn aggregate_task_moves_keep_independent_warning_obligations() {
    let prefix="type Work={left:Task<int>;right:Task<int>;};async function one():Task<int>{return 1;}";
    for body in ["const first:Work={left:one(),right:one()};const second=first;await second.left;await second.right;","const owner:Work={left:one(),right:one()};const left=owner.left;await left;await owner.right;"] {
        let source=format!("{prefix}async function main():Task<void>{{{body}}}");assert!(check(&support::format_round_trip(&source)).unwrap().warnings().is_empty(),"{source}");
    }
    let source=format!("{prefix}async function main():Task<void>{{const first:Work={{left:one(),right:one()}};const second=first;await second.left;}}");assert!(!check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
}

#[test]
fn inferred_task_arrays_transfer_through_pop_and_push() {
    support::execute("async function one():Task<int>{return 1;}async function main():Task<void>{const tasks=[one()];tasks.push(one());console.log(await tasks.pop()!);console.log(await tasks.pop()!);}","1\n1\n",None);
}

#[test]
fn nested_task_cannot_return_a_loan_of_its_parent_owned_input() {
    let borrowed="async function size(text:ReadOnly<string>):Task<int>{return text.length;}async function invalid(text:ReadOnly<string>):Task<Task<int>>{return size(text);}function main(){}";
    assert!(check(&support::format_round_trip(borrowed)).is_err());
    let source="async function size(text:ReadOnly<string>):Task<int>{return text.length;}async function invalid(text:string):Task<Task<int>>{return size(text);}function main(){}";
    assert!(check(&support::format_round_trip(source)).is_err());
    support::execute("async function one():Task<int>{return 1;}async function outer():Task<Task<int>>{return one();}async function main():Task<void>{const inner=await outer();console.log(await inner);}","1\n",None);
}

#[test]
fn owned_parameter_cannot_escape_as_a_task_loan() {
    let source="async function size(text:ReadOnly<string>):Task<int>{return text.length;}function invalid(text:Owned<string>):Task<int>{return size(text);}function main(){}";
    assert!(check(&support::format_round_trip(source)).is_err());
}
#[test]
fn call_failure_paths_keep_task_responsibility_and_finally_can_handle_it() {
    let prefix="async function one():Task<int>{return 1;}function risky():void{throw 'failure';}";
    let source=format!("{prefix}async function main():Task<void>{{const task=one();risky();await task;}}");
    assert!(!check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
    let source=format!("{prefix}async function main():Task<void>{{const task=one();try{{risky();}}finally{{await task;}}}}");
    assert!(check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
}

#[test]
fn throwing_later_argument_can_discard_an_earlier_task_temporary() {
    let source="async function one():Task<int>{return 1;}function risky():int{throw 'failure';}async function consume(task:Task<int>,value:int):Task<int>{return (await task)+value;}async function main():Task<void>{await consume(one(),risky());}";
    assert!(!check(&support::format_round_trip(source)).unwrap().warnings().is_empty());
}

#[test]
fn task_returned_by_await_keeps_its_own_responsibility() {
    let prefix="async function one():Task<int>{return 1;}async function outer():Task<Task<int>>{return one();}";
    for body in ["const inner=await outer();", "await outer();"] {
        let source=format!("{prefix}async function main():Task<void>{{{body}}}");
        assert!(!check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
    }
    let source=format!("{prefix}async function main():Task<void>{{const inner=await outer();await inner;}}");
    assert!(check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
}

#[test]
fn task_match_results_and_discarded_sequence_operands_keep_responsibility() {
    let prefix="async function one():Task<int>{return 1;}";
    for body in ["const task=match(true){true=>one(),false=>one()};", "await (one(),one());"] {
        let source=format!("{prefix}async function main():Task<void>{{{body}}}");
        assert!(!check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
    }
    let source=format!("{prefix}async function main():Task<void>{{const task=match(true){{true=>one(),false=>one()}};await task;}}");
    assert!(check(&support::format_round_trip(&source)).unwrap().warnings().is_empty());
    support::execute(&format!("{prefix}async function main():Task<void>{{const task=match(true){{true=>one(),false=>one()}};console.log(await task);}}"),"1\n",None);
}

#[test]
fn async_emission_preserves_strings_that_look_like_generated_methods() {
    support::execute("async function text():Task<string>{return 'fn call OnceCallback< fn invoke_once self.call() }';}async function main():Task<void>{console.log(await text());}","fn call OnceCallback< fn invoke_once self.call() }\n",None);
}

#[test]
fn task_parameter_contract_compiles_without_a_concrete_async_declaration() {
    support::execute("function accept(task:Task<int>):void{}function main(){}","",None);
}

#[test]
fn awaited_values_infer_generic_arguments_and_keep_record_and_native_view_loans() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip(include_str!("fixtures/async-tasks/borrowed-data.tsr"));
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);support::success(&output);assert!(output.stdout.is_empty());
    for point in ["strictEqual(await userTask, 3);"] {
        let invalid=source.replace(point,&format!("take(user); {point}"));
        let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(&invalid))),30);
        assert!(!output.status.success(),"A task transfer must retain its caller loan: {point}");
    }
}

#[test]
fn tasks_returning_multiple_input_borrows_name_their_lifetimes() {
    support::execute("async function size(left:ReadOnly<string>,right:ReadOnly<string>):Task<int>{return left.length+right.length;}async function change(left:Mutable<int[]>,right:ReadOnly<int[]>):Task<void>{left.push(right.length);}async function main():Task<void>{let left='one';let right='two';size(left,right);console.log(await size(left,right));left='x';right='y';console.log(await size(left,right));const values=[1];const input=[2,3];await change(values,input);console.log(values.length);}","6\n2\n2\n",None);
}
