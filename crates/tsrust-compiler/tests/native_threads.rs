mod support;


#[test]
fn owned_worker_result_and_throw_are_joined() {
    let directory = support::TempDirectory::new();
    let source = r#"
import {spawn,join,detach} from 'std:threads';
function main(){
    const started=spawn(():int=>42);
    match(started){Ok(worker)=>{match(join(worker)){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.code);}}},Err(error)=>{console.log(error.code);}}
    const failed=spawn(():int=>{throw 'worker failure';});
    match(failed){Ok(worker)=>{match(join(worker)){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.code);console.log(error.error);}}},Err(error)=>{console.log(error.code);}}
    match(spawn(():void=>{})){Ok(worker)=>{detach(worker);},Err(error)=>{console.log(error.code);}}
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"42\nworker\nworker failure\n");
}

#[test]
fn lifecycle_rejects_unhandled_results_and_unsupported_storage() {
    for body in [
        "spawn(():int=>1);",
        "match(spawn(():int=>1)){Ok(worker)=>{const bad=[worker];},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{const abandoned=():void=>{detach(worker);};},Err(error)=>{}}",
    ] {
        let directory=support::TempDirectory::new();
        let source=format!("import {{spawn,detach}} from 'std:threads';function main(){{{body}}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted: {source}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("thread-lifecycle")||error.contains("thread-storage"),"{source}\n{error}");
    }
}

#[test]
fn worker_inputs_must_be_owned_and_handles_are_single_use() {
    for body in [
        "const text='borrowed';const callback=():int=>text.length;match(spawn(callback)){Ok(worker)=>{detach(worker);},Err(error)=>{}}console.log(text);",
        "match(spawn(():int=>1)){Ok(worker)=>{detach(worker);detach(worker);},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{flag && (detach(worker),true);},Err(error)=>{}}",
    ] {
        let directory=support::TempDirectory::new();
        let source=format!("import {{spawn,join,detach}} from 'std:threads';function probe(flag:boolean){{{body}}}function main(){{probe(true);}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted: {source}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(!error.contains("panicked"),"{error}");
    }
}

#[test]
fn owned_captures_handle_transfer_and_error_unions_execute() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {spawn,join,detach,Thread,SpawnError} from 'std:threads';
function start():Result<Thread<int,undefined>,SpawnError>{return spawn(():int=>23);}
function branches(flag:boolean){match(start()){Ok(worker)=>{if(flag){detach(worker);}else{detach(worker);}},Err(error)=>{throw error.code;}}}
function factory(text:string):Function<string>{return ():string=>text;}
function settle(worker:Thread<string,undefined>):string{
    match(join(worker)){Ok(text)=>{return text;},Err(error)=>{return error.code;}}
}
function main(){
    branches(true);branches(false);
    match(start()){Ok(worker)=>{match(join(worker)){Ok(value)=>{console.log(value);},Err(error)=>{throw error.code;}}},Err(error)=>{throw error.code;}}
    match(spawn(factory('owned payload'))){Ok(worker)=>{console.log(settle(worker));},Err(error)=>{throw error.code;}}
    const flag=true;
    match(spawn(():int=>{if(flag){throw 17;}throw 'other';})){
        Ok(worker)=>{match(join(worker)){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.code);console.log(error.error);}}},
        Err(error)=>{throw error.code;}
    }
    match(spawn(():int=>{const values=[1];return values[9];})){
        Ok(worker)=>{match(join(worker)){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.code);}}},
        Err(error)=>{throw error.code;}
    }
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"23\nowned payload\nworker\n17\nworker\n");
}

