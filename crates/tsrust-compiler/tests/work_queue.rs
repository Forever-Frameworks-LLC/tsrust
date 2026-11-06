mod support;
#[test]
fn bounded_workers_settle_and_closed_queue_drains(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/work-queue/main.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,b"bounded work queue accepted\n");
}
#[test]
fn queue_payload_and_receiver_loans_preserve_ownership(){
 for (source,code) in [
  "import {Channel} from 'std:futures';import {TrackedResource} from 'std:testing';function main(){const queue=new Channel<TrackedResource>(1);}",
  "import {Channel,Sender} from 'std:futures';function main(){const queue=new Channel<Sender<int>>(1);}",
  "import {Channel} from 'std:futures';async function main():Task<void>{const queue=new Channel<int>(1);const pending=queue.receive();queue.close();await pending;}",
  "import {Channel} from 'std:futures';type Work={text:string;};async function main():Task<void>{const queue=new Channel<Work>(1);const work:Work={text:'owned'};await queue.send(work);console.log(work.text);}",
 ].into_iter().zip(["channel-transfer","channel-transfer","capture-conflict","transferred"]){
  let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted {source}");assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),"{}",String::from_utf8_lossy(&output.stderr));
 }
}

#[test]
fn scheduling_keeps_grounded_generic_task_failure_contracts() {
 let source=r#"
import {allSettled} from 'std:futures';import {strictEqual} from 'std:testing';
async function value():Task<int>{const text='é'.repeat(2);function verify():void{strictEqual(text,'éé');}verify();return 7;}
async function failed():Task<int>{strictEqual('é'.repeat(2),'éé');throw 'checked cause';}
async function main():Task<void>{
 const results=await allSettled([value(),failed().start()]).timeout(5000);
 strictEqual(results.length,2);
 match(clone(results[0])){Ok(value)=>{strictEqual(value,7);},Err(_)=>{throw 'lost successful value';}}
 match(clone(results[1])){Ok(_)=>{throw 'lost checked failure';},Err(error)=>{strictEqual(error.code,'worker');if(typeof error.cause==='string'){strictEqual(error.cause,'checked cause');}else{throw 'lost checked cause';}}}
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 let output=support::run(support::cli().arg("run").arg(path),120);support::success(&output);assert!(output.stdout.is_empty());
}

#[test]
fn scheduling_inference_does_not_invent_unknown_callback_errors() {
 // Keep the existing unknown-origin contract negative alongside the grouped
 // task witness: collecting grounded origins must not turn absence into pure.
 let source="type Holder={run:ReadOnly<Function<int>>;};function use(holder:ReadOnly<Holder>):int{try{return holder.run();}catch(error){console.log(error.name);return 0;}}function main(){}";
 let error=tsrust_compiler::check(source).err().expect("no record supplies a callback origin");
 assert_eq!(error.code,"error-effect");
 assert!(error.message.contains("construct a record"),"{}",error.message);
}
