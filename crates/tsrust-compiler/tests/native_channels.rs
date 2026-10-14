mod support;

#[test]
fn channel_ownership_backpressure_and_worker_context() {
    let directory = support::TempDirectory::new();
    let source = r#"
import {Channel,Sender,spawn,ThreadPool} from 'std:threads';
import {strictEqual} from 'std:testing';
function producer(sender:Sender<string>):Function<void>{return ():void=>{unwrap(sender.send('first'));unwrap(sender.send('second'));};}
function main(){
    try { const invalid=new Channel<int>(0); throw 'accepted capacity'; }
    catch(error) { if(typeof error==='string'){throw error;}else{strictEqual(error.code,'range');} }
    const channel=new Channel<string>(1);
    const sender=channel.sender();
    const worker=unwrap(spawn(producer(sender)));
    strictEqual(unwrap(channel.receive()),'first');
    strictEqual(unwrap(channel.receive()),'second');
    unwrap(worker.join());
    match(channel.receive()){Ok(_)=>{throw 'expected closure';},Err(error)=>strictEqual(error.code,'closed')}
    const bounded=new Channel<string>(1);
    const writer=bounded.sender();
    unwrap(writer.send('queued'));
    match(writer.trySend('recover')){Ok(_)=>{throw 'expected full';},Err(error)=>{strictEqual(error.code,'full');strictEqual(error.value,'recover');}}
    bounded.close();
    strictEqual(unwrap(bounded.receive()),'queued');
    match(writer.send('unsent')){Ok(_)=>{throw 'expected closed';},Err(error)=>strictEqual(error.value,'unsent')}
    const listener=unwrap(spawn((task):void=>{while(!task.cancellationRequested()) {}}));
    listener.cancel();
    match(listener.join()){Ok(_)=>{},Err(error)=>strictEqual(error.code,'cancelled')}
    const pool=new ThreadPool({workers:2,capacity:4});
    const task=unwrap(pool.task((context):boolean=>context.cancellationRequested()));
    strictEqual(unwrap(task.join()),false);
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn channel_endpoints_preserve_transfer_and_unique_ownership() {
    for body in [
        "const inbox=new Channel<int>(1);const copy=clone(inbox);",
        "const inbox=new Channel<int>(1);const sender=inbox.sender();const copy=clone(sender);",
        "const inbox=new Channel<Function<int>>(1);",
        "const inbox=new Channel<Channel<int>>(1);",
        "const inbox=new Channel<Sender<int>>(1);",
        "const inbox=new Channel<int>(1);const sender=inbox.sender();sender.send('wrong');",
        "const inbox=new Channel<int>();",
        "const inbox=new Channel<int>(1.5);",
        "const worker=unwrap(spawn((task):TaskContext=>task));",
        "const worker=unwrap(spawn((task:int):int=>task));",
    ] {
        let directory=support::TempDirectory::new();
        let source=format!("import {{Channel,Sender,TaskContext,spawn}} from 'std:threads';function main(){{{body}}}");
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success(),"accepted: {source}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
fn readable_channel_examples_execute() {
    for (example, expected) in [("channel-worker", "hello\ngoodbye\n"), ("channel-batch", "204\n")] {
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples").join(example).join("main.tsr");
        let source=std::fs::read_to_string(path).unwrap();
        let directory=support::TempDirectory::new();
        let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("run").arg(path),60);
        support::success(&output); assert_eq!(output.stdout,expected.as_bytes());
    }
}

#[test]
fn channel_finally_closes_before_outer_worker_cleanup() {
    let source=r#"
import {Channel,spawn} from 'std:threads';
import {strictEqual} from 'std:testing';
function listen(inbox:Channel<int>):void {match(inbox.receive()){Ok(_)=>{},Err(error)=>strictEqual(error.code,'closed')}}
function fail():void {
    const inbox=new Channel<int>(1);const sender=inbox.sender();
    const worker=unwrap(spawn(():void=>listen(inbox)));
    try {throw 'caller';} finally {sender.close();}
}
function main(){try{fail();}catch(error){if(typeof error==='string'){strictEqual(error,'caller');}else{throw error;}}}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),30));
}