#[test]
fn void_workers_join_and_discarded_module_owners_reject() {
    let directory=support::TempDirectory::new();
    let source="import {spawn,join} from 'std:threads';import {strictEqual} from 'std:testing';function main(){match(spawn(():void=>{})){Ok(worker)=>{match(join(worker)){Ok(_)=>{strictEqual(1,1);},Err(error)=>{throw error.code;}}},Err(error)=>{throw error.code;}}}";
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for source in [
        "import {spawn} from 'std:threads';function main(){const value=(spawn(():int=>1),2);}",
    ] {
        let path=directory.source(&support::format_round_trip(source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("thread-lifecycle"));
    }
}

#[test]
fn scoped_cleanup_covers_normal_return_throw_and_loop_exits() {
    let directory = support::TempDirectory::new();
    let source = r#"
import {spawn,join,detach} from 'std:threads';
import {strictEqual} from 'std:testing';
function early(){const worker=unwrap(spawn(():void=>{console.log('returned');}));return;}
function failing(){const worker=unwrap(spawn(():void=>{console.log('cleaned');throw 'worker error';}));throw 'caller error';}
function main(){
    {const worker=unwrap(spawn(():void=>{console.log('normal');}));}
    console.log('after normal');
    early();console.log('after return');
    try{failing();}catch(error){if(typeof error==='string'){strictEqual(error,'caller error');console.log(error);}else{throw error;}}finally{console.log('finally');}
    for(let i=0;i<2;i+=1){const worker=unwrap(spawn(():void=>{console.log('iteration');}));if(i===0){continue;}break;}
    console.log('after loop');
    const done=unwrap(spawn(():void=>{}));unwrap(join(done));
    const detached=unwrap(spawn(():void=>{}));detach(detached);
    try{const worker=unwrap(spawn(():int=>{throw 'original';}));unwrap(join(worker));}catch(error){console.log(error.code);}
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"normal\nafter normal\nreturned\nafter return\ncleaned\ncaller error\nfinally\niteration\niteration\nafter loop\nworker\n");
}

#[test]
fn scoped_workers_overlap_before_join_without_timing_a_speedup() {
    let directory = support::TempDirectory::new();
    // Each worker must see the other's marker before either can complete.
    // Sequential execution fails the deadline instead of appearing to overlap.
    let source = r#"
import {spawn,join} from 'std:threads';
import {writeFile,readFile} from 'std:fs';
import {fromUtf8} from 'std:bytes';
import {monotonicNow} from 'std:time';
import {strictEqual} from 'std:testing';
function rendezvous(own:string,other:string):int{
    const bytes=unwrap(fromUtf8('ready'));unwrap(writeFile(own,bytes));
    const start=monotonicNow();
    while(monotonicNow()-start<10000.0){match(readFile(other)){Ok(bytes)=>{return 21;},Err(error)=>{}}}
    throw 'workers did not overlap';
}
function main(){
    const first=unwrap(spawn(():int=>rendezvous('first.ready','second.ready')));
    const second=unwrap(spawn(():int=>rendezvous('second.ready','first.ready')));
    const a=unwrap(join(first));const b=unwrap(join(second));strictEqual(a+b,42);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),60);
    support::success(&output);
}

