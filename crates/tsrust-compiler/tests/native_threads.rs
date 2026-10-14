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
fn lifecycle_rejects_abandonment_and_partial_consumption() {
    for body in [
        "spawn(():int=>1);",
        "match(spawn(():int=>1)){Ok(_)=>{},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{while(flag){if(flag){detach(worker);break;}break;}},Err(error)=>{}}",
        "while(flag){match(spawn(():int=>1)){Ok(worker)=>{continue;},Err(error)=>{break;}}}",
        "const pending=spawn(():int=>1);",
        "match(spawn(():int=>1)){Ok(worker)=>{},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{if(flag){detach(worker);}},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{return;},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{throw 'failure';},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{let value=1;value+=1;detach(worker);},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{const bad=[worker];},Err(error)=>{}}",
        "match(spawn(():int=>1)){Ok(worker)=>{const abandoned=():void=>{detach(worker);};},Err(error)=>{}}",
    ] {
        let directory=support::TempDirectory::new();
        let source=format!("import {{spawn,join,detach}} from 'std:threads';function probe(flag:boolean){{{body}}}function main(){{probe(true);}}");
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
        "match(spawn(():int=>1)){Ok(worker)=>{const values=[1];console.log(values[9]);detach(worker);},Err(error)=>{}}",
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
        "import {spawn} from 'std:threads';const pending=spawn(():int=>1);function main(){}",
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
import {spawnScoped,join,detach} from 'std:threads';
import {unwrap} from 'std:result';
import {strictEqual} from 'std:testing';
function early(){const worker=unwrap(spawnScoped(():void=>{console.log('returned');}));return;}
function failing(){const worker=unwrap(spawnScoped(():void=>{console.log('cleaned');throw 'worker error';}));throw 'caller error';}
function main(){
    {const worker=unwrap(spawnScoped(():void=>{console.log('normal');}));}
    console.log('after normal');
    early();console.log('after return');
    try{failing();}catch(error){if(typeof error==='string'){strictEqual(error,'caller error');console.log(error);}else{throw error;}}finally{console.log('finally');}
    for(let i=0;i<2;i+=1){const worker=unwrap(spawnScoped(():void=>{console.log('iteration');}));if(i===0){continue;}break;}
    console.log('after loop');
    const done=unwrap(spawnScoped(():void=>{}));unwrap(join(done));
    const detached=unwrap(spawnScoped(():void=>{}));detach(detached);
    try{const worker=unwrap(spawnScoped(():int=>{throw 'original';}));unwrap(join(worker));}catch(error){console.log(error.code);}
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
import {spawnScoped,join} from 'std:threads';
import {unwrap} from 'std:result';
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
    const first=unwrap(spawnScoped(():int=>rendezvous('first.ready','second.ready')));
    const second=unwrap(spawnScoped(():int=>rendezvous('second.ready','first.ready')));
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
import {unwrap} from 'std:result';
import {spawnScoped,join,ScopedThread,ThreadError} from 'std:threads';
import {strictEqual} from 'std:testing';
function result(flag:boolean):Result<string,string|int>{if(flag){return Ok('owned');}return Err('failure');}
function take(worker:ScopedThread<int,undefined>):int{return unwrap(join(worker));}
function main(){
    strictEqual(unwrap(result(true)),'owned');
    const values=unwrap(Ok<int[],string>([1,2]));values.push(3);strictEqual(values.length,3);
    unwrap(Ok<void,string>());
    try{unwrap(Err<int,string>('direct'));}catch(error){strictEqual(error,'direct');}
    try{unwrap(result(false));}catch(error){strictEqual(String(error),'failure');}
    const worker=unwrap(spawnScoped(():int=>42));strictEqual(take(worker),42);
}
"#;
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for body in [
        "unwrap(42);",
        "const operation=unwrap;",
        "const worker=unwrap(spawnScoped(():int=>42));join(worker);join(worker);",
        "const worker=unwrap(spawnScoped(():int=>42));const workers=[worker];",
        "const worker=unwrap(spawnScoped(():int=>42));const cb=():void=>{join(worker);};",
        "const text='borrowed';const cb=():int=>text.length;const worker=unwrap(spawnScoped(cb));console.log(text);",
    ] {
        let source=format!("import {{unwrap}} from 'std:result';import {{spawnScoped,join}} from 'std:threads';function main(){{{body}}}");
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
