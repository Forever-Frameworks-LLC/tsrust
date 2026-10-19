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

const PRELUDE: &str = "import {Signal,ThemeConfig,ReactiveBoolean} from 'std:ui';\nimport {CryptoError} from 'std:crypto';\nimport {SignalListener,SignalError} from 'std:signals';\nimport {Pool as SqlPool,Transaction as SqlTransaction,Cursor as SqlCursor,Row as SqlRow,SqlError} from 'std:sql';\nimport {Connection,Statement,Transaction,Row,SqliteError} from 'std:sqlite';\nimport {Router,Context,ContextNext,App,Client,Request,Response,Body,Server,Next,HttpError,TlsOptions,ClientOptions,ServerOptions,FormField,MultipartPart} from 'std:http';\nimport {Job, AsyncError, Channel as AsyncChannel, Sender as AsyncSender} from 'std:futures';\nimport {Channel, Sender, TaskContext, ChannelError, SendError, Thread, ThreadPoolOptions, ThreadPool, ThreadPoolStats, SpawnError, ThreadError, ScopedThread} from 'std:threads';\nimport { s, ValidationError, SchemaIssue } from 'std:schema';\ntype Item = { id: int; name: string; };\nconst schema = s.string();\n";

/// A receiver `x` of one type inside `probe`.
struct Receiver {
    ty: &'static str,
    /// Parameter list of `probe` that declares `x`.
    parameter: &'static str,
}