#[test]
fn unwrap_preserves_owned_and_union_errors_and_scoped_moves_remain_linear() {
    let directory = support::TempDirectory::new();
    let source=r#"
import {spawn,join,Thread,ThreadError} from 'std:threads';
import {strictEqual} from 'std:testing';
function result(flag:boolean):Result<string,string|int>{if(flag){return Ok('owned');}return Err('failure');}
function take(worker:Thread<int,undefined>):int{return unwrap(join(worker));}
function main(){
    strictEqual(unwrap(result(true)),'owned');
    const values=unwrap(Ok<int[],string>([1,2]));values.push(3);strictEqual(values.length,3);
    unwrap(Ok<void,string>());
    try{unwrap(Err<int,string>('direct'));}catch(error){strictEqual(error,'direct');}
    try{unwrap(result(false));}catch(error){strictEqual(String(error),'failure');}
    const worker=unwrap(spawn(():int=>42));strictEqual(take(worker),42);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for body in [
        "unwrap(42);",
        "const operation=unwrap;",
        "const worker=unwrap(spawn(():int=>42));join(worker);join(worker);",
        "const worker=unwrap(spawn(():int=>42));const workers=[worker];",
        "const worker=unwrap(spawn(():int=>42));const cb=():void=>{join(worker);};",
        "const text='borrowed';const cb=():int=>text.length;const worker=unwrap(spawn(cb));console.log(text);",
    ] {
        let source=format!("import {{spawn,join}} from 'std:threads';function main(){{{body}}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted {source}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
fn scoped_example_without_explicit_throw_activates_typed_transport() {
    let directory=support::TempDirectory::new();
    let source=include_str!("../../../examples/scoped-workers/main.tsr");
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);assert_eq!(output.stdout,b"25205\n");
}

#[test]
fn default_spawn_cleans_up_ignored_success_replacement_and_fallible_work() {
    let directory=support::TempDirectory::new();
    let source=r#"
import {spawn,join} from 'std:threads';
import {strictEqual} from 'std:testing';
function main(){
    match(spawn(():void=>{console.log('ignored');})){Ok(_)=>{},Err(error)=>{throw error;}}
    console.log('after ignored');
    {
        let worker=unwrap(spawn(():void=>{console.log('replaced');}));
        const silent=false;
        worker=unwrap(spawn(():void=>{if(silent){console.log('unused');}}));
        console.log('after replacement');
    }
    try{
        const worker=unwrap(spawn(():void=>{console.log('bounds cleanup');}));
        const values=[1];console.log(values[9]);
        unwrap(join(worker));
    }catch(error){console.log('after failure');}
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"ignored\nafter ignored\nreplaced\nafter replacement\nbounds cleanup\nafter failure\n");
}

#[test]
fn global_unwrap_preserves_user_defined_names() {
    for source in [
        "function unwrap(value:int):int{return value+1;}function main(){unwrap(2);}",
        "function main(){const unwrap=(value:int):int=>value+1;unwrap(2);}",
    ] { tsrust_compiler::check(&support::format_round_trip(source)).unwrap(); }
}

#[test]
fn configurable_pool_cancellation_and_shutdown_execute() {
    let directory = support::TempDirectory::new();
    let source = r#"
import {pool,spawn,join,cancel,cancellationRequested} from 'std:threads';
import {strictEqual} from 'std:testing';
function main(){
    match(pool.configure(0,1)){Ok(_)=>{throw 'accepted invalid workers';},Err(error)=>{strictEqual(error.code,'range');}}
    unwrap(pool.configure(1,2));
    match(pool.configure(2,4)){Ok(_)=>{throw 'reconfigured';},Err(error)=>{strictEqual(error.code,'configured');}}
    const parent=unwrap(pool.task(():int=>{const child=unwrap(pool.task(():int=>42));return unwrap(join(child));}));
    strictEqual(unwrap(join(parent)),42);
    strictEqual(pool.stats().outstanding,0);
    const first=unwrap(pool.task(():void=>{while(!cancellationRequested()) {}}));
    while(pool.stats().running===0) {}
    const queued=unwrap(pool.task(():int=>{throw 'queued callback ran';}));
    strictEqual(pool.stats().outstanding,2);
    match(pool.task(():void=>{})){Ok(_)=>{throw 'capacity overflow';},Err(error)=>{strictEqual(error.code,'full');}}
    strictEqual(cancel(queued),true);strictEqual(cancel(queued),false);cancel(first);
    match(join(first)){Ok(_)=>{throw 'lost cancellation';},Err(error)=>{strictEqual(error.code,'cancelled');}}
    match(join(queued)){Ok(_)=>{throw 'lost queued cancellation';},Err(error)=>{strictEqual(error.code,'cancelled');}}
    strictEqual(pool.stats().outstanding,0);
    const dedicated=unwrap(spawn(():void=>{while(!cancellationRequested()) {}}));
    cancel(dedicated);
    match(join(dedicated)){Ok(_)=>{throw 'lost dedicated cancellation';},Err(error)=>{strictEqual(error.code,'cancelled');}}
    strictEqual(cancellationRequested(),false);
    unwrap(pool.shutdown());unwrap(pool.shutdown());
    match(pool.task(():void=>{})){Ok(_)=>{throw 'reopened pool';},Err(error)=>{strictEqual(error.code,'closed');}}
    const after=unwrap(spawn(():int=>7));strictEqual(unwrap(join(after)),7);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),120));
    let path=directory.source(&support::format_round_trip(include_str!("../../../examples/worker-pool/main.tsr")));
    let output=support::run(support::cli().arg("run").arg(path),120);support::success(&output);
    assert_eq!(output.stdout,b"42\ncancelled\n0\n");
}

#[test]
fn cancellation_borrows_but_does_not_duplicate_worker_ownership() {
    for body in [
        "cancel(42);",
        "pool.missing();",
        "pool.task();",
        "const task=pool.task;",
        "pool.task(():int=>1);",
        "const worker=unwrap(spawn(():int=>1));join(worker);cancel(worker);",
        "pool.configure('two',2);",
        "cancellationRequested(1);",
        "spawn(():int=>1);",
    ] {
        let directory=support::TempDirectory::new();
        let source=format!("import {{pool,spawn,join,cancel,cancellationRequested}} from 'std:threads';function main(){{{body}}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted {source}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
fn pool_namespace_aliases_preserve_lexical_shadowing() {
    let directory=support::TempDirectory::new();
    let source="import {pool as workers,join} from 'std:threads';type Local={task:int;};function shadow(workers:Local):int{return workers.task;}function main(){unwrap(workers.configure(1,2));const task=unwrap(workers.task(():int=>42));console.log(unwrap(join(task)));console.log(shadow({task:7}));unwrap(workers.shutdown());}";
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),120);support::success(&output);
    assert_eq!(output.stdout,b"42\n7\n");
    for name in ["__poolTask","__poolConfigure","__poolStats","__poolShutdown","configurePool","spawnDedicated"] {
        let path=directory.source(&format!("import {{{name}}} from 'std:threads';function main(){{}}"));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"exported {name}");
    }
}
