//! Completion at typing speed. Every row runs against the real server in
//! four modes and must answer the same in each: typed without a pause, typed
//! at 80 ms per key, opened whole and settled, and settled with a syntax
//! error elsewhere in the file.
mod support;
use std::collections::BTreeSet;
use std::time::Duration;
use support::session::Session;

/// What kind of receiver or position a row covers. A new kind needs a row:
/// `every_kind_and_std_module_has_a_row` fails otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Sqlite,
    Http,
    Task,
    Job,
    AsyncChannel,
    AsyncSender,
    Local,
    Keyword,
    ImportedValue,
    ImportedMember,
    RecordField,
    ArrayMember,
    StringMember,
    MapMember,
    SetMember,
    ResultMember,
    ResultPayload,
    UnionNarrowedByIf,
    MatchOkBinding,
    MatchErrBinding,
    MatchArms,
    CatchBinding,
    CallbackParameter,
    IndexedElement,
    CallResult,
    RecordLiteral,
    TypePosition,
    ImportNames,
    ImportPath,
    BuiltinNamespace,
    StdModule,
    MemberPrefix,
}

const KINDS: [Kind; 32] = [
    Kind::Sqlite,
    Kind::Http,
    Kind::Task,
    Kind::Job,
    Kind::AsyncChannel,
    Kind::AsyncSender,
    Kind::Local,
    Kind::Keyword,
    Kind::ImportedValue,
    Kind::ImportedMember,
    Kind::RecordField,
    Kind::ArrayMember,
    Kind::StringMember,
    Kind::MapMember,
    Kind::SetMember,
    Kind::ResultMember,
    Kind::ResultPayload,
    Kind::UnionNarrowedByIf,
    Kind::MatchOkBinding,
    Kind::MatchErrBinding,
    Kind::MatchArms,
    Kind::CatchBinding,
    Kind::CallbackParameter,
    Kind::IndexedElement,
    Kind::CallResult,
    Kind::RecordLiteral,
    Kind::TypePosition,
    Kind::ImportNames,
    Kind::ImportPath,
    Kind::BuiltinNamespace,
    Kind::StdModule,
    Kind::MemberPrefix,
];

struct Row {
    kind: Kind,
    name: &'static str,
    /// Text before the cursor that is already in the document.
    before: String,
    /// Text typed at the cursor, one change per character.
    typed: &'static str,
    after: &'static str,
    includes: &'static [&'static str],
    excludes: &'static [&'static str],
    /// No label is offered here by design; the reason is reviewed with the row.
    silent: Option<&'static str>,
    /// The answer needs a check of the rest of the document. With a syntax
    /// error elsewhere the row may offer nothing, but never a wrong name.
    needs_check: bool,
}

/// Names that are never members. A member position that offers them has
/// fallen back to scope completion.
const NOT_MEMBERS: &[&str] = &["function", "const", "main", "greeting"];

