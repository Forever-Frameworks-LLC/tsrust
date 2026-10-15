//! The editor and the checker must agree on what a type offers.
//!
//! Fields are compared exactly. For each receiver type the checker decides
//! which field names it accepts: `x.NAME` is compiled for every candidate
//! name, and a name is a field when the checker does not answer as it does
//! for a name that cannot exist. Completion must offer exactly those fields,
//! and the detail of each must name the checker's type for it. A field that
//! the checker accepts and the editor does not offer teaches a false rule
//! about the language. `Result.payload` was such a field.
//!
//! Methods are compared in one direction. The checker validates the argument
//! count of some methods before their receiver, so a call without arguments
//! cannot prove that a method exists. It can prove that one does not: every
//! method that completion offers must not be rejected as unknown or for its
//! receiver type. The other direction needs a member query from the compiler.
mod support;
use std::collections::{BTreeMap, BTreeSet};
use support::session::Session;
use tsrust_lsp::{project_diagnostics::source_id, workspace::Workspace};

const PRELUDE: &str = "import {Job, AsyncError, Channel as AsyncChannel, Sender as AsyncSender} from 'std:futures';\nimport {Channel, Sender, TaskContext, ChannelError, SendError, Thread, ThreadPoolOptions, ThreadPool, ThreadPoolStats, SpawnError, ThreadError, ScopedThread} from 'std:threads';\nimport { s, ValidationError, SchemaIssue } from 'std:schema';\ntype Item = { id: int; name: string; };\nconst schema = s.string();\n";

/// A receiver `x` of one type inside `probe`.
struct Receiver {
    ty: &'static str,
    /// Parameter list of `probe` that declares `x`.
    parameter: &'static str,
}

const RECEIVERS: &[Receiver] = &[
    Receiver { ty: "Job<int, undefined>", parameter: "x: ReadOnly<Job<int, undefined>>" },
    Receiver { ty: "AsyncError<string>", parameter: "x: ReadOnly<AsyncError<string>>" },
    Receiver { ty: "AsyncChannel<int>", parameter: "x: ReadOnly<AsyncChannel<int>>" },
    Receiver { ty: "AsyncSender<int>", parameter: "x: ReadOnly<AsyncSender<int>>" },
    Receiver { ty: "Task<int>", parameter: "x: Task<int>" },
    Receiver { ty: "Channel<int>", parameter: "x: ReadOnly<Channel<int>>" },
    Receiver { ty: "Sender<int>", parameter: "x: Sender<int>" },
    Receiver { ty: "TaskContext", parameter: "x: ReadOnly<TaskContext>" },
    Receiver { ty: "ChannelError", parameter: "x: ReadOnly<ChannelError>" },
    Receiver { ty: "SendError<string>", parameter: "x: ReadOnly<SendError<string>>" },
    Receiver { ty: "Thread<int, undefined>", parameter: "x: Owned<Thread<int, undefined>>" },
    Receiver { ty: "ThreadPoolOptions", parameter: "x: ReadOnly<ThreadPoolOptions>" },
    Receiver { ty: "ThreadPool", parameter: "x: ReadOnly<ThreadPool>" },
    Receiver { ty: "ThreadPoolStats", parameter: "x: ReadOnly<ThreadPoolStats>" },
    Receiver { ty: "ScopedThread<int, undefined>", parameter: "x: Owned<ScopedThread<int, undefined>>" },
    Receiver { ty: "SpawnError", parameter: "x: ReadOnly<SpawnError>" },
    Receiver { ty: "ThreadError<string>", parameter: "x: ReadOnly<ThreadError<string>>" },
    Receiver { ty: "int[]", parameter: "x: ReadOnly<int[]>" },
    Receiver { ty: "string[][]", parameter: "x: ReadOnly<string[][]>" },
    Receiver { ty: "string", parameter: "x: ReadOnly<string>" },
    Receiver { ty: "int", parameter: "x: int" },
    Receiver { ty: "number", parameter: "x: number" },
    Receiver { ty: "boolean", parameter: "x: boolean" },
    Receiver { ty: "Item", parameter: "x: ReadOnly<Item>" },
    Receiver { ty: "Map<string, int>", parameter: "x: ReadOnly<Map<string, int>>" },
    Receiver { ty: "Set<int>", parameter: "x: ReadOnly<Set<int>>" },
    Receiver { ty: "[int, string]", parameter: "x: ReadOnly<[int, string]>" },
    Receiver { ty: "Result<string, int>", parameter: "x: ReadOnly<Result<string, int>>" },
    Receiver { ty: "Result<Item, string>", parameter: "x: ReadOnly<Result<Item, string>>" },
    Receiver { ty: "Result<void, string>", parameter: "x: ReadOnly<Result<void, string>>" },
    Receiver { ty: "ValidationError", parameter: "x: ReadOnly<ValidationError>" },
    Receiver { ty: "SchemaIssue", parameter: "x: ReadOnly<SchemaIssue>" },
];

