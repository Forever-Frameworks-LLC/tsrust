//! A real language server over stdio, driven the way an editor drives it:
//! one full-document change per typed character, then a request.
#![allow(dead_code)]
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::{Value, json};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

pub struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    messages: Receiver<Message>,
    root: PathBuf,
    next_request: i32,
    versions: std::collections::BTreeMap<String, i32>,
    /// `tsrust/serverCounters` style log lines are read from the final request.
    pub log: Vec<String>,
}

/// The cursor is where `|` stands in `text`.
pub fn cursor(text: &str) -> (String, usize) {
    let offset = text.find('|').expect("cursor marker");
    (text.replacen('|', "", 1), offset)
}

fn position(text: &str, offset: usize) -> Value {
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    json!({"line": text[..offset].matches('\n').count(), "character": text[line_start..offset].encode_utf16().count()})
}

impl Session {
    /// Start the server on a fresh workspace folder that holds `files`.
    pub fn start(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrust-lsp-session-{}-{}",
            std::process::id(),
            NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        // Editors resolve the folder; temporary directories are often links.
        let root = root.canonicalize().unwrap();
        for (name, text) in files {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        // `TSRUST_LSP_SERVER` runs the same checks against another build, for
        // example to record what an earlier server answered.
        let server = std::env::var("TSRUST_LSP_SERVER").unwrap_or_else(|_| env!("CARGO_BIN_EXE_tsrust-lsp").to_owned());
        let mut child = Command::new(server)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
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
        let mut session = Self { child, stdin, messages, root, next_request: 0, versions: Default::default(), log: Vec::new() };
        let folder = session.uri("");
        let folder = folder.trim_end_matches('/').to_owned();
        let reply = session.request("initialize", json!({"capabilities": {}, "rootUri": folder, "workspaceFolders": [{"uri": folder, "name": "workspace"}]}));
        assert!(reply.get("capabilities").is_some(), "initialize: {reply}");
        session.notify("initialized", json!({}));
        session
    }

    pub fn uri(&self, name: &str) -> String {
        tsrust_lsp::project_diagnostics::file_uri(&tsrust_compiler::project::SourceId(
            self.root.join(name).to_str().unwrap().into(),
        )).unwrap().to_string()
    }

    fn send(&mut self, message: Message) {
        let stdin = self.stdin.as_mut().unwrap();
        message.write(stdin).unwrap();
        stdin.flush().unwrap();
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        self.send(Notification::new(method.into(), params).into());
    }

    /// Send one request and wait for its response. Server requests and
    /// notifications that arrive first are not the reply.
    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_request += 1;
        let id = RequestId::from(self.next_request);
        self.send(Message::Request(Request { id: id.clone(), method: method.into(), params }));
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let wait = deadline.saturating_duration_since(Instant::now());
            match self.messages.recv_timeout(wait).unwrap_or_else(|_| panic!("no response to {method}")) {
                Message::Response(response) if response.id == id => {
                    return match response.error {
                        Some(error) => json!({"error": error.message}),
                        None => response.result.unwrap_or(Value::Null),
                    };
                }
                Message::Notification(notification) if notification.method == "window/logMessage" => {
                    self.log.extend(notification.params["message"].as_str().map(str::to_owned));
                }
                _ => {}
            }
        }
    }

    pub fn open(&mut self, name: &str, text: &str) {
        std::fs::write(self.root.join(name), text).unwrap();
        self.versions.insert(name.to_owned(), 1);
        let uri = self.uri(name);
        self.notify("textDocument/didOpen", json!({"textDocument": {"uri": uri, "languageId": "tsrust", "version": 1, "text": text}}));
    }

    pub fn change(&mut self, name: &str, text: &str) {
        let version = self.versions.get_mut(name).expect("open document");
        *version += 1;
        let version = *version;
        let uri = self.uri(name);
        self.notify("textDocument/didChange", json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]}));
    }

    /// Type `typed` between `before` and `after`, one character per change.
    pub fn type_text(&mut self, name: &str, before: &str, typed: &str, after: &str, delay: Duration) {
        let mut end = 0;
        for character in typed.chars() {
            end += character.len_utf8();
            self.change(name, &format!("{before}{}{after}", &typed[..end]));
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
        }
    }

    /// Wait for a specific publication, including a closed workspace sentinel.
    pub fn wait_for_diagnostics(&mut self,name:&str)->Value {
        let uri=self.uri(name);let deadline=Instant::now()+Duration::from_secs(10);
        loop {
            let message=self.messages.recv_timeout(deadline.saturating_duration_since(Instant::now())).expect("server must publish diagnostics before the deadline");
            if let Message::Notification(notification)=message {
                if notification.method=="textDocument/publishDiagnostics" && notification.params["uri"]==uri {return notification.params["diagnostics"].clone();}
            }
        }
    }

    /// Wait until the server has published diagnostics and stays quiet.
    pub fn settle(&mut self) {
        let mut quiet = Duration::from_millis(900);
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            match self.messages.recv_timeout(quiet) {
                Ok(Message::Notification(notification)) if notification.method == "textDocument/publishDiagnostics" => {
                    quiet = Duration::from_millis(400);
                }
                Ok(_) => {}
                Err(_) => return,
            }
        }
    }

    pub fn completion(&mut self, name: &str, text: &str, offset: usize) -> Result<Vec<String>, String> {
        let uri = self.uri(name);
        let reply = self.request("textDocument/completion", json!({"textDocument": {"uri": uri}, "position": position(text, offset)}));
        if let Some(error) = reply.get("error") {
            return Err(error.to_string());
        }
        let items = if reply.is_array() { reply } else { reply.get("items").cloned().unwrap_or(Value::Null) };
        Ok(items.as_array().into_iter().flatten().filter_map(|item| item["label"].as_str().map(str::to_owned)).collect())
    }

    pub fn completion_items(&mut self, name: &str, text: &str, offset: usize) -> Value {
        let uri = self.uri(name);
        self.request("textDocument/completion", json!({"textDocument": {"uri": uri}, "position": position(text, offset)}))
    }

    pub fn hover(&mut self, name: &str, text: &str, offset: usize) -> Option<String> {
        let uri = self.uri(name);
        let reply = self.request("textDocument/hover", json!({"textDocument": {"uri": uri}, "position": position(text, offset)}));
        reply.pointer("/contents/value").and_then(Value::as_str).map(str::to_owned)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
