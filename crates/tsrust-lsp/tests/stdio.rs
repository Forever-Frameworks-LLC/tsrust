use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::{Value, json};
use std::io::{BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};
struct Server {
    child: Child,
    stdin: Option<ChildStdin>,
    messages: Receiver<Message>,
}
impl Server {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tsrust-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (sender, messages) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Ok(Some(message)) = Message::read(&mut reader) {
                if sender.send(message).is_err() {
                    break;
                }
            }
        });
        Self {
            child,
            stdin,
            messages,
        }
    }
    fn request(&mut self, id: i32, method: &str, params: Value) {
        self.send(Message::Request(Request {
            id: RequestId::from(id),
            method: method.into(),
            params,
        }));
    }
    fn notify(&mut self, method: &str, params: Value) {
        self.send(Notification::new(method.into(), params).into());
    }
    fn send(&mut self, message: Message) {
        let stdin = self.stdin.as_mut().unwrap();
        message.write(stdin).unwrap();
        stdin.flush().unwrap();
    }
    fn receive(&self) -> Message {
        self.messages
            .recv_timeout(Duration::from_secs(5))
            .expect("framed response within timeout")
    }
    /// The next response. Debounced diagnostics may be published first when
    /// the machine is busy; those notifications are not the reply.
    fn response(&self) -> Message {
        loop {
            let message = self.receive();
            if matches!(message, Message::Response(_)) {
                return message;
            }
        }
    }
    fn finish(&mut self, expected: i32) {
        self.notify("exit", Value::Null);
        self.stdin.take();
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "server must exit promptly"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[test]
fn framed_transport_unicode_diagnostics_repair_and_lifecycle() {
    let mut server = Server::start();
    server.request(0, "unknown", json!({}));
    assert!(
        matches!(server.receive(), Message::Response(r) if r.error.as_ref().unwrap().code == -32002)
    );
    server.request(1, "initialize", json!({"capabilities":{"general":{"positionEncodings":["utf-16"]}},"initializationOptions":{"debounceMs":20}}));
    assert!(
        matches!(server.receive(), Message::Response(r) if r.result.as_ref().unwrap()["capabilities"]["textDocumentSync"]["change"] == 1)
    );
    server.notify("initialized", json!({}));
    let source = "function main() { const x = \"😀\"; const y: int = true; }";
    let expected = tsrust_lsp::diagnostics(source, "file:///unicode.tsr").unwrap();
    server.notify("textDocument/didOpen", json!({"textDocument":{"uri":"file:///unicode.tsr","languageId":"tsrust","version":1,"text":source}}));
    assert!(
        matches!(server.receive(), Message::Notification(n) if n.method == "textDocument/publishDiagnostics" && n.params["version"] == 1 && n.params["diagnostics"] == serde_json::to_value(expected).unwrap())
    );
    server.notify("textDocument/didChange", json!({"textDocument":{"uri":"file:///unicode.tsr","version":2},"contentChanges":[{"text":"function main() {}"}]}));
    assert!(
        matches!(server.receive(), Message::Notification(n) if n.params["version"] == 2 && n.params["diagnostics"] == json!([]))
    );
    server.notify(
        "textDocument/didClose",
        json!({"textDocument":{"uri":"file:///unicode.tsr"}}),
    );
    assert!(
        matches!(server.receive(), Message::Notification(n) if n.params["diagnostics"] == json!([]))
    );
    server.request(2, "shutdown", Value::Null);
    assert!(matches!(server.receive(), Message::Response(r) if r.error.is_none()));
    server.request(3, "unknown", Value::Null);
    assert!(
        matches!(server.receive(), Message::Response(r) if r.error.as_ref().unwrap().code == -32600)
    );
    server.finish(0);
}
#[test]
fn utf16_is_mandatory_and_exit_without_shutdown() {
    let mut server = Server::start();
    server.request(
        1,
        "initialize",
        json!({"capabilities":{"general":{"positionEncodings":["utf-8"]}}}),
    );
    assert!(
        matches!(server.receive(), Message::Response(r) if r.result.as_ref().unwrap()["capabilities"]["positionEncoding"] == "utf-16")
    );
    server.finish(1);
}

#[test]
fn quickfix_transport_uses_current_document_version() {
    let mut server = Server::start();
    server.request(
        1,
        "initialize",
        json!({"capabilities":{},"initializationOptions":{"debounceMs":2000}}),
    );
    assert!(
        matches!(server.receive(), Message::Response(r) if r.result.as_ref().unwrap()["capabilities"]["codeActionProvider"]["codeActionKinds"] == json!(["quickfix"]))
    );
    server.notify("initialized", json!({}));
    let uri = "file:///fix.tsr";
    let source = "function main(){const x=1}";
    server.notify(
        "textDocument/didOpen",
        json!({"textDocument":{"uri":uri,"languageId":"tsrust","version":4,"text":source}}),
    );
    let range = tsrust_lsp::diagnostics(source, uri).unwrap()[0].range;
    server.request(
        2,
        "textDocument/codeAction",
        json!({"textDocument":{"uri":uri},"range":range,"context":{"diagnostics":[]}}),
    );
    assert!(
        matches!(server.response(),Message::Response(r) if r.result.as_ref().unwrap()[0]["edit"]["documentChanges"][0]["textDocument"]["version"] == 4)
    );
    server.notify("textDocument/didChange",json!({"textDocument":{"uri":uri,"version":5},"contentChanges":[{"text":"function main(){const x=1;}"}]}));
    server.request(
        3,
        "textDocument/codeAction",
        json!({"textDocument":{"uri":uri},"range":range,"context":{"diagnostics":[]}}),
    );
    loop {
        if let Message::Response(response) = server.receive() {
            assert_eq!(response.id, RequestId::from(3));
            assert_eq!(response.result.unwrap(), json!([]));
            break;
        }
    }
    server.request(4, "shutdown", Value::Null);
    assert!(matches!(server.receive(), Message::Response(_)));
    server.finish(0);
}

#[test]
fn formatting_readonly_tuple_generics_returns_tight_idempotent_edit() {
    let mut server = Server::start();
    server.request(1, "initialize", json!({"capabilities":{}}));
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none()));
    server.notify("initialized", json!({}));
    let source = "function names(pair: ReadOnly < [string, string] >): int { const [left, right] = pair; return left.length + right.length; } function main() {}";
    let uri = "file:///formatter-generics.tsr";
    server.notify("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"tsrust","version":1,"text":source}}));
    server.request(2, "textDocument/formatting", json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    let Message::Response(response) = server.response() else { panic!("format response"); };
    assert!(response.error.is_none(), "{:?}", response.error);
    let edits = response.result.unwrap();
    let formatted = edits[0]["newText"].as_str().unwrap();
    assert!(formatted.contains("ReadOnly<[string, string]>"), "{formatted}");
    assert!(tsrust_compiler::check(formatted).is_ok());
    server.notify("textDocument/didChange", json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":formatted}]}));
    server.request(3, "textDocument/formatting", json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none() && r.result == Some(json!([]))));
    server.request(4, "shutdown", Value::Null);
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none()));
    server.finish(0);
}

