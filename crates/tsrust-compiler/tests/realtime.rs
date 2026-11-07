mod support;
use std::path::Path;

// One stable project path and one native testing build, rather than compiling
// separate programs for each network/ownership assertion.
#[test]
fn public_realtime_registered_suite_executes() {
    let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source=root.join("examples/realtime/realtime.test.tsr");
    let original=std::fs::read_to_string(&source).unwrap();
    assert_eq!(support::format_round_trip(&original),original);
    let output=support::run(support::cli().current_dir(&root).arg("test").arg("--profile").arg(&source).env("TSRUST_TEST_REPORTER","json"),900);
    support::success(&output);
    let text=String::from_utf8(output.stdout).unwrap();
    print!("{text}");
    let rows=text.lines().filter(|line|!line.trim().is_empty()).map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|error|panic!("Malformed suite JSON {line:?}: {error}"))).collect::<Vec<_>>();
    let cases=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();
    assert_eq!(cases.len(),18,"{text}");
    let names=cases.iter().map(|row|row["name"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(),18,"duplicate registered cases: {text}");
    let expected=["SSE backend bytes and ownership > framing preserves empty and CRLF backend semantics","SSE backend bytes and ownership > owned producer completes and drains buffered events","SSE backend bytes and ownership > unpolled response drop settles owned producer","SSE backend bytes and ownership > partial body drop settles owned producer","SSE backend bytes and ownership > producer failure is a body failure and drops typed cause","SSE backend bytes and ownership > invalid event fields fail the body","WebSocket owned transport > text and binary tags survive full socket and split halves","explicit realtime policies > keepalive is opt in and preserves comment bytes","explicit realtime policies > authentication and Origin middleware reject before upgrade","explicit realtime policies > explicit frame and message limits reject an oversized echo","explicit realtime policies > invalid close frames fail before affecting the connection","explicit realtime policies > explicit shutdown deadline settles an active upgraded handler","explicit realtime policies > representative binary sizes retain exact bytes","WebSocket TLS verification > explicit private CA verifies a matching hostname","WebSocket TLS verification > default public roots reject the private CA","WebSocket TLS verification > custom trust still rejects a wrong hostname","WebSocket TLS verification > custom trust still rejects an expired certificate","WebSocket TLS verification > custom trust still rejects a future certificate"].into_iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names,expected,"registered suite names changed: {text}");
    for row in cases{assert_eq!(row["status"],"passed","{row}");let duration=row["durationMs"].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
    let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();
    assert_eq!(summaries.len(),1,"{text}");
    for(key,expected)in[("passed",18),("failed",0),("hookFailures",0),("skipped",0)]{assert_eq!(summaries[0][key],expected,"{text}");}
}

#[test]
fn realtime_resource_contracts_reject_clone_readonly_and_reuse() {
    for source in [
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
    ] {
        let directory=support::TempDirectory::new();let source=support::format_round_trip(source);let path=directory.source(&source);
        let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
        let rust=checked.emit_rust();
        assert_eq!(rust.contains("// tsrust-http-websocket-selected"),websocket);
        assert!(!rust.contains("unsafe {"));
    }
}
