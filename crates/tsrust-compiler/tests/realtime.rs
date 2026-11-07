mod support;

#[test]
fn realtime_resource_contracts_reject_clone_readonly_and_reuse() {
    for source in [
        "import {WebSocketMessage} from 'std:http';function bad(message:ReadOnly<WebSocketMessage|undefined>){console.log(message.kind);}function main(){}",
        "import {SseEvent} from 'std:http';import {Sender} from 'std:futures';async function bad(sender:Sender<SseEvent>):Task<void>{await sender.send({data:1});}function main(){}",
        "import {WebSocket} from 'std:http';function bad(socket:ReadOnly<WebSocket>){socket.split();}function main(){}",
        "import {WebSocket} from 'std:http';function bad(socket:WebSocket){const duplicate=clone(socket);}function main(){}",
        "import {WebSocket} from 'std:http';function bad(socket:WebSocket){const halves=socket.split();socket.split();}function main(){}",
        "import {WebSocketRead} from 'std:http';async function bad(socket:ReadOnly<WebSocketRead>):Task<void>{await socket.receive();}function main(){}",
        "import {WebSocketWrite} from 'std:http';async function bad(socket:ReadOnly<WebSocketWrite>):Task<void>{await socket.send({kind:'text',data:'bad'});}function main(){}",
    ] {
        let directory=support::TempDirectory::new();
        let source=support::format_round_trip(source);
        let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);
        assert!(!output.status.success(),"accepted {source}");
        let stderr=String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("error[syntax]")&&!stderr.contains("error[import]"),"{stderr}");
    }
}

#[test]
fn sse_and_websocket_select_only_the_required_runtime_family() {
    use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
    for(source,websocket)in [
        ("import {sse,SseEvent} from 'std:http';import {Channel} from 'std:futures';function main(){const events=new Channel<SseEvent>(1);events.close();const response=sse(events);console.log(response.status());}",false),
        ("import {connectWebSocket} from 'std:http';async function main():Task<void>{const socket=await connectWebSocket('ws://127.0.0.1:1/');await socket.close();}",true),
        ("import {Router} from 'std:http';function main(){const app=new Router();app.webSocket('/',async socket=>{await socket.close();});}",true),
        ("import {WebSocketRead} from 'std:http';async function inspect(socket:Mutable<WebSocketRead>):Task<void>{const message=await socket.receive();if(message!==undefined&&message.kind==='text'){console.log(message.data);}}function main(){}",true),
        ("import {SseEvent} from 'std:http';import {Sender} from 'std:futures';async function send(sender:Sender<SseEvent>):Task<void>{await sender.send({data:'prefix'});}function main(){}",false),
        ("import {connectWebSocket} from 'std:http';import {rejects} from 'std:testing';function pem():string{return 'selected';}async function inspect(url:string):Task<void>{const stored={connect:connectWebSocket};await rejects(async()=>{const socket=await stored.connect(url,{ca:pem()});await socket.close();});}function main(){}",true),
    ] {
        let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
        let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
        let rust=checked.emit_rust();
        assert_eq!(rust.contains("// tsrust-http-websocket-selected"),websocket);
        assert!(!rust.contains("unsafe {"));
    }
}

#[test]
fn websocket_literal_messages_keep_native_context_in_lazy_tasks(){
 use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};
 let prefix="import {WebSocketWrite} from 'std:http';import {fromArray} from 'std:bytes';";
 let valid=format!("{prefix}async function send(socket:Owned<WebSocketWrite>):Task<void>{{await socket.send({{kind:'text',data:'ok'}});await socket.send({{kind:'binary',data:unwrap(fromArray([0,255]))}});await socket.close();}}function main(){{}}");
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&valid));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();let checked=check_graph(graph).unwrap_or_else(|error|panic!("lazy message context: {error:?}"));assert!(!checked.emit_rust().contains("unsafe {"));
 for message in ["{kind:'text',data:1}","{kind:'unknown',data:'bad'}"]{
  let source=format!("{prefix}async function bad(socket:Owned<WebSocketWrite>):Task<void>{{await socket.send({message});await socket.close();}}function main(){{}}");let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();let error=check_graph(graph).err().expect("invalid message must retain its native union contract");assert_eq!(error.code,"type");
 }
}
