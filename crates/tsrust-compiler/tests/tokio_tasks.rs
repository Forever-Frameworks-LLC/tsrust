mod support;
use tsrust_compiler::check;
#[test]
fn scheduled_timer_completes_and_cancellation_settles(){
    execute("import {delay} from 'std:futures';async function work():Task<int>{await delay(1);return 42;}async function main():Task<void>{const job=work().start();console.log(await job);const cancelled=work().start();cancelled.cancel();try{await cancelled;}catch(error){console.log(error.code);}}","42\ncancelled\n",None);
}
#[test]
fn started_jobs_cannot_be_abandoned_or_cloned(){
    for body in ["const job=work().start();","const job=work().start();job.cancel();","const job=work().start();const copy=clone(job);await job;await copy;","const job=work().start();await job;await job;","const job=work().start();const owner={job:job};"]{
        let source=format!("async function work():Task<int>{{return 1;}}async function main():Task<void>{{{body}}}");
        assert!(check(&source).is_err(),"accepted {source}");
    }
}
#[test]
fn groups_settle_all_inputs_in_source_order(){
    execute("import {delay,allSettled,race} from 'std:futures';async function work(value:int):Task<int>{await delay(1);return value;}async function main():Task<void>{const results=await allSettled([work(1),work(2).start()]);for(const result of results){console.log(unwrap(clone(result)));}console.log(await race([work(3),work(4)]));}","1\n2\n3\n",None);
}
#[test]
fn async_channels_preserve_reusable_borrowed_endpoints(){
    execute("import {Channel} from 'std:futures';async function main():Task<void>{const channel=new Channel<int>(1);await channel.send(7);console.log(await channel.receive());await channel.send(9);channel.close();console.log(await channel.receive());try{await channel.receive();}catch(error){console.log(error.code);}}","7\n9\nclosed\n",None);
}

fn execute(source:&str,expected:&str,_:Option<()>){
    let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(&path),120);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}
#[test]
fn deadlines_empty_groups_and_typed_failures_execute(){
 execute("import {delay,race,allSettled} from 'std:futures';async function fail():Task<int>{throw 'original';}async function main():Task<void>{try{await delay(60000).timeout(0);}catch(error){console.log(error.code);}const jobs:Task<int>[]=[];console.log((await allSettled(jobs)).length);const empty:Task<int>[]=[];try{await race(empty);}catch(error){console.log(error.code);}try{await fail().start();}catch(error){console.log(error.code);console.log(error.cause);}}","timeout\n0\nempty\nworker\noriginal\n",None);
}
#[test]
fn borrowed_group_inputs_release_after_settlement(){
 execute(include_str!("../../../examples/tokio-tasks/channels.tsr"),"42\n",None);
}
#[test]
fn started_responsibility_survives_lazy_wrappers(){
 for body in ["const job=work().start();const deadline=job.timeout(10);","const group=allSettled([work().start()]);","const job=work().start();if(flag){await job;}"] {
  let source=format!("import {{allSettled}} from 'std:futures';async function work():Task<int>{{return 1;}}async function probe(flag:boolean):Task<void>{{{body}}}function main(){{}}");
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);
  assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("unsettled-job"),"{}",String::from_utf8_lossy(&output.stderr));
 }
}

#[test]
fn yield_and_empty_groups_use_concise_public_names(){
 execute("import {yield,allSettled,race} from 'std:futures';async function main():Task<void>{await yield();console.log((await allSettled([])).length);try{await race([]);}catch(error){console.log(error.code);}}","0\nempty\n",None);
}
#[test]
fn returned_jobs_and_borrowed_deadlines_keep_their_owners(){
 execute("import {Job,Channel} from 'std:futures';async function work():Task<int>{return 7;}async function parent():Task<Job<int,undefined>>{return work().start();}async function main():Task<void>{const child=await parent().start();console.log(await child);const queue=new Channel<int>(1);await queue.send(5);console.log(await queue.receive().timeout(1000));queue.close();}","7\n5\n",None);
}
#[test]
fn documented_future_examples_execute(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut seen=std::collections::BTreeSet::new();
 for family in ["std:futures","TaskMethods","Job","AsyncChannel","AsyncSender"] {
  for entry in catalog[family].as_array().unwrap(){
   let examples=entry["examples"].as_array().expect("future documentation needs executable examples");
   for example in examples {let source=example["source"].as_str().unwrap();if seen.insert(source){execute(source,example["output"].as_str().unwrap(),None);}}
  }
 }
}
#[test]
fn started_obligations_cover_exits_aggregates_and_collection_updates(){
 for body in [
 "work().start();",
 "const job=work().start();return;",
 "const job=work().start();throw 'stop';",
 "const owner:Owner={job:work().start()};",
 "const jobs:Job<int,undefined>[]=[work().start()];",
 "const jobs:Job<int,undefined>[]=[];jobs.unshift(work().start());",
 "const jobs:Job<int,undefined>[]=[];const removed=jobs.splice(0,0,work().start());await allSettled(removed);",
 "while(flag){const job=work().start();break;}",
 "const job=work().start();const consume=():void=>{const moved=job;};",
 ] {
  let source=format!("import {{Job,allSettled}} from 'std:futures';type Owner={{job:Job<int,undefined>;}};async function work():Task<int>{{return 1;}}async function probe(flag:boolean):Task<void>{{{body}}}function main(){{}}");
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);
  assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("unsettled-job"),"{body}: {}",String::from_utf8_lossy(&output.stderr));
 }
}
#[test]
fn cached_runtime_links_after_application_changes(){
 let directory=support::TempDirectory::new();
 for value in [42,43]{
  let source=support::format_round_trip(&format!("import {{yield}} from 'std:futures';async function main():Task<void>{{await yield();console.log({value});}}"));
  let input=directory.source(&source);let binary=directory.0.join(format!("program-{value}"));
  support::success(&support::run(support::cli().arg("build").arg(&input).arg("-o").arg(&binary).env("TSRUST_CACHE_DIR",directory.0.join("cache")),120));
  let output=support::run(&mut std::process::Command::new(binary),30);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),format!("{value}\n"));
 }
}
