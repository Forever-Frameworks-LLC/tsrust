use crossbeam_channel::{after, bounded, select};
use lsp_server::{Connection, ErrorCode, Message, Notification, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use serde_json::{Value, json};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::{Duration, Instant};
use tsrust_lsp::{Documents, Snapshot, code_actions, diagnostics};

struct DocumentJob {
    snapshot: Snapshot,
    workspace: Option<tsrust_lsp::workspace::WorkspaceSnapshot>,
}

enum DiagnosticBatch {
    Single(Vec<lsp_types::Diagnostic>),
    Project(
        tsrust_lsp::project_session::Buckets,
        Option<tsrust_lsp::project_intelligence::ProjectIndex>,
        Option<tsrust_lsp::project_actions::ProjectActions>,
    ),
}

struct DocumentChecked {
    snapshot: Snapshot,
    workspace: Option<tsrust_lsp::workspace::WorkspaceSnapshot>,
    result: Result<DiagnosticBatch, String>,
}
enum Job {
    Document(DocumentJob),
    Workspace(tsrust_lsp::workspace_scan::Job),
}
enum Checked {
    Document(DocumentChecked),
    Workspace(tsrust_lsp::workspace_scan::Checked),
}
fn send(connection: &Connection, message: impl Into<Message>) -> Result<(), String> {
    connection
        .sender
        .send(message.into())
        .map_err(|e| e.to_string())
}
fn log_failure(connection: &Connection, message: String) -> Result<(), String> {
    eprintln!("tsrust-lsp: {message}");
    send(
        connection,
        Notification::new(
            "window/showMessage".into(),
            json!({"type": 1, "message": message}),
        ),
    )
}
fn run(
    connection: Connection,
    checker: impl Fn(&str, &str) -> Result<Vec<lsp_types::Diagnostic>, String> + Send + 'static,
) -> Result<i32, String> {
    let (jobs, work) = bounded::<Job>(1);
    let (completed, results) = bounded::<Checked>(1);
    // Analysis is synchronous. Dropping this thread's handle permits exit without
    // waiting for an obsolete check. Cooperative interruption is compiler task C3.
    std::thread::spawn(move || {
        let mut scan_cache=tsrust_lsp::workspace_scan::Cache::default();
        while let Ok(job) = work.recv() {
            let job = match job {
                Job::Workspace(job) => {
                    let result=catch_unwind(AssertUnwindSafe(||scan_cache.check(&job)))
                        .unwrap_or_else(|_|Err("workspace analysis panicked".into()));
                    if completed.send(Checked::Workspace(tsrust_lsp::workspace_scan::Checked {job,result})).is_err() {break;}
                    continue;
                }
                Job::Document(job) => job,
            };
            let DocumentJob {snapshot, workspace} = job;
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Some(workspace) = &workspace {
                    let entry = tsrust_lsp::project_diagnostics::source_id(&snapshot.uri)?;
                    let result = workspace.check_with_diagnostics(&entry);
                    let index = match &result {
                        Ok(analysis) => Some(tsrust_lsp::project_intelligence::ProjectIndex::checked(analysis)),
                        Err(_) => {
                            tsrust_compiler::project::load_project_headers(workspace, &entry.0)
                                .ok()
                                .map(|graph| {
                                    tsrust_lsp::project_intelligence::ProjectIndex::new(&graph)
                                })
                        }
                    };
                    let errors = result.err().unwrap_or_default();
                    let error = errors.first();
                    let actions = error
                        .map(|error| {
                            tsrust_lsp::project_actions::ProjectActions::build(
                                workspace, &entry, error,
                            )
                        })
                        .transpose()?;
                    tsrust_lsp::project_diagnostics::convert_all(workspace, &errors)
                        .map(|buckets| DiagnosticBatch::Project(buckets, index, actions))
                } else {
                    checker(&snapshot.text, &snapshot.uri).map(DiagnosticBatch::Single)
                }
            }))
            .unwrap_or_else(|_| {
                Err(
                    "compiler analysis panicked; restart the server and report the source case"
                        .into(),
                )
            });
            if completed
                .send(Checked::Document(DocumentChecked {
                    snapshot,
                    workspace,
                    result,
                }))
                .is_err()
            {
                break;
            }
        }
    });
    let mut documents = Documents::default();
    let mut projects = tsrust_lsp::project_session::ProjectSession::default();
    let mut scanner = tsrust_lsp::workspace_scan::Scanner::default();
    let mut busy = false;
    let mut initialized = false;
    let mut ready = false;
    let mut shutdown = false;
    let mut debounce = Duration::from_millis(200);
    loop {
        if ready
            && !shutdown
            && !busy
            && let Some(snapshot) = documents.next(Instant::now())
        {
            let workspace = projects.prepare(&snapshot);
            jobs.send(Job::Document(DocumentJob {
                snapshot,
                workspace,
            }))
            .map_err(|_| "analysis worker stopped")?;
            busy = true;
        }
        if ready && !shutdown && !busy
            && let Some(job)=scanner.next(Instant::now(),projects.workspace.snapshot()) {
            jobs.send(Job::Workspace(job)).map_err(|_|"analysis worker stopped")?;
            busy=true;
        }
        let wait = if !busy && ready && !shutdown {
            documents.due_in(Instant::now()).into_iter()
                .chain(scanner.due_in(Instant::now())).min().unwrap_or(Duration::from_secs(60))
        } else {
            Duration::from_secs(60)
        };
        select! {
            recv(connection.receiver) -> message => {
                let Ok(message) = message else { return Ok(if shutdown { 0 } else { 1 }); };
                match message {
                    Message::Request(request) => {
                        let response = match request.method.as_str() {
                            "initialize" if !initialized && !shutdown => {
                                    debounce = Duration::from_millis(request.params.pointer("/initializationOptions/debounceMs").and_then(Value::as_u64).unwrap_or(200).min(2000));
                                    scanner.initialize(&request.params, Instant::now());
                                    initialized = true;
                                    Response::new_ok(request.id, json!({
                                        "capabilities": {"workspace":{"workspaceFolders":{"supported":true,"changeNotifications":true}}, "positionEncoding": "utf-16", "textDocumentSync": {"openClose": true, "change": 1}, "hoverProvider": true, "codeActionProvider": {"codeActionKinds": ["quickfix"]}, "documentFormattingProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "signatureHelpProvider": {"triggerCharacters": ["(", ","], "retriggerCharacters": [","]}, "definitionProvider": true, "referencesProvider": true, "documentHighlightProvider": true, "documentSymbolProvider": true, "foldingRangeProvider": true},
                                        "serverInfo": {"name": "tsrust-lsp", "version": env!("CARGO_PKG_VERSION")}
                                    }))
                            }
                            "shutdown" if initialized && !shutdown => {
                                shutdown = true;
                                Response::new_ok(request.id, Value::Null)
                            }
                            "textDocument/formatting" if initialized && !shutdown => {
                                let snapshot = request.params.pointer("/textDocument/uri").and_then(Value::as_str).and_then(|uri| documents.snapshot(uri));
                                match snapshot.map(|snapshot| tsrust_compiler::format(&snapshot.text).map(|formatted| {
                                    let range = tsrust_lsp::LineIndex::new(&snapshot.text).range(&snapshot.text, tsrust_compiler::Span::new(0, snapshot.text.len())).expect("snapshot boundaries");
                                    if formatted == snapshot.text { json!([]) } else { json!([{"range":range,"newText":formatted}]) }
                                })) {
                                    Some(Ok(result)) => Response::new_ok(request.id, result),
                                    Some(Err(error)) => Response::new_err(request.id, ErrorCode::RequestFailed as i32, error.message),
                                    None => Response::new_ok(request.id, json!([])),
                                }
                            }
                            "textDocument/codeAction" if initialized && !shutdown => {
                                let snapshot = request.params.pointer("/textDocument/uri").and_then(Value::as_str).and_then(|uri| documents.snapshot(uri));
                                match snapshot.map(|snapshot| {
                                    if tsrust_compiler::project_cli::uses_modules(&snapshot.text).unwrap_or(false) {
                                        projects.code_actions(&snapshot.uri, &request.params)
                                    } else { code_actions(snapshot, &request.params) }
                                }) {
                                    Some(Ok(result)) => Response::new_ok(request.id, result),
                                    Some(Err(error)) => Response::new_err(request.id, ErrorCode::InvalidParams as i32, error),
                                    None => Response::new_ok(request.id, json!([])),
                                }
                            }
                            method if initialized && !shutdown && matches!(method, "textDocument/hover" | "textDocument/completion" | "textDocument/signatureHelp" | "textDocument/definition" | "textDocument/references" | "textDocument/documentHighlight" | "textDocument/documentSymbol" | "textDocument/foldingRange") => {
                                let uri = request.params.pointer("/textDocument/uri").and_then(Value::as_str);
                                match uri.and_then(|uri| projects.request(method, &request.params, uri).or_else(|| documents.index(uri).map(|index| index.request(method, &request.params, uri)))) {
                                    Some(Ok(result)) => Response::new_ok(request.id, result),
                                    Some(Err(error)) => Response::new_err(request.id, ErrorCode::InvalidParams as i32, error),
                                    None => Response::new_ok(request.id, Value::Null),
                                }
                            }
                            _ if shutdown => Response::new_err(request.id, ErrorCode::InvalidRequest as i32, "Server is shutting down".into()),
                            _ if !initialized => Response::new_err(request.id, ErrorCode::ServerNotInitialized as i32, "Server is not initialized".into()),
                            _ => Response::new_err(request.id, ErrorCode::MethodNotFound as i32, "Method is not supported".into()),
                        };
                        send(&connection, response)?;
                    }
                    Message::Notification(notification) => {
                        if notification.method == "exit" { return Ok(if shutdown { 0 } else { 1 }); }
                        if !initialized || shutdown { continue; }
                        let now = Instant::now();
                        let handled: Result<(), String> = match notification.method.as_str() {
                            "initialized" => {ready=true;Ok(())},
                            "workspace/didChangeWorkspaceFolders" => {
                                scanner.change_folders(&notification.params,now);Ok(())
                            },
                            "workspace/didChangeWatchedFiles" => serde_json::from_value::<lsp_types::DidChangeWatchedFilesParams>(notification.params).map_err(|e|e.to_string()).map(|params| {
                                scanner.dirty(now,debounce);
                                for change in params.changes {
                                    for dependent in projects.changed_file(change.uri.as_str()) { documents.reschedule(&dependent, now, debounce); }
                                }
                            }),
                            "textDocument/didOpen" => serde_json::from_value::<DidOpenTextDocumentParams>(notification.params).map_err(|e| e.to_string()).map(|params| {
                                let doc = params.text_document;
                                if auxiliary_uri(doc.uri.as_str()) {
                                    scanner.dirty(now,debounce);
                                    let snapshot=Snapshot{uri:doc.uri.to_string(),version:doc.version,generation:0,text:doc.text};
                                    for dependent in projects.open(&snapshot) {documents.reschedule(&dependent,now,debounce);}
                                } else if doc.language_id == "tsrust" {
                                    scanner.dirty(now,debounce);
                                    let uri = doc.uri.to_string();
                                    documents.open(uri.clone(), doc.version, doc.text, now);
                                    if let Some(snapshot) = documents.snapshot(&uri) {
                                        for dependent in projects.open(snapshot) { documents.reschedule(&dependent, now, debounce); }
                                    }
                                }
                            }),
                            "textDocument/didChange" => serde_json::from_value::<DidChangeTextDocumentParams>(notification.params).map_err(|e| e.to_string()).and_then(|params| {
                                if params.content_changes.len() != 1 || params.content_changes[0].range.is_some() {
                                    return Err("Expected exactly one full-document change".into());
                                }
                                let uri = params.text_document.uri.to_string();
                                let change = params.content_changes.into_iter().next().unwrap();
                                if auxiliary_uri(&uri) {
                                    scanner.dirty(now,debounce);
                                    let snapshot=Snapshot{uri:uri.clone(),version:params.text_document.version,generation:0,text:change.text};
                                    for dependent in projects.change(&snapshot) {documents.reschedule(&dependent,now,debounce);}
                                } else if !documents.change(&uri, params.text_document.version, change.text, now, debounce) {
                                    eprintln!("tsrust-lsp: ignored outdated or unopened document change: {uri}");
                                } else if let Some(snapshot) = documents.snapshot(&uri) {
                                    scanner.dirty(now,debounce);
                                    for dependent in projects.change(snapshot) { documents.reschedule(&dependent, now, debounce); }
                                }
                                Ok(())
                            }),
                            "textDocument/didClose" => serde_json::from_value::<DidCloseTextDocumentParams>(notification.params).map_err(|e| e.to_string()).and_then(|params| {
                                let uri = params.text_document.uri.to_string();
                                scanner.dirty(now,debounce);
                                documents.close(&uri);
                                let (dependents, publications) = projects.close(&uri);
                                for dependent in dependents { documents.reschedule(&dependent, now, debounce); }
                                if scanner.covers(&uri) {Ok(())} else {
                                    for publication in publications { publish_project(&connection, publication)?; }
                                    send(&connection, Notification::new("textDocument/publishDiagnostics".into(), json!({"uri": uri, "diagnostics": []})))
                                }
                            }),
                            _ => Ok(()),
                        };
                        if let Err(error) = handled { eprintln!("tsrust-lsp: invalid notification: {error}"); }
                    }
                    Message::Response(_) => {}
                }
            }
            recv(results) -> result => {
                busy = false;
                let checked = match result.map_err(|_| "analysis worker stopped")? {
                    Checked::Workspace(checked) => {
                        if !shutdown {
                            match scanner.accept(checked,&projects,Instant::now(),debounce) {
                                Ok(publications)=>for publication in publications {
                                    if !scanner.covers(&publication.uri) {documents.reschedule(&publication.uri,Instant::now(),debounce);}
                                    publish_project(&connection,publication)?;
                                },
                                Err(error)=>log_failure(&connection,format!("Could not check workspace: {error}. Previous diagnostics may be outdated."))?,
                            }
                        }
                        continue;
                    }
                    Checked::Document(checked)=>checked,
                };
                if !shutdown && documents.current(&checked.snapshot) && checked.workspace.as_ref().is_none_or(|snapshot| projects.workspace.current(snapshot)) {
                    match checked.result {
                        Ok(DiagnosticBatch::Single(diagnostics)) => {
                            if scanner.covers(&checked.snapshot.uri) {continue;}
                            for publication in projects.forget_project(&checked.snapshot.uri) { publish_project(&connection, publication)?; }
                            send(&connection, Notification::new("textDocument/publishDiagnostics".into(), json!({
                                "uri": checked.snapshot.uri, "version": checked.snapshot.version, "diagnostics": diagnostics
                            })))?;
                        },
                        Ok(DiagnosticBatch::Project(buckets,index,actions)) => {
                            if let Ok(entry)=tsrust_lsp::project_diagnostics::source_id(&checked.snapshot.uri) {
                                projects.workspace.register(entry,checked.workspace.as_ref().expect("project input"));
                            }
                            projects.cache_actions(&checked.snapshot.uri, checked.workspace.as_ref().expect("project input"), actions);
                            projects.cache_index(&checked.snapshot.uri, checked.workspace.as_ref().expect("project input"), index);
                            if !scanner.covers(&checked.snapshot.uri) {
                                for publication in projects.accept(&checked.snapshot.uri, checked.workspace.as_ref().expect("project input"), buckets)? { publish_project(&connection, publication)?; }
                            }
                        },
                        Err(error) => log_failure(&connection, format!("Could not check {}: {error}. Previous diagnostics may be outdated.", checked.snapshot.uri))?,
                    }
                } else if !shutdown && documents.current(&checked.snapshot) {
                    documents.reschedule(&checked.snapshot.uri, Instant::now(), debounce);
                }
            }
            recv(after(wait)) -> _ => {}
        }
    }
}
fn auxiliary_uri(uri:&str)->bool {
    tsrust_lsp::project_diagnostics::source_path_id(uri).is_ok_and(|id|std::path::Path::new(&id.0).extension().and_then(|value|value.to_str()).is_some_and(|extension|matches!(extension,"json"|"jsonc")))
}
fn publish_project(
    connection: &Connection,
    publication: tsrust_lsp::project_session::Publication,
) -> Result<(), String> {
    let mut params = json!({"uri": publication.uri, "diagnostics": publication.diagnostics});
    if let Some(version) = publication.version {
        params["version"] = json!(version);
    }
    send(
        connection,
        Notification::new("textDocument/publishDiagnostics".into(), params),
    )
}
fn main() {
    let (connection, _io_threads) = Connection::stdio();
    let code = match run(connection, diagnostics) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("tsrust-lsp: {error}");
            1
        }
    };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_server::{Request, RequestId};
    fn request(client: &Connection, id: i32, method: &str) {
        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(id),
                method: method.into(),
                params: json!({}),
            }))
            .unwrap();
    }
    fn notify(client: &Connection, method: &str, params: Value) {
        client
            .sender
            .send(Notification::new(method.into(), params).into())
            .unwrap();
    }
    fn receive(client: &Connection) -> Message {
        client
            .receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("server must stay responsive")
    }
    fn initialize(client: &Connection) {
        request(client, 1, "initialize");
        assert!(matches!(receive(client), Message::Response(response) if response.error.is_none()));
        notify(client,"initialized",json!({}));
    }
    #[test]
    fn slow_check_cannot_publish_after_edit_close_or_reopen() {
        let (server, client) = Connection::memory();
        let (entered_tx, entered_rx) = bounded(1);
        let (release_tx, release_rx) = bounded(1);
        let first = std::sync::atomic::AtomicBool::new(true);
        let thread = std::thread::spawn(move || {
            run(server, move |text, uri| {
                if first.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                diagnostics(text, uri)
            })
        });
        initialize(&client);
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument": {"uri":"file:///a.tsr", "languageId":"tsrust", "version":1, "text":"function main( {"}}),
        );
        entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument": {"uri":"file:///a.tsr", "version":2}, "contentChanges":[{"text":"function main() {}"}]}),
        );
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument": {"uri":"file:///b.tsr", "languageId":"tsrust", "version":1, "text":"function main( {"}}),
        );
        notify(
            &client,
            "textDocument/didClose",
            json!({"textDocument":{"uri":"file:///a.tsr"}}),
        );
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument": {"uri":"file:///a.tsr", "languageId":"tsrust", "version":1, "text":"function main() {}"}}),
        );
        assert!(
            matches!(receive(&client), Message::Notification(n) if n.params["uri"] == "file:///a.tsr" && n.params["diagnostics"] == json!([]) && n.params.get("version").is_none())
        );
        request(&client, 2, "test/barrier");
        assert!(
            matches!(receive(&client), Message::Response(response) if response.id == RequestId::from(2))
        );
        release_tx.send(()).unwrap();
        assert!(
            matches!(receive(&client), Message::Notification(n) if n.params["uri"] == "file:///b.tsr" && n.params["diagnostics"].as_array().unwrap().len() == 1)
        );
        assert!(
            matches!(receive(&client), Message::Notification(n) if n.params["uri"] == "file:///a.tsr" && n.params["diagnostics"] == json!([]) && n.params["version"] == 1)
        );
        request(&client, 3, "shutdown");
        assert!(
            matches!(receive(&client), Message::Response(response) if response.error.is_none())
        );
        notify(&client, "exit", Value::Null);
        assert_eq!(thread.join().unwrap().unwrap(), 0);
    }
    #[test]
    fn shutdown_does_not_wait_for_obsolete_analysis() {
        let (server, client) = Connection::memory();
        let (entered_tx, entered_rx) = bounded(1);
        let (release_tx, release_rx) = bounded(1);
        let thread = std::thread::spawn(move || {
            run(server, move |text, uri| {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                diagnostics(text, uri)
            })
        });
        initialize(&client);
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument": {"uri":"file:///a.tsr", "languageId":"tsrust", "version":1, "text":"function main() {}"}}),
        );
        entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        request(&client, 2, "shutdown");
        assert!(
            matches!(receive(&client), Message::Response(response) if response.error.is_none())
        );
        notify(&client, "exit", Value::Null);
        assert_eq!(thread.join().unwrap().unwrap(), 0);
        release_tx.send(()).unwrap();
    }
}