/// A reviewed difference between the checker and the editor. Each entry must
/// still be a difference; the test fails when one is resolved, so the entry
/// is removed with the change that resolves it.
struct Exception {
    ty: &'static str,
    name: &'static str,
    reason: &'static str,
}

const EXCEPTIONS: &[Exception] = &[];

/// A name no type declares.
const ABSENT: &str = "zzqqabsent";

fn diagnostic(body: &str, parameter: &str) -> Option<(String, String)> {
    let source = format!("{PRELUDE}function probe({parameter}) {{\n  {body}\n}}\nfunction main() {{}}\n");
    let entry = source_id("file:///tsrust-agreement/main.tsr").unwrap();
    let mut workspace = Workspace::default();
    workspace.open(entry.clone(), 1, source);
    workspace.snapshot().check_with_diagnostics(&entry).err().map(|errors| (errors[0].code.to_owned(), errors[0].message.clone()))
}

/// The checker's answer for `name`, with the name itself removed so that two
/// rejections of different names compare equal.
fn answer(form: &str, name: &str, parameter: &str) -> Option<(String, String)> {
    diagnostic(&form.replace("NAME", name), parameter).map(|(code, mut message)| {
        // Only the last quoted name is the member; a type can have the same spelling.
        let quoted = format!("\"{name}\"");
        if let Some(at) = message.rfind(&quoted) {
            message.replace_range(at..at + quoted.len(), "\"NAME\"");
        }
        (code, message)
    })
}

/// The checker's type of field `name` of the receiver: the type it names
/// when a member that cannot exist is read from that field.
fn field_type(name: &str, parameter: &str) -> Option<String> {
    let (_, message) = diagnostic(&format!("console.log(x.{name}.{ABSENT});"), parameter)?;
    let rest = message.strip_prefix("Type \"")?;
    // Diagnostics spell a native type by its linked name; the editor shows
    // the declared name.
    let mut ty = rest[..rest.find("\" has no field")?].to_owned();
    while let Some(start) = ty.find("__tsrust_module_native#") {
        let end = start + ty[start..].rfind('#').map_or(0, |end| end + 1);
        let name_end = ty[start..].find(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '#' | ':'))).map_or(ty.len(), |offset| start + offset);
        let linked = &ty[start..name_end];
        let declared = linked.rsplit('#').next().unwrap_or(linked).to_owned();
        let _ = end;
        ty.replace_range(start..name_end, &declared);
    }
    Some(ty)
}

fn alternatives(ty: &str) -> BTreeSet<String> {
    let mut depth = 0i32;
    let mut parts = BTreeSet::new();
    let mut begin = 0;
    for (offset, ch) in ty.char_indices() {
        match ch {
            '(' | '[' | '<' | '{' => depth += 1,
            ')' | ']' | '>' | '}' => depth -= 1,
            '|' if depth == 0 => {
                parts.insert(ty[begin..offset].split_whitespace().collect::<String>());
                begin = offset + 1;
            }
            _ => {}
        }
    }
    parts.insert(ty[begin..].split_whitespace().collect::<String>());
    parts
}

fn candidates() -> BTreeSet<String> {
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let mut names = BTreeSet::new();
    for entries in catalog.as_object().unwrap().values() {
        for entry in entries.as_array().into_iter().flatten() {
            if let Some(name) = entry["name"].as_str().filter(|name| name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')) {
                names.insert(name.to_owned());
            }
        }
    }
    for name in ["workers", "capacity", "queued", "running", "completed", "outstanding", "payload", "cause", "ok", "value", "error", "id", "name", "issues", "truncated", "code", "message", "path", "pathTruncated", "expected", "observed", "size", "length"] {
        names.insert(name.to_owned());
    }
    names
}

