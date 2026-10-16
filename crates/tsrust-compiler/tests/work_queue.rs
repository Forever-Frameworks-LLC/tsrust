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