#[test]
fn formatting_exception_handlers_returns_joined_idempotent_edit() {
    let mut server=Server::start();
    server.request(1,"initialize",json!({"capabilities":{}}));
    assert!(matches!(server.response(),Message::Response(r) if r.error.is_none()));
    server.notify("initialized",json!({}));
    let uri="file:///formatter-exceptions.tsr";
    let source="function main(){try{try{throw 1;}\n\nfinally{}}\n\ncatch(error){}\n\nfinally{}}";
    server.notify("textDocument/didOpen",json!({"textDocument":{"uri":uri,"languageId":"tsrust","version":1,"text":source}}));
    server.request(2,"textDocument/formatting",json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    let Message::Response(response)=server.response() else {panic!("format response");};
    assert!(response.error.is_none(),"{:?}",response.error);
    let edits=response.result.unwrap();
    let formatted=edits[0]["newText"].as_str().unwrap();
    assert!(formatted.contains("} catch (error) {"),"{formatted}");
    assert!(formatted.contains("} finally {"),"{formatted}");
    server.notify("textDocument/didChange",json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":formatted}]}));
    server.request(3,"textDocument/formatting",json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    assert!(matches!(server.response(),Message::Response(r) if r.error.is_none() && r.result==Some(json!([]))));
    server.request(4,"shutdown",Value::Null);
    assert!(matches!(server.response(),Message::Response(r) if r.error.is_none()));
    server.finish(0);
}

#[test]
fn formatting_interfaces_returns_tight_generics_and_separate_fields() {
    let mut server = Server::start();
    server.request(1, "initialize", json!({"capabilities":{}}));
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none()));
    server.notify("initialized", json!({}));
    let source = "interface Box < T > { value: T, label?: string } function main() { const values = new Map < string, int >(); }";
    let uri = "file:///formatter-generics.tsr";
    server.notify("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"tsrust","version":1,"text":source}}));
    server.request(2, "textDocument/formatting", json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    let Message::Response(response) = server.response() else { panic!("format response"); };
    assert!(response.error.is_none(), "{:?}", response.error);
    let edits = response.result.unwrap();
    let formatted = edits[0]["newText"].as_str().unwrap();
    assert!(formatted.contains("interface Box<T>"), "{formatted}");
    assert!(formatted.contains("value: T,\n    label?: string"), "{formatted}");
    assert!(formatted.contains("new Map<string, int>()"), "{formatted}");
    assert!(tsrust_compiler::check(formatted).is_ok());
    server.notify("textDocument/didChange", json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":formatted}]}));
    server.request(3, "textDocument/formatting", json!({"textDocument":{"uri":uri},"options":{"tabSize":4,"insertSpaces":true}}));
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none() && r.result == Some(json!([]))));
    server.request(4, "shutdown", Value::Null);
    assert!(matches!(server.response(), Message::Response(r) if r.error.is_none()));
    server.finish(0);
}

#[test]
fn named_capture_readiness_diagnostic_marks_use_and_related_binding() {
    let source = "function main() {\n    function read(): int { return value; }\n    console.log(read());\n    const value = 3;\n}";
    let uri = "file:///capture-readiness.tsr";
    let diagnostics = serde_json::to_value(tsrust_lsp::diagnostics(source, uri).unwrap()).unwrap();
    let error = &diagnostics[0];
    assert_eq!(error["code"], "unassigned");
    assert_eq!(error["range"]["start"], json!({"line":2,"character":16}));
    assert_eq!(error["range"]["end"], json!({"line":2,"character":20}));
    assert!(error["message"].as_str().unwrap().contains("captured binding"));
    assert_eq!(error["relatedInformation"][0]["location"]["uri"], uri);
    assert_eq!(error["relatedInformation"][0]["location"]["range"]["start"], json!({"line":3,"character":10}));
}