const RECEIVERS: &[Receiver] = &[
Receiver {ty:"ThemeConfig",parameter:"x: ReadOnly<ThemeConfig>"},
Receiver {ty:"ReactiveBoolean",parameter:"x: ReactiveBoolean"},
Receiver {ty:"Signal",parameter:"x: ReadOnly<Signal>"},
Receiver {ty:"InheritedItem",parameter:"x: ReadOnly<InheritedItem>"},
Receiver {ty:"CryptoError",parameter:"x: ReadOnly<CryptoError>"},
Receiver {ty:"SignalError",parameter:"x: ReadOnly<SignalError>"},
Receiver {ty:"SignalListener",parameter:"x: ReadOnly<SignalListener>"},
Receiver {ty:"ServerOptions",parameter:"x: ReadOnly<ServerOptions>"},
Receiver {ty:"FormField",parameter:"x: ReadOnly<FormField>"},
Receiver {ty:"MultipartPart",parameter:"x: ReadOnly<MultipartPart>"},
Receiver {ty:"LruCache<int,string>",parameter:"x: LruCache<int,string>"},
Receiver {ty:"CacheEntry<int,string>",parameter:"x: ReadOnly<CacheEntry<int,string>>"},
Receiver {ty:"CacheInsertError<int,string>",parameter:"x: ReadOnly<CacheInsertError<int,string>>"},
Receiver {ty:"Time",parameter:"x: ReadOnly<Time>"},
Receiver {ty:"Instant",parameter:"x: ReadOnly<Instant>"},
Receiver {ty:"Duration",parameter:"x: ReadOnly<Duration>"},
Receiver {ty:"TimeZone",parameter:"x: ReadOnly<TimeZone>"},
Receiver {ty:"Date",parameter:"x: ReadOnly<Date>"},
Receiver {ty:"DateError",parameter:"x: ReadOnly<DateError>"},
Receiver {ty:"Uuid",parameter:"x: ReadOnly<Uuid>"},
Receiver {ty:"UuidError",parameter:"x: ReadOnly<UuidError>"},
Receiver {ty:"Decimal",parameter:"x: ReadOnly<Decimal>"},
Receiver {ty:"DecimalError",parameter:"x: ReadOnly<DecimalError>"},
Receiver {ty:"NumericError",parameter:"x: ReadOnly<NumericError>"},
Receiver {ty:"TrackedResource",parameter:"x: ReadOnly<TrackedResource>"},
Receiver {ty:"SqlPool",parameter:"x: SqlPool"},
Receiver {ty:"SqlTransaction",parameter:"x: SqlTransaction"},
Receiver {ty:"SqlRow",parameter:"x: SqlRow"},
Receiver {ty:"SqlError",parameter:"x: SqlError"},

Receiver {ty:"Connection",parameter:"x: Connection"},
Receiver {ty:"Statement",parameter:"x: Statement"},
Receiver {ty:"Transaction",parameter:"x: Transaction"},
Receiver {ty:"Row",parameter:"x: Row"},
Receiver {ty:"SqliteError",parameter:"x: SqliteError"},

    Receiver {ty:"SqlCursor",parameter:"x: SqlCursor"},
    Receiver {ty:"Router",parameter:"x: Router"},
    Receiver {ty:"Context",parameter:"x: Mutable<Context>"},
    Receiver {ty:"ContextNext",parameter:"x: ContextNext"},
    Receiver {ty:"App",parameter:"x: App"},
    Receiver {ty:"Client",parameter:"x: Client"},
    Receiver {ty:"Request",parameter:"x: Request"},
    Receiver {ty:"Response",parameter:"x: Response"},
    Receiver {ty:"Body",parameter:"x: Body"},
    Receiver {ty:"Server",parameter:"x: Server"},
    Receiver {ty:"Next",parameter:"x: Next"},
    Receiver {ty:"TlsOptions",parameter:"x: TlsOptions"},
    Receiver {ty:"ClientOptions",parameter:"x: ClientOptions"},
    Receiver {ty:"HttpError",parameter:"x: HttpError"},

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

fn prelude(parameter: &str) -> String {
    // Only the resource row needs the bundled testing implementation in every probe.
    let testing = if parameter.contains("TrackedResource") { "import {TrackedResource} from 'std:testing';\n" } else { "" };
    let numeric=if parameter.contains("NumericError"){ "import {NumericError} from 'std:numeric';\n" }else{""};
    let date=if ["Date","Time","Instant","Duration"].iter().any(|name|parameter.contains(name)){ "import {Date,DateError,Time,Instant,Duration,TimeZone} from 'std:date';\n" }else{""};
    let uuid=if parameter.contains("Uuid"){ "import {Uuid,UuidError} from 'std:uuid';\n" }else{""};
    let decimal=if parameter.contains("Decimal"){ "import {Decimal,DecimalError} from 'std:decimal';\n" }else{""};
    let cache=if parameter.contains("Cache"){ "import {LruCache,CacheEntry,CacheInsertError} from 'std:cache';\n" }else{""};
    let inherited=if parameter.contains("InheritedItem"){ "interface Parent<T>{value:T;} interface InheritedItem extends Parent<string>{count:int;}\n" }else{""};
    format!("{inherited}{cache}{uuid}{date}{decimal}{numeric}{testing}{PRELUDE}")
}

fn diagnostic(body: &str, parameter: &str) -> Option<(String, String)> {
    let prelude = prelude(parameter);
    let source = format!("{prelude}function probe({parameter}) {{\n  {body}\n}}\nfunction main() {{}}\n");
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

// Missing-member diagnostics call every callable "closure". Compare its checked
// contract with a checked declaration of the displayed type instead.
fn callable_field_agrees(receiver:&Receiver,name:&str,shown:&str)->bool {
 use tsrust_compiler::project::{ProjectType,SourceId};
 let source=format!("{} interface DisplayedContract {{ member:{shown}; }} function probe({}){{}} function main(){{}}",prelude(receiver.parameter),receiver.parameter);
 let entry=SourceId("/tsrust-agreement-callable/main.tsr".into());let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source);
 let Ok(analysis)=workspace.snapshot().check_with_diagnostics(&entry) else{return false;};let facts=analysis.checked_type_facts();
 let find=|owner:&str,field:&str|facts.record_fields.iter().find(|(ty,n,_,_,_)|n==field && matches!(ty,ProjectType::Named{name,..} if name==owner)).map(|(_,_,_,ty,_)|ty);
 find(receiver.ty,name).zip(find("DisplayedContract","member")).is_some_and(|(actual,displayed)|actual==displayed)
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
            if let Some(fields) = entry["fields"].as_object() { names.extend(fields.keys().cloned()); }
            if let Some(name) = entry["name"].as_str().filter(|name| name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')) {
                names.insert(name.to_owned());
            }
        }
    }
    for name in ["cert", "key", "ca", "workers", "capacity", "queued", "running", "completed", "outstanding", "payload", "cause", "ok", "value", "error", "id", "name", "issues", "truncated", "code", "message", "path", "pathTruncated", "expected", "observed", "size", "length"] {
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
        eprintln!("agreement receiver: {}", receiver.ty);
        assert!(diagnostic("", receiver.parameter).is_none(), "{}: the probe program must check", receiver.ty);
        let (field_form, call_form) = ("console.log(x.NAME);", "x.NAME();");
        let absent_field = answer(field_form, ABSENT, receiver.parameter);
        let absent_call = answer(call_form, ABSENT, receiver.parameter);
        assert!(absent_field.is_some() && absent_call.is_some(), "{}: an absent member must be rejected", receiver.ty);
        let fields = candidates.iter().filter(|name| answer(field_form, name, receiver.parameter) != absent_field).cloned().collect::<BTreeSet<_>>();
        let prelude = prelude(receiver.parameter);
        let before = format!("{prelude}function probe({}) {{\n  x.", receiver.parameter);
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
            if checked=="closure" {
                if !callable_field_agrees(receiver,name,shown) {failures.push(format!("{}.{}: displayed callback type `{shown}` disagrees with its checked contract",receiver.ty,name));}
                continue;
            }
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
