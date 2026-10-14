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