const SHAPES: &str = "export type Point = { x: int; y: int; };\nexport function origin(): Point { return { x: 0, y: 0 }; }\nexport function scale(point: ReadOnly<Point>, factor: int): Point { return { x: point.x * factor, y: point.y * factor }; }\n";
const GREETING: &str = "export function greeting(): string { return \"hi\"; }\n";
const FILES: [(&str, &str); 6] = [("shapes.tsr", SHAPES), ("greeting.tsr", GREETING), ("compute.rs", "pub fn echo(value:String)->String{value}"), ("api.bindings.json",r#"{"schemaVersion":1,"source":"./api.rs","dependencies":{}}"#),("api.bindings.lock","version = 4"),("api.rs","pub struct Reply{pub text:String,pub count:i64}pub fn reply()->Reply{Reply{text:String::new(),count:0}}")];

const HEAD: &str = "import { greeting } from './greeting.tsr';\nimport { Point, origin, scale } from './shapes.tsr';\nimport { s } from 'std:schema';\n\ntype Item = { id: int; name: string; };\nconst schema = s.string();\n\nfunction helper(count: int): int { return count; }\n\nfunction main() {\n  const values: int[] = [1, 2, 3];\n  const item: Item = { id: 1, name: \"a\" };\n  const text = \"abc\";\n  const lookup = new Map<string, int>();\n  const seen = new Set<int>();\n  const start = origin();\n  const parsed = schema.safeParse(greeting());\n  const loaded: Result<string, int> = Ok(greeting());\n  ";
const TAIL: &str = "\n}\n";

fn rows() -> Vec<Row> {
    let row = |kind, name, typed, includes, excludes| Row { kind, name, before: HEAD.into(), typed, after: TAIL, includes, excludes, silent: None, needs_check: false };
    let mut rows = vec![
        Row {kind:Kind::Task,name:"lazy task scheduling",before:"async function answer():Task<int>{return 42;}async function main():Task<void>{const task=answer();".into(),typed:"task.",after:";await task;}",includes:&["start","timeout"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::Sqlite,name:"SQLite connection",before:"import {open} from 'std:sqlite';function main(){const db=unwrap(open(':memory:'));".into(),typed:"db.",after:";}",includes:&["prepare","begin","close"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::Http,name:"HTTP app routes",before:"import {App} from 'std:http';function main(){const app=new App();".into(),typed:"app.",after:";}",includes:&["get","post","use","listen","fallback"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::Job,name:"started job lifecycle",before:"async function answer():Task<int>{return 42;}async function main():Task<void>{const job=answer().start();".into(),typed:"job.",after:";await job;}",includes:&["cancel","timeout"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::AsyncChannel,name:"async channel methods",before:"import {Channel} from 'std:futures';async function main():Task<void>{const queue=new Channel<int>(2);".into(),typed:"queue.",after:"close();}",includes:&["sender","send","receive","close"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::AsyncSender,name:"async sender methods",before:"import {Channel} from 'std:futures';async function main():Task<void>{const queue=new Channel<int>(2);const sender=queue.sender();await ".into(),typed:"sender.",after:"send(1);}",includes:&["send"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::ImportNames,name:"future imports",before:"import { ".into(),typed:"yi",after:" } from 'std:futures';function main(){}",includes:&["yield"],excludes:&["yieldNow"],silent:None,needs_check:false},
        Row {kind:Kind::ImportedMember,name:"Rust binding struct",before:"import {reply} from './api.bindings.json';function main(){const response=reply();".into(),typed:"response.",after:"text.toUpperCase();}",includes:&["text","count"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row {kind:Kind::ImportedMember,name:"Rust bridge result",before:"import {echo} from './compute.rs';function main(){const message=echo('hello');".into(),typed:"message.",after:"toUpperCase();}",includes:&["length","toUpperCase"],excludes:NOT_MEMBERS,silent:None,needs_check:true},
        Row { kind: Kind::MatchOkBinding, name: "match expression payload", before: "type User={name:string;};function main(){const result:Result<User,string>=Ok({name:'Ada'});const count=match(result){Ok(user)=>".into(), typed: "user.", after: "name.length,Err(_)=>0,};}", includes: &["name"], excludes: NOT_MEMBERS, silent: None, needs_check: false },
        Row { kind: Kind::RecordField, name: "match variant payload", before: "type User={name:string;};type Guest={code:int;};function get(value:User|Guest):int{return match(value){User(user)=>".into(), typed: "user.", after: "name.length,Guest(guest)=>guest.code,};}", includes: &["name"], excludes: &["code","function","const"], silent: None, needs_check: false },
        row(Kind::Local, "local prefix", "val", &["values"], &[]),
        row(Kind::Local, "local function prefix", "hel", &["helper"], &[]),
        row(Kind::Keyword, "keyword prefix", "ret", &["return"], &[]),
        row(Kind::ImportedValue, "imported function prefix", "sca", &["scale"], &[]),
        row(Kind::ImportedMember, "member of an imported function's result", "start.", &["x", "y"], NOT_MEMBERS),
        row(Kind::ImportedMember, "member through a new local", "const moved = scale(start, 2);\n  moved.", &["x", "y"], NOT_MEMBERS),
        row(Kind::RecordField, "record field", "item.", &["id", "name"], NOT_MEMBERS),
        row(Kind::ArrayMember, "array member", "values.", &["length", "map", "push"], NOT_MEMBERS),
        row(Kind::ArrayMember, "new array local", "const fresh = [1, 2];\n  fresh.", &["length", "map"], NOT_MEMBERS),
        row(Kind::StringMember, "string member", "text.", &["length", "trim", "toUpperCase"], NOT_MEMBERS),
        row(Kind::MapMember, "map member", "lookup.", &["get", "set", "size"], NOT_MEMBERS),
        row(Kind::SetMember, "set member", "seen.", &["add", "has", "size"], NOT_MEMBERS),
        row(Kind::ResultMember, "result member", "loaded.", &["payload"], NOT_MEMBERS),
        row(Kind::MemberPrefix, "result member prefix in a call", "console.log(loaded.pa", &["payload"], NOT_MEMBERS),
        row(Kind::ResultMember, "schema outcome member", "parsed.", &["success"], NOT_MEMBERS),
        row(Kind::ResultMember, "narrowed schema success", "if (parsed.success) { parsed.", &["data"], NOT_MEMBERS),
        row(Kind::ResultMember, "narrowed schema data member", "if (parsed.success) { parsed.data.", &["length", "trim"], NOT_MEMBERS),
        row(Kind::ResultMember, "narrowed schema failure", "if (!parsed.success) { parsed.error.", &["issues", "truncated"], NOT_MEMBERS),
        row(Kind::IndexedElement, "indexed schema issue", "if (!parsed.success) { parsed.error.issues[0].", &["code", "message", "path"], NOT_MEMBERS),
        row(Kind::MemberPrefix, "array member prefix", "const n = values.ma", &["map"], NOT_MEMBERS),
        row(Kind::MatchOkBinding, "ok binding member", "match (loaded) { Ok(value) => { value.", &["length", "trim"], NOT_MEMBERS),
        row(Kind::MatchErrBinding, "err binding member", "match (loaded) { Ok(value) => { } Err(error) => { error.", &["toString"], NOT_MEMBERS),
        row(Kind::MatchArms, "match arms", "match (loaded) { ", &["Ok", "Err"], NOT_MEMBERS),
        row(Kind::IndexedElement, "indexed scalar element", "values[0].", &["toString"], NOT_MEMBERS),
        Row { needs_check: true, ..row(Kind::CallbackParameter, "callback parameter in an open closure", "values.forEach((entry) => { entry.", &["toString"], NOT_MEMBERS) },
        row(Kind::CallResult, "chained call result", "values.filter((v) => v > 1).", &["length", "map"], NOT_MEMBERS),
        row(Kind::RecordLiteral, "record literal fields", "const other: Item = { ", &["id", "name"], &[]),
        row(Kind::TypePosition, "type position in a declaration", "const next: It", &["Item"], &["values", "helper"]),
        row(Kind::BuiltinNamespace, "console", "console.", &["log", "error"], NOT_MEMBERS),
        row(Kind::BuiltinNamespace, "Math", "const m = Math.", &["PI", "abs", "max"], NOT_MEMBERS),
        row(Kind::StdModule, "schema builder", "const built = s.", &["string", "int", "object"], NOT_MEMBERS),
        row(Kind::StdModule, "schema value", "schema.", &["parse", "safeParse"], NOT_MEMBERS),
    ];
    rows.push(Row {
        kind: Kind::ResultPayload,
        name: "result payload member",
        before: HEAD.into(),
        typed: "parsed.payload.",
        after: TAIL,
        includes: &[],
        excludes: NOT_MEMBERS,
        silent: Some("ValidationError | string has no member that both alternatives accept"),
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::UnionNarrowedByIf,
        name: "nullable narrowed by if",
        before: format!("{HEAD}const maybe: Item | null = item;\n  if (maybe !== null) {{ "),
        typed: "maybe.",
        after: " }\n}\n",
        includes: &["id", "name"],
        excludes: NOT_MEMBERS,
        silent: None,
        needs_check: true,
    });
    rows.push(Row {
        kind: Kind::CatchBinding,
        name: "catch binding member",
        before: format!("{HEAD}try {{ helper(1); }} catch (error) {{ "),
        typed: "error.",
        after: " }\n}\n",
        includes: &[],
        excludes: NOT_MEMBERS,
        silent: Some("a caught value has no declared type"),
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::TypePosition,
        name: "type position in a parameter",
        before: "type Item = { id: int; };\nfunction take(a: ".into(),
        typed: "It",
        after: ") {}\nfunction main() {}\n",
        includes: &["Item"],
        excludes: &["take", "main"],
        silent: None,
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::ImportNames,
        name: "import names of a std module",
        before: "import { ".into(),
        typed: "stric",
        after: " } from 'std:testing';\nfunction main() {}\n",
        includes: &["strictEqual"],
        excludes: &[],
        silent: None,
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::ImportNames,
        name: "import names of a project module",
        before: "import { ".into(),
        typed: "sc",
        after: " } from './shapes.tsr';\nfunction main() {}\n",
        includes: &["scale"],
        excludes: &[],
        silent: None,
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::ImportPath,
        name: "import path of a std module",
        before: "import { strictEqual } from '".into(),
        typed: "std:te",
        after: "';\nfunction main() {}\n",
        includes: &["std:testing"],
        excludes: &[],
        silent: None,
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::ImportPath,
        name: "import path of a project module",
        before: "import { scale } from '".into(),
        typed: "./sh",
        after: "';\nfunction main() {}\n",
        includes: &["./shapes.tsr"],
        excludes: &[],
        silent: None,
        needs_check: false,
    });
    rows.push(Row {
        kind: Kind::ImportNames,
        name: "thread imports",
        before: "import { ".into(), typed: "spa",
        after: " } from 'std:threads';\nfunction main() {}\n",
        includes: &["spawn", "spawnScoped"], excludes: &[], silent: None, needs_check: false,
    });
    rows.push(Row {
        kind: Kind::RecordField,
        name: "thread error fields",
        before: "import {ThreadError} from 'std:threads';\nfunction probe(error: ReadOnly<ThreadError<string>>) {\n  ".into(),
        typed: "error.", after: "\n}\nfunction main() {}\n",
        includes: &["code", "error"], excludes: NOT_MEMBERS, silent: None, needs_check: false,
    });
    rows.push(Row {
        kind: Kind::CallResult, name: "inferred worker context",
        before: "import {spawn} from 'std:threads';function main(){const worker=unwrap(spawn((task):void=>{".into(),
        typed: "task.", after: "\n}));}\n", includes: &["cancellationRequested"], excludes: NOT_MEMBERS, silent: None, needs_check: false,
    });
    for (name, before, typed, includes) in [
        ("channel receiver methods", "import {Channel} from 'std:threads';function main(){const inbox=new Channel<int>(2);", "inbox.", &["sender", "receive", "tryReceive", "close"][..]),
        ("channel sender methods", "import {Channel} from 'std:threads';function main(){const inbox=new Channel<int>(2);const output=inbox.sender();", "output.", &["send", "trySend", "close"][..]),
        ("worker handle methods", "import {spawn} from 'std:threads';function main(){const worker=unwrap(spawn(():int=>42));", "worker.", &["join", "cancel", "detach"][..]),
        ("worker context methods", "import {TaskContext} from 'std:threads';function probe(task:ReadOnly<TaskContext>){", "task.", &["cancellationRequested"][..]),
    ] {
        rows.push(Row {kind: Kind::CallResult, name, before: before.into(), typed, after: "\n}\n", includes, excludes: NOT_MEMBERS, silent: None, needs_check: false});
    }
    rows.push(Row {
        kind: Kind::CallResult, name: "default pool methods",
        before: "import {ThreadPool as Workers} from 'std:threads';\nfunction main(){const workers=new Workers();\n".into(),
        typed: "workers.", after: "\n}\n",
        includes: &["task", "stats", "shutdown"], excludes: &["configure", "create"], silent: None, needs_check: false,
    });
    rows.push(Row {
        kind: Kind::CallResult, name: "scoped pool methods",
        before: "import {ThreadPool} from 'std:threads';\nfunction main(){const jobs=new ThreadPool({workers: 1, capacity: 4});\n".into(),
        typed: "jobs.", after: "\n}\n",
        includes: &["task", "stats", "shutdown"], excludes: &["configure", "create"], silent: None, needs_check: true,
    });
    rows.push(Row {
        kind: Kind::RecordField, name: "pool statistics fields",
        before: "import {ThreadPool} from 'std:threads';\nfunction main(){const pool=new ThreadPool();const stats=pool.stats();\n".into(),
        typed: "stats.", after: "\n}\n",
        includes: &["workers", "capacity", "queued", "running", "completed", "outstanding"], excludes: NOT_MEMBERS, silent: None, needs_check: true,
    });
    rows.push(Row {
        kind: Kind::ImportNames, name: "result propagation imports",
        before: "import { ".into(), typed: "unw",
        after: " } from 'std:result';\nfunction main() {}\n",
        includes: &["unwrap"], excludes: &[], silent: None, needs_check: false,
    });
    rows.push(Row {
        kind: Kind::CallResult, name: "unwrapped result members",
        before: "function text():Result<string,int>{return Ok('hello');}\nfunction main(){const value=unwrap(text());\n".into(),
        typed: "value.", after: "\n}\n",
        includes: &["length", "slice"], excludes: NOT_MEMBERS, silent: None, needs_check: false,
    });
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Immediate,
    Paced,
    Settled,
    ErrorElsewhere,
}

const BROKEN: &str = "function broken() { const = ; }\n";

fn observe(row: &Row, mode: Mode) -> Result<BTreeSet<String>, String> {
    let mut session = Session::start(&FILES);
    let name = "main.tsr";
    let after = if mode == Mode::ErrorElsewhere { format!("{}{BROKEN}", row.after) } else { row.after.to_owned() };
    let text = format!("{}{}{after}", row.before, row.typed);
    let offset = row.before.len() + row.typed.len();
    match mode {
        Mode::Immediate | Mode::Paced => {
            session.open(name, &format!("{}{after}", row.before));
            session.settle();
            let delay = if mode == Mode::Paced { Duration::from_millis(80) } else { Duration::ZERO };
            session.type_text(name, &row.before, row.typed, &after, delay);
        }
        Mode::Settled | Mode::ErrorElsewhere => {
            session.open(name, &text);
            session.settle();
        }
    }
    session.completion(name, &text, offset).map(|labels| labels.into_iter().collect())
}

fn failures(row: &Row, mode: Mode, labels: &Result<BTreeSet<String>, String>) -> Vec<String> {
    let labels = match labels {
        Ok(labels) => labels,
        Err(error) => return vec![format!("request failed: {error}")],
    };
    let mut failures = Vec::new();
    let missing = row.includes.iter().filter(|label| !labels.contains(**label)).collect::<Vec<_>>();
    let unanswerable = mode == Mode::ErrorElsewhere && row.needs_check && labels.is_empty();
    if !missing.is_empty() && !unanswerable {
        failures.push(format!("missing {missing:?}"));
    }
    let unexpected = row.excludes.iter().filter(|label| labels.contains(**label)).collect::<Vec<_>>();
    if !unexpected.is_empty() {
        failures.push(format!("offers non-members {unexpected:?}"));
    }
    if row.silent.is_none() && row.includes.is_empty() {
        failures.push("row has no expectation".into());
    }
    if row.silent.is_some() && !labels.is_empty() && mode == Mode::Settled {
        failures.push(format!("declared silent but offers {} labels", labels.len()));
    }
    failures
}

#[test]
fn every_kind_and_std_module_has_a_row() {
    let rows = rows();
    for kind in KINDS {
        assert!(rows.iter().any(|row| row.kind == kind), "no completion row for {kind:?}");
    }
    for row in &rows {
        assert!(row.silent.is_some() || !row.includes.is_empty(), "{}: no expectation", row.name);
    }
    // Every importable std module needs an import-path expectation. The list
    // comes from the compiler, so a new module cannot land without a row.
    // `std:schema` and `std:testing` have no native binding of their own name.
    let modules = tsrust_compiler::native_bindings::BINDINGS.iter().map(|binding| binding.module).filter(|module| module.starts_with("std:") && !module.ends_with("-runtime")).chain(["std:schema", "std:testing"]).collect::<BTreeSet<_>>();
    let mut session = Session::start(&FILES);
    let before = "import { x } from '";
    let text = format!("{before}std:';\nfunction main() {{}}\n");
    session.open("main.tsr", &text);
    session.settle();
    let offered = session.completion("main.tsr", &text, before.len() + 4).unwrap().into_iter().collect::<BTreeSet<_>>();
    let missing = modules.iter().filter(|module| !offered.contains(**module)).collect::<Vec<_>>();
    assert!(missing.is_empty(), "import path completion does not offer {missing:?}");
}

#[test]
fn completion_answers_at_typing_speed() {
    let report = std::env::var("TSRUST_LSP_MATRIX_REPORT").ok();
    let only = std::env::var("TSRUST_LSP_MATRIX_ROW").ok();
    let mut lines = Vec::new();
    let mut failed = Vec::new();
    for row in rows() {
        if only.as_ref().is_some_and(|only| !row.name.contains(only.as_str())) {
            continue;
        }
        let settled = observe(&row, Mode::Settled);
        for mode in [Mode::Settled, Mode::Immediate, Mode::Paced, Mode::ErrorElsewhere] {
            let labels = if mode == Mode::Settled { settled.clone() } else { observe(&row, mode) };
            let mut problems = failures(&row, mode, &labels);
            // The answer must not depend on how fast the text arrived.
            if matches!(mode, Mode::Immediate | Mode::Paced) && labels != settled {
                problems.push("differs from the settled answer".into());
            }
            let count = labels.as_ref().map_or(0, BTreeSet::len);
            lines.push(format!("{:?}\t{}\t{mode:?}\t{count}\t{}", row.kind, row.name, if problems.is_empty() { "ok".into() } else { problems.join("; ") }));
            if !problems.is_empty() {
                failed.push(format!("{} [{mode:?}]: {}", row.name, problems.join("; ")));
            }
        }
    }
    if let Some(path) = report {
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
        return;
    }
    assert!(failed.is_empty(), "{} completion failures:\n{}", failed.len(), failed.join("\n"));
}