#[cfg(test)]
mod project_protocol_tests {
    use super::*;
    use lsp_server::{Request, RequestId};
    fn notify(client: &Connection, method: &str, params: Value) {
        client
            .sender
            .send(Notification::new(method.into(), params).into())
            .unwrap();
    }
    fn until(client: &Connection, predicate: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let message = client
                .receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("project diagnostics must arrive");
            if let Message::Notification(notification) = message
                && notification.method == "textDocument/publishDiagnostics"
                && predicate(&notification.params)
            {
                return notification.params;
            }
        }
    }
    #[test]
    fn live_json_data_edits_recheck_dependents_and_clear_diagnostics() {
        let (server, client) = Connection::memory();
        let worker = std::thread::spawn(move || run(server, diagnostics));
        client.sender.send(Message::Request(Request { id: 1.into(), method: "initialize".into(), params: json!({"initializationOptions":{"debounceMs":0}}) })).unwrap();
        assert!(matches!(client.receiver.recv_timeout(Duration::from_secs(5)).unwrap(), Message::Response(_)));
        notify(&client,"initialized",json!({}));
        let root="file:///tsrust-live-data/main.tsr";
        let data="file:///tsrust-live-data/settings.jsonc";
        notify(&client,"textDocument/didOpen",json!({"textDocument":{"uri":data,"languageId":"jsonc","version":1,"text":"{\"host\":\"local\",}"}}));
        notify(&client,"textDocument/didOpen",json!({"textDocument":{"uri":root,"languageId":"tsrust","version":1,"text":"import settings from './settings.jsonc'; function main(){console.log(settings.host.toUpperCase());}"}}));
        until(&client,|p|p["uri"]==root && p["diagnostics"].as_array().is_some_and(Vec::is_empty));
        notify(&client,"textDocument/didChange",json!({"textDocument":{"uri":data,"version":2},"contentChanges":[{"text":"{\"host\":42}"}]}));
        until(&client,|p|p["uri"]==root && p["diagnostics"].as_array().is_some_and(|items|!items.is_empty()));
        notify(&client,"textDocument/didChange",json!({"textDocument":{"uri":data,"version":3},"contentChanges":[{"text":"{\"host\":\"restored\"}"}]}));
        until(&client,|p|p["uri"]==root && p["diagnostics"].as_array().is_some_and(Vec::is_empty));
        notify(&client,"textDocument/didClose",json!({"textDocument":{"uri":data}}));
        until(&client,|p|p["uri"]==root && p["diagnostics"].as_array().is_some_and(|items|!items.is_empty()));
        client.sender.send(Message::Request(Request{id:9.into(),method:"shutdown".into(),params:json!(null)})).unwrap();
        loop { if matches!(client.receiver.recv_timeout(Duration::from_secs(5)).unwrap(),Message::Response(response) if response.id==RequestId::from(9)) {break;} }
        notify(&client,"exit",json!(null)); assert_eq!(worker.join().unwrap().unwrap(),0);
    }
    #[test]
    fn live_alias_configuration_updates_import_diagnostics_without_becoming_source() {
        fn receive(client:&Connection)->Message {client.receiver.recv_timeout(Duration::from_secs(5)).unwrap()}
        fn request(client:&Connection,id:i32,method:&str) {client.sender.send(Message::Request(Request{id:RequestId::from(id),method:method.into(),params:json!(null)})).unwrap();}
        fn initialize(client:&Connection) {
            client.sender.send(Message::Request(Request{id:RequestId::from(1),method:"initialize".into(),params:json!({"initializationOptions":{"debounceMs":0}})})).unwrap();
            assert!(matches!(receive(client),Message::Response(response) if response.error.is_none()));
            notify(client,"initialized",json!({}));
        }
        let (server,client)=Connection::memory();
        let worker=std::thread::spawn(move||run(server,diagnostics));initialize(&client);
        let config="file:///tsrust-live-alias/tsrust.config.json";
        let root="file:///tsrust-live-alias/main.tsr";
        let dep="file:///tsrust-live-alias/src/greeting.tsr";
        notify(&client,"textDocument/didOpen",json!({"textDocument":{"uri":config,"languageId":"json","version":1,"text":r#"{"paths":{"@/*":"./src/*"}}"#}}));
        notify(&client,"textDocument/didOpen",json!({"textDocument":{"uri":dep,"languageId":"tsrust","version":1,"text":"export function greeting():string{return 'hello';}"}}));
        notify(&client,"textDocument/didOpen",json!({"textDocument":{"uri":root,"languageId":"tsrust","version":1,"text":"import {greeting} from '@/greeting';function main(){console.log(greeting());}"}}));
        until(&client,|p|p["uri"]==root&&p["diagnostics"].as_array().is_some_and(Vec::is_empty));
        client.sender.send(Message::Request(Request{id:RequestId::from(7),method:"textDocument/hover".into(),params:json!({"textDocument":{"uri":root},"position":{"line":0,"character":64}})})).unwrap();
        loop {if let Message::Response(response)=receive(&client) {if response.id==RequestId::from(7) {let result=response.result.unwrap();assert!(result.to_string().contains("string"),"{result}");break;}}}
        notify(&client,"textDocument/didChange",json!({"textDocument":{"uri":config,"version":2},"contentChanges":[{"text":r#"{"paths":{"@/*":"./missing/*"}}"#}]}));
        until(&client,|p|p["uri"]==root&&p["diagnostics"].as_array().is_some_and(|items|!items.is_empty()));
        notify(&client,"textDocument/didChange",json!({"textDocument":{"uri":config,"version":3},"contentChanges":[{"text":r#"{"paths":{"@/*":"./src/*"}}"#}]}));
        until(&client,|p|p["uri"]==root&&p["diagnostics"].as_array().is_some_and(Vec::is_empty));
        notify(&client,"textDocument/didClose",json!({"textDocument":{"uri":config}}));
        until(&client,|p|p["uri"]==root&&p["diagnostics"].as_array().is_some_and(|items|!items.is_empty()));
        request(&client,9,"shutdown");
        loop {if matches!(receive(&client),Message::Response(response) if response.id==RequestId::from(9)){break;}}
        notify(&client,"exit",json!(null));assert_eq!(worker.join().unwrap().unwrap(),0);
    }
    #[test]
    fn live_unsaved_dependency_errors_recheck_clear_and_keep_versions() {
        let (server, client) = Connection::memory();
        let worker = std::thread::spawn(move || run(server, diagnostics));
        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(1),
                method: "initialize".into(),
                params: json!({"initializationOptions":{"debounceMs":0}}),
            }))
            .unwrap();
        assert!(matches!(
            client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap(),
            Message::Response(_)
        ));
        notify(&client,"initialized",json!({}));
        let root = "file:///tsrust-live-project/main.tsr";
        let dep = "file:///tsrust-live-project/dep.tsr";
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":dep,"languageId":"tsrust","version":1,"text":"export function value():int{return 2;}"}}),
        );
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":root,"languageId":"tsrust","version":1,"text":"import { value } from './dep.tsr';function main(){console.log(value());}"}}),
        );
        until(&client, |p| {
            p["uri"] == root
                && p["version"] == 1
                && p["diagnostics"].as_array().is_some_and(Vec::is_empty)
        });
        let entry_source =
            "import { value } from './dep.tsr';function main(){console.log(value());}";
        client.sender.send(Message::Request(Request { id: RequestId::from(3), method: "textDocument/hover".into(), params: json!({"textDocument":{"uri":root},"position":{"line":0,"character":entry_source.rfind("value()").unwrap()}}) })).unwrap();
        loop {
            if let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                && response.id == RequestId::from(3)
            {
                    assert!(
                        response.result.unwrap()["contents"]["value"]
                            .as_str()
                            .unwrap()
                            .contains("value")
                    );
                    break;
            }
        }
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":dep,"version":2},"contentChanges":[{"text":"export function value():int{return missing;}"}]}),
        );
        let error = until(&client, |p| {
            p["uri"] == dep
                && p["version"] == 2
                && p["diagnostics"].as_array().is_some_and(|v| !v.is_empty())
        });
        assert!(
            error["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("missing")
        );
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":dep,"version":3},"contentChanges":[{"text":"export function value():int{return 3;}"}]}),
        );
        until(&client, |p| {
            p["uri"] == dep
                && p["version"] == 3
                && p["diagnostics"].as_array().is_some_and(Vec::is_empty)
        });
        notify(
            &client,
            "textDocument/didClose",
            json!({"textDocument":{"uri":dep}}),
        );
        let missing = until(&client, |p| {
            p["uri"] == root && p["diagnostics"].as_array().is_some_and(|v| !v.is_empty())
        });
        assert!(
            missing["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("Missing source")
        );
        notify(
            &client,
            "textDocument/didOpen",
            json!({"textDocument":{"uri":dep,"languageId":"tsrust","version":3,"text":"export function value():int{return 4;}"}}),
        );
        until(&client, |p| {
            p["uri"] == root
                && p["version"] == 1
                && p["diagnostics"].as_array().is_some_and(Vec::is_empty)
        });
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":dep,"version":4},"contentChanges":[{"text":"export function value():int{const n=4 return n;}"}]}),
        );
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":root,"version":2},"contentChanges":[{"text":entry_source}]}),
        );
        until(&client, |p| p["uri"] == root && p["version"] == 2);
        client.sender.send(Message::Request(Request {
            id: RequestId::from(8), method: "textDocument/codeAction".into(),
            params: json!({"textDocument":{"uri":dep},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1000}},"context":{"diagnostics":[]}}),
        })).unwrap();
        loop {
            if let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                && response.id == RequestId::from(8)
            {
                let actions = response.result.unwrap();
                assert_eq!(actions.as_array().unwrap().len(), 1, "{actions}");
                assert_eq!(
                    actions[0]["edit"]["documentChanges"][0]["textDocument"]["uri"],
                    dep
                );
                assert_eq!(
                    actions[0]["edit"]["documentChanges"][0]["textDocument"]["version"],
                    4
                );
                assert_eq!(
                    actions[0]["edit"]["documentChanges"][0]["edits"][0]["newText"],
                    ";"
                );
                break;
            }
        }
        let incomplete = "import { value } from './dep.tsr';function main(){val";
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":root,"version":3},"contentChanges":[{"text":incomplete}]}),
        );
        until(&client, |p| p["uri"] == root && p["version"] == 3);
        client.sender.send(Message::Request(Request {
            id: RequestId::from(10), method: "textDocument/completion".into(),
            params: json!({"textDocument":{"uri":root},"position":{"line":0,"character":incomplete.len()}}),
        })).unwrap();
        loop {
            if let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                && response.id == RequestId::from(10)
            {
                let completion = response.result.unwrap();
                assert!(
                    completion
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|item| item["label"] == "value"),
                    "{completion}"
                );
                break;
            }
        }
        // An unfinished text and an import that is still being typed keep project assistance.
        let typing = "import { value } from './dep.tsr';\nimport { val } from './dep.tsr';\nfunction main(){const s=\"abc";
        notify(
            &client,
            "textDocument/didChange",
            json!({"textDocument":{"uri":root,"version":4},"contentChanges":[{"text":typing}]}),
        );
        until(&client, |p| p["uri"] == root && p["version"] == 4);
        client.sender.send(Message::Request(Request {
            id: RequestId::from(11), method: "textDocument/completion".into(),
            params: json!({"textDocument":{"uri":root},"position":{"line":1,"character":"import { val".len()}}),
        })).unwrap();
        loop {
            if let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                && response.id == RequestId::from(11)
            {
                let completion = response.result.unwrap();
                let labels = completion.as_array().unwrap().iter().map(|item| item["label"].clone()).collect::<Vec<_>>();
                assert_eq!(labels, [json!("value")], "{completion}");
                break;
            }
        }
        client
            .sender
            .send(Message::Request(Request {
                id: RequestId::from(9),
                method: "shutdown".into(),
                params: Value::Null,
            }))
            .unwrap();
        loop {
            if let Message::Response(response) = client
                .receiver
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                && response.id == RequestId::from(9)
            {
                    break;
            }
        }
        notify(&client, "exit", json!({}));
        assert_eq!(worker.join().unwrap().unwrap(), 0);
    }
}