#[test]
fn completion_offers_the_fields_the_checker_accepts_and_no_method_it_rejects() {
    let candidates = candidates();
    let mut failures = Vec::new();
    let mut unresolved = EXCEPTIONS.iter().map(|exception| (exception.ty, exception.name)).collect::<BTreeSet<_>>();
    let only = std::env::var("TSRUST_LSP_AGREEMENT_TYPE").ok();
    for receiver in RECEIVERS {
        if only.as_ref().is_some_and(|only| only != receiver.ty) {
            continue;
        }
        assert!(diagnostic("", receiver.parameter).is_none(), "{}: the probe program must check", receiver.ty);
        let (field_form, call_form) = ("console.log(x.NAME);", "x.NAME();");
        let absent_field = answer(field_form, ABSENT, receiver.parameter);
        let absent_call = answer(call_form, ABSENT, receiver.parameter);
        assert!(absent_field.is_some() && absent_call.is_some(), "{}: an absent member must be rejected", receiver.ty);
        let fields = candidates.iter().filter(|name| answer(field_form, name, receiver.parameter) != absent_field).cloned().collect::<BTreeSet<_>>();
        let before = format!("{PRELUDE}function probe({}) {{\n  x.", receiver.parameter);
        let text = format!("{before}\n}}\nfunction main() {{}}\n");
        let mut session = Session::start(&[]);
        session.open("main.tsr", &text);
        session.settle();
        let reply = session.completion_items("main.tsr", &text, before.len());
        let items = if reply.is_array() { reply } else { reply["items"].clone() };
        let mut details = BTreeMap::new();
        let mut offered = BTreeSet::new();
        let mut methods = BTreeSet::new();
        for item in items.as_array().into_iter().flatten() {
            let label = item["label"].as_str().unwrap().to_owned();
            match item["kind"].as_u64() {
                // A snippet that replaces the whole access is an action, not a member.
                Some(15) => {}
                Some(2) => {
                    methods.insert(label);
                }
                _ => {
                    details.insert(label.clone(), item["detail"].as_str().unwrap_or("").to_owned());
                    offered.insert(label);
                }
            }
        }
        let mut differences = fields.difference(&offered).map(|name| (name.clone(), "the checker accepts this field; completion does not offer it".to_owned())).collect::<Vec<_>>();
        differences.extend(offered.difference(&fields).map(|name| (name.clone(), "completion offers this field; the checker rejects it".to_owned())));
        for name in &methods {
            let answered = answer(call_form, name, receiver.parameter);
            let receiver_rejected = answered.as_ref().is_some_and(|(_, message)| message.ends_with(&format!(", found {}.", receiver.ty)));
            if answered == absent_call || receiver_rejected {
                differences.push((name.clone(), format!("completion offers this method; the checker rejects it: {}", answered.map_or_else(String::new, |(_, message)| message))));
            }
        }
        for (name, problem) in differences {
            let listed = EXCEPTIONS.iter().find(|exception| exception.ty == receiver.ty && exception.name == name);
            if !listed.is_some_and(|exception| unresolved.remove(&(exception.ty, exception.name))) {
                failures.push(format!("{}.{name}: {problem}", receiver.ty));
            }
        }
        for name in fields.intersection(&offered) {
            let Some(checked) = field_type(name, receiver.parameter) else { continue };
            let shown = details[name].split_once(": ").map_or("", |(_, ty)| ty);
            if alternatives(shown) != alternatives(&checked) {
                failures.push(format!("{}.{name}: completion shows `{shown}`; the checker's type is `{checked}`", receiver.ty));
            }
        }
    }
    if only.is_none() {
        for (ty, name) in unresolved {
            let reason = EXCEPTIONS.iter().find(|exception| exception.ty == ty && exception.name == name).unwrap().reason;
            failures.push(format!("{ty}.{name}: listed as an exception ({reason}) but the editor and the checker now agree"));
        }
    }
    assert!(failures.is_empty(), "{} disagreements between the editor and the checker:\n{}", failures.len(), failures.join("\n"));
}
