//! Program-owned globals: full-section witnesses, not parser-only acceptance.
mod support;
use std::fs;

#[test]
fn module_schema_and_mutable_state_are_reused_across_imports() {
    let dir = support::TempDirectory::new();
    fs::write(dir.0.join("state.tsr"), support::format_round_trip(r#"
import { s } from 'std:schema';
export const person = s.object({ name: s.string(), age: s.number() });
export const settings = { retries: 3 };
export let requests = 0;
export function recordRequest() { requests++; }
export function configure() { settings.retries = 5; }
"#)).unwrap();
    let main = r#"
import { strictEqual } from 'std:testing';
import { person, requests, settings, recordRequest, configure } from './state.tsr';
function main() {
  recordRequest(); recordRequest(); configure();
  strictEqual(requests, 2);
  strictEqual(settings.retries, 5);
  const copy = clone(settings);
  copy.retries = 9;
  strictEqual(settings.retries, 5);
  match (person.safeParseJson('{"name":"Sean","age":30}')) {
    Ok(value) => { strictEqual(value.name, 'Sean'); },
    Err(error) => { throw 'Global schema rejected valid input'; }
  }
  match (person.safeParseJson('{"name":"Again","age":31}')) {
    Ok(value) => { strictEqual(value.name, 'Again'); },
    Err(error) => { throw 'Global schema was not reusable'; }
  }
}
"#;
    let file = dir.0.join("main.tsr");
    fs::write(&file, support::format_round_trip(main)).unwrap();
    support::success(&support::run(support::cli().arg("run").arg(file), 60));
}

#[test]
fn conflicting_indirect_global_mutation_is_rejected() {
    let dir = support::TempDirectory::new();
    let file = dir.source(r#"
const values = [1, 2, 3];
function reset() { values.push(4); }
function inspect(input: int[]) {
  reset();
  console.log(input.length);
}
function main() { inspect(values); }
"#);
    let output = support::run(support::cli().arg("check").arg(file), 30);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(!diagnostic.contains("Expected a type or function declaration"), "{diagnostic}");
    assert!(diagnostic.contains("borrow") || diagnostic.contains("access"), "{diagnostic}");
}

#[test]
fn imported_live_binding_cannot_be_reassigned_by_importer() {
    let dir = support::TempDirectory::new();
    fs::write(dir.0.join("state.tsr"), "export let count = 0;").unwrap();
    let file = dir.0.join("main.tsr");
    fs::write(&file, "import {count} from './state.tsr';function main(){count=1;}").unwrap();
    let output = support::run(support::cli().arg("check").arg(file), 30);
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("import") && (diagnostic.contains("assign") || diagnostic.contains("rebind")), "{diagnostic}");
}

#[test]
fn diamond_imports_share_one_live_owner_and_default_binding() {
    let dir = support::TempDirectory::new();
    for (name, source) in [
        ("state.tsr", "export let count=0;export function increment(){count++;}const settings={limit:4};export default settings;"),
        ("left.tsr", "import {increment} from './state.tsr';export const initialized=initialize();function initialize():int{increment();return 1;}"),
        ("right.tsr", "import {increment} from './state.tsr';export const initialized=initialize();function initialize():int{increment();return 2;}"),
        ("main.tsr", "import {strictEqual} from 'std:testing';import {initialized as left} from './left.tsr';import {initialized as right} from './right.tsr';import settings from './state.tsr';import {count} from './state.tsr';function main(){strictEqual(left,1);strictEqual(right,2);strictEqual(count,2);strictEqual(settings.limit,4);}"),
    ] { fs::write(dir.0.join(name), support::format_round_trip(source)).unwrap(); }
    let output = support::run(support::cli().arg("run").arg(dir.0.join("main.tsr")), 60);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn module_initialization_rejects_indirect_early_reads_and_consumption() {
    let dir = support::TempDirectory::new();
    for (source, expected) in [
        ("const first=read();const later=4;function read():int{return later;}function main(){console.log(first);}", "initial"),
        ("const values=[1,2];function take():int[]{return values;}function main(){console.log(take());}", "consum"),
    ] {
        let file = dir.0.join("main.tsr");
        fs::write(&file, support::format_round_trip(source)).unwrap();
        let output = support::run(support::cli().arg("check").arg(file), 30);
        assert!(!output.status.success());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(diagnostic.contains(expected), "{diagnostic}");
    }
}

#[test]
fn documented_module_state_fixture_executes() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/module-state");
    let dir = support::TempDirectory::new();
    for name in ["state.tsr", "main.tsr"] {
        let source = fs::read_to_string(root.join(name)).unwrap();
        fs::write(dir.0.join(name), support::format_round_trip(&source)).unwrap();
    }
    let output = support::run(support::cli().arg("run").arg(dir.0.join("main.tsr")), 60);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn module_callbacks_recursion_and_shadowing_preserve_live_state() {
    let dir = support::TempDirectory::new();
    let file = dir.source(&support::format_round_trip(r#"
import { strictEqual } from 'std:testing';
let count = 0;
const values = [1, 2, 3];
function add(n: int): void {
  if (n > 0) { count++; add(n - 1); }
}
function apply(action: Mutable<Function<void, []>>) { action(); }
function shadow(): int { function read():int{return count;} const count = 99; return read(); }
function factorial():int{const recur=function fact(n:int):int{return n<=1?1:n*fact(n-1);};return recur(4);}
function main() {
  add(3);
  apply(() => { count++; });
  strictEqual(count, 4);
  strictEqual(shadow(), 99);
  strictEqual(factorial(), 24);
  const size = () => values.length;
  strictEqual(size(), 3);
}
"#));
    let output = support::run(support::cli().arg("run").arg(file), 60);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn initialization_runs_without_main_and_failure_prevents_entry() {
    let dir = support::TempDirectory::new();
    let file = dir.0.join("main.tsr");
    fs::write(&file, support::format_round_trip("const ready=initialize();function initialize():int{console.log('initialized');return 1;}")).unwrap();
    let output = support::run(support::cli().arg("run").arg(&file), 60);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "initialized");
    fs::write(&file, support::format_round_trip("const ready=initialize();function initialize():int{throw 'startup failed';}function main(){console.log('ENTRY RAN');}")).unwrap();
    let output = support::run(support::cli().arg("run").arg(&file), 60);
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("ENTRY RAN"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("startup failed"));
}

#[test]
fn inferred_module_object_context_does_not_leak_into_called_function() {
    let dir = support::TempDirectory::new();
    let file = dir.source("const item=make();function make(){return {count:1};}function main(){}");
    let output = support::run(support::cli().arg("check").arg(file), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("record literal needs"));
}

#[test]
fn owned_callbacks_cannot_snapshot_or_steal_module_storage() {
    let dir = support::TempDirectory::new();
    let file = dir.source("let count=0;function apply(action:Function<void,[]>){action();}function main(){apply(()=>{count++;});}");
    let output = support::run(support::cli().arg("check").arg(file), 30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("owned callback"));
}
