use std::cell::RefCell;
use std::collections::BTreeMap;
use tsrust_compiler::project::{SourceId, SourceProvider, load_project};

struct Memory {
    files: BTreeMap<String, String>,
    reads: RefCell<BTreeMap<String, usize>>,
}
impl Memory {
    fn new(files: &[(&str, &str)]) -> Self {
        Self {
            files: files
                .iter()
                .map(|(id, text)| (id.to_string(), text.to_string()))
                .collect(),
            reads: RefCell::new(BTreeMap::new()),
        }
    }
}
impl SourceProvider for Memory {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let base = importer
            .and_then(|id| id.0.rsplit_once('/').map(|(p, _)| p))
            .unwrap_or("");
        let combined = if importer.is_some() {
            format!("{base}/{path}")
        } else {
            path.to_string()
        };
        let mut parts = vec![];
        for part in combined.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                p => parts.push(p),
            }
        }
        let key = parts.join("/");
        if self.files.contains_key(&key) {
            Ok(SourceId(key))
        } else {
            Err(format!("Missing source {key}"))
        }
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        *self.reads.borrow_mut().entry(id.0.clone()).or_default() += 1;
        self.files
            .get(&id.0)
            .cloned()
            .ok_or("Missing snapshot".into())
    }
}
#[test]
fn diamond_aliases_exports_and_byte_coordinates() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "// 🦀\nimport { left } from './left.tsr';\nimport { right } from './right.tsr';\nfunction main() { console.log(left() + right()); }",
        ),
        (
            "left.tsr",
            "import { value as number } from './shared.tsr'; export function left(): int { return number(); }",
        ),
        (
            "right.tsr",
            "import { value } from './shared.tsr'; export function right(): int { return value(); }",
        ),
        (
            "shared.tsr",
            "export type Row = { value: int; }; export function value(): int { return 2; } function main() {}",
        ),
    ]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    assert_eq!(
        graph
            .order
            .iter()
            .map(|id| id.0.as_str())
            .collect::<Vec<_>>(),
        ["shared.tsr", "left.tsr", "right.tsr", "main.tsr"]
    );
    assert!(memory.reads.borrow().values().all(|&count| count == 1));
    let shared = &graph.modules[&SourceId("shared.tsr".into())];
    assert!(!shared.exports.contains_key("main"));
    let span = shared.exports["value"];
    assert_eq!(&shared.source.text[span.start..span.end], "value");
    assert_eq!(&shared.declaration_source[span.start..span.end], "value");
    for module in graph.modules.values() {
        assert_eq!(module.source.text.len(), module.declaration_source.len());
    }
    assert_eq!(
        graph.modules[&SourceId("left.tsr".into())].imports[0].bindings[0].local,
        "number"
    );
}
#[test]
fn cycles_missing_sources_private_names_and_duplicate_bindings() {
    for (source, dependency, code) in [
        (
            "import { x } from './dep.tsr'; function main(){}",
            "import { main } from './main.tsr'; export function x(){}",
            "module-cycle",
        ),
        (
            "import { x } from './absent.tsr'; function main(){}",
            "export function x(){}",
            "module-load",
        ),
        (
            "import { x } from './dep.tsr'; function main(){}",
            "function x(){}",
            "module-private",
        ),
        (
            "import { x as main } from './dep.tsr'; function main(){}",
            "export function x(){}",
            "module-duplicate",
        ),
        (
            "import { x, x } from './dep.tsr'; function main(){}",
            "export function x(){}",
            "module-duplicate",
        ),
        (
            "export function main(){} export function main(){}",
            "",
            "module-duplicate",
        ),
    ] {
        let memory = Memory::new(&[("main.tsr", source), ("dep.tsr", dependency)]);
        let diagnostic = load_project(&memory, "main.tsr").unwrap_err();
        assert_eq!(diagnostic.code, code, "{source}");
        assert!(memory.files.contains_key(&diagnostic.location.source.0));
        if code == "module-cycle" {
            assert!(!diagnostic.related.is_empty());
        }
    }
}
#[test]
fn unsupported_forms_and_malformed_imports_are_source_errors() {
    for (source, code) in [
        ("import x from './dep.tsr';", "module-syntax"),
        ("import { x } from 'dep';", "module-path"),
        ("import { x } from './dep';", "module-path"),
        ("import { x } from 'std:path';", "module-native"),
        ("export const x = 1;", "module-export"),
        ("export { x };", "module-export"),
        ("import", "module-syntax"),
        ("import { x as", "module-syntax"),
        ("import { x } from", "module-syntax"),
        ("export", "module-export"),
        ("const x = 1;", "unsupported"),
    ] {
        let memory = Memory::new(&[("main.tsr", source)]);
        assert_eq!(
            load_project(&memory, "main.tsr").unwrap_err().code,
            code,
            "{source}"
        );
    }
}
#[test]
fn import_words_in_comments_strings_and_templates_are_not_declarations() {
    let source = "/* import { bad } from './bad.tsr'; */ function main() { console.log('export function nope'); console.log(`import ${1}`); }";
    let memory = Memory::new(&[("main.tsr", source)]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    assert!(graph.modules[&graph.entry].imports.is_empty());
}
#[test]
fn graph_loading_does_not_claim_type_checking() {
    let memory = Memory::new(&[("main.tsr", "function main() { console.log(unknown); }")]);
    assert!(load_project(&memory, "main.tsr").is_ok());
    assert!(tsrust_compiler::check(&memory.files["main.tsr"]).is_err());
}

#[test]
fn filesystem_provider_loads_retained_diamond_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/modules/diamond/main.tsr"
    );
    let graph = load_project(&tsrust_compiler::project::FileSourceProvider, path).unwrap();
    assert_eq!(graph.modules.len(), 4);
    assert!(
        graph
            .modules
            .keys()
            .all(|id| std::path::Path::new(&id.0).is_absolute())
    );
    assert!(graph.order.last().unwrap() == &graph.entry);
}

mod support;
#[test]
fn linked_diamond_executes_only_entry_main() {
    let graph = load_project(
        &tsrust_compiler::project::FileSourceProvider,
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/modules/diamond/main.tsr"
        ),
    )
    .unwrap();
    let checked = tsrust_compiler::project::check_graph(graph)
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    let output = native_stdout(&checked.emit_rust(), "0");
    assert_eq!(output, "4\n");
}
#[test]
fn linked_types_are_nominal_and_errors_keep_original_files() {
    let files = [
        (
            "main.tsr",
            "import { User, take } from './user.tsr'; function main(){const user:User={name:\"Ada\"};const saved=take(user);console.log(user.name);}",
        ),
        (
            "user.tsr",
            "export type User={name:string;}; export function take(user:User):User{return user;}",
        ),
    ];
    let memory = Memory::new(&files);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .unwrap();
    assert_eq!(error.code, "transferred");
    assert_eq!(error.location.source.0, "main.tsr");
    let span = error.location.span;
    assert!(memory.files["main.tsr"][span.start..span.end].contains("user"));
    assert!(!error.related.is_empty());
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { User, make } from './a.tsr'; import { take } from './b.tsr'; function main(){const user:User=make();take(user);}",
        ),
        (
            "a.tsr",
            "export type User={name:string;};export function make():User{return {name:\"Ada\"};}",
        ),
        (
            "b.tsr",
            "type User={name:string;};export function take(user:User){console.log(user.name);}",
        ),
    ]);
    assert_eq!(
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
            .err()
            .unwrap()
            .code,
        "type"
    );
}
#[test]
fn linked_aliases_local_shadowing_and_closures_execute() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { value as answer } from './a.tsr'; function main(){console.log(answer());{const answer=3;console.log(answer);}const f=():int=>{return answer();};console.log(f());}",
        ),
        (
            "a.tsr",
            "export function value():int{const f=():int=>{return 9;};return f();}",
        ),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "9\n3\n9\n");
}
#[test]
fn dependency_semantic_error_is_not_reported_in_entry() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { value } from './a.tsr'; function main(){console.log(value());}",
        ),
        (
            "a.tsr",
            "export function value():int{return missing;} function main():int{return 9;}",
        ),
    ]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .unwrap();
    assert_eq!(error.location.source.0, "a.tsr");
    let span = error.location.span;
    assert_eq!(&memory.files["a.tsr"][span.start..span.end], "missing");
}

fn native_stdout(rust: &str, optimization: &str) -> String {
    let directory = support::TempDirectory::new();
    let path = directory.0.join("program.rs");
    let binary = directory.0.join("program");
    std::fs::write(&path, rust).unwrap();
    let compiled = support::run(
        support::rustc()
            .arg("--edition=2024")
            .arg("-C")
            .arg(format!("opt-level={optimization}"))
            .arg(&path)
            .arg("-o")
            .arg(&binary),
        60,
    );
    support::success(&compiled);
    let result = support::run(&mut std::process::Command::new(binary), 10);
    support::success(&result);
    assert!(result.stderr.is_empty());
    String::from_utf8(result.stdout).unwrap()
}

#[test]
fn nullable_imported_types_callbacks_and_recursion_execute() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { User, make, count, apply } from './a.tsr';function main(){const user:User|null=make();if(user!==null){console.log(user.name);}console.log(count(4));console.log(apply((n:int):int=>{return n+1;},8));}",
        ),
        (
            "a.tsr",
            "export type User={name:string;};export function make():User{return {name:\"Ada\"};}export function count(n:int):int{if(n===0){return 0;}return 1+count(n-1);}export function apply(f:(n:int)=>int,n:int):int{return f(n);}",
        ),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    for optimization in ["0", "3"] {
        assert_eq!(
            native_stdout(&checked.emit_rust(), optimization),
            "Ada\n4\n9\n"
        );
    }
}
#[test]
fn private_symbols_reserved_names_and_missing_entry_are_rejected() {
    for (entry, dependency, code) in [
        (
            "import { visible } from './a.tsr';function main(){hidden();}",
            "export function visible(){}function hidden(){}",
            "unknown",
        ),
        (
            "import { visible } from './a.tsr';function main(){}",
            "export function visible(){}function clone(){}",
            "duplicate",
        ),
        (
            "import { visible } from './a.tsr';function other(){}",
            "export function visible(){}function main(){}",
            "entry",
        ),
    ] {
        let memory = Memory::new(&[("main.tsr", entry), ("a.tsr", dependency)]);
        let error =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .err()
                .unwrap();
        if code == "unknown" {
            assert!(error.message.contains("hidden"));
            assert!(!error.message.contains("__tsrust_module_"));
        } else {
            assert_eq!(error.code, code);
        }
    }
}

#[test]
fn cli_project_check_lint_emit_build_and_run_preserve_single_file_contracts() {
    let directory = support::TempDirectory::new();
    let entry = directory.0.join("entry file.tsr");
    let dependency = directory.0.join("dependency.tsr");
    std::fs::write(
        &entry,
        "import { value } from './dependency.tsr';function main(){console.log(value());}",
    )
    .unwrap();
    std::fs::write(
        &dependency,
        "export function value():int{return 7;}function main(){console.log(99);}",
    )
    .unwrap();
    let checked = support::run(support::cli().arg("check").arg(&entry).arg("--json"), 10);
    support::success(&checked);
    assert!(String::from_utf8_lossy(&checked.stdout).contains("\"ok\":true"));
    assert!(!String::from_utf8_lossy(&checked.stdout).contains("__tsrust_module_"));
    let lint = support::run(support::cli().arg("lint").arg(&entry), 10);
    support::success(&lint);
    assert!(lint.stdout.is_empty());
    assert!(lint.stderr.is_empty());
    let emitted = support::run(support::cli().arg("emit").arg(&entry), 10);
    support::success(&emitted);
    assert_eq!(
        native_stdout(&String::from_utf8(emitted.stdout).unwrap(), "3"),
        "7\n"
    );
    let output = support::run(support::cli().arg("run").arg(&entry), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"7\n");
    let binary = directory.0.join("app");
    support::success(&support::run(
        support::cli()
            .arg("build")
            .arg(&entry)
            .arg("-o")
            .arg(&binary),
        60,
    ));
    assert_eq!(
        support::run(&mut std::process::Command::new(binary), 10).stdout,
        b"7\n"
    );
}
#[test]
fn cli_dependency_compile_and_runtime_errors_identify_original_source() {
    let directory = support::TempDirectory::new();
    let entry = directory.0.join("main.tsr");
    let dependency = directory.0.join("dependency.tsr");
    std::fs::write(
        &entry,
        "import { value } from './dependency.tsr';function main(){console.log(value());}",
    )
    .unwrap();
    std::fs::write(&dependency, "export function value():int{return missing;}").unwrap();
    let error = support::run(support::cli().arg("check").arg(&entry).arg("--json"), 10);
    assert_eq!(error.status.code(), Some(1));
    let json = String::from_utf8(error.stdout).unwrap();
    assert!(json.contains("dependency.tsr"));
    assert!(!json.contains("__tsrust_module_"));
    let source = "export function value():int{const values=[1];return values[5];}";
    std::fs::write(&dependency, source).unwrap();
    for release in [false, true] {
        let mut command = support::cli();
        command.arg("run").arg(&entry);
        if release {
            command.arg("--release");
        }
        let error = support::run(&mut command, 60);
        assert!(!error.status.success());
        let message = String::from_utf8(error.stderr).unwrap();
        assert!(message.contains("runtime error:"), "{message}");
        assert!(message.contains("dependency.tsr"), "{message}");
        let start = source.find("values[5]").unwrap();
        assert!(
            message.contains(&format!(
                "source bytes {start}..{}",
                start + "values[5]".len()
            )),
            "{message}"
        );
        assert!(!message.contains("program.rs"), "{message}");
    }
}

#[test]
fn project_clone_fix_uses_original_file_range_and_repairs_ownership() {
    let mut memory = Memory::new(&[
        (
            "main.tsr",
            "import { User, take } from './user.tsr';function main(){const user:User={name:\"Ada\"};const saved=take(user);console.log(user.name);}",
        ),
        (
            "user.tsr",
            "export type User={name:string;};export function take(user:User):User{return user;}",
        ),
    ]);
    let diagnostic =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
            .err()
            .unwrap();
    let fix = &diagnostic.fixes[0];
    assert_eq!(fix.location.source.0, "main.tsr");
    memory.files.get_mut("main.tsr").unwrap().replace_range(
        fix.location.span.start..fix.location.span.end,
        &fix.replacement,
    );
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}", e.message));
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "Ada\n");
}
#[test]
fn project_metadata_has_source_names_and_capture_ranges() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { make } from './a.tsr';function main(){const f=make(2);console.log(f());}",
        ),
        (
            "a.tsr",
            "export function make(n:int):()=>int{return ():int=>{return n;};}",
        ),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}", e.message));
    let json = checked.to_json();
    assert!(!json.contains("__tsrust_module_"), "{json}");
    assert!(json.contains("a.tsr::make"), "{json}");
    assert!(json.contains("\"file\":\"a.tsr\""), "{json}");
    for closure in checked.closures() {
        assert!(closure.span.end <= memory.files["a.tsr"].len());
    }
}
#[test]
fn emitting_cannot_overwrite_an_imported_source() {
    let directory = support::TempDirectory::new();
    let entry = directory.0.join("main.tsr");
    let dependency = directory.0.join("a.tsr");
    std::fs::write(
        &entry,
        "import { value } from './a.tsr';function main(){value();}",
    )
    .unwrap();
    let source = "export function value(){}";
    std::fs::write(&dependency, source).unwrap();
    let output = support::run(
        support::cli()
            .arg("emit")
            .arg(&entry)
            .arg("-o")
            .arg(&dependency),
        10,
    );
    assert!(!output.status.success());
    assert_eq!(std::fs::read_to_string(dependency).unwrap(), source);
}

#[test]
fn imported_records_arrays_loops_and_callback_specializations_execute() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { Row, sum, apply } from './ops.tsr';function main(){const rows:Row[]=[{values:[1,2]},{values:[3]}];let total=0;for(const row of rows){total=total+sum(row.values);}console.log(total);console.log(apply((n:int):int=>{return n+1;},2));console.log(apply((n:int):int=>{return n*2;},3));}",
        ),
        (
            "ops.tsr",
            "export type Row={values:int[];};export function sum(values:int[]):int{let total=0;for(const value of values){total=total+value;}return total;}export function apply(action:(n:int)=>int,n:int):int{return action(n);}",
        ),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    for optimization in ["0", "3"] {
        assert_eq!(
            native_stdout(&checked.emit_rust(), optimization),
            "6\n3\n6\n"
        );
    }
}
#[test]
fn imported_mutation_contracts_and_retained_capture_loans_are_enforced() {
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { touch } from './ops.tsr';function main(){let values=[1];const read=():int=>{return values[0];};touch(values);console.log(read());}",
        ),
        (
            "ops.tsr",
            "export function touch(values:int[]){values.push(2);}",
        ),
    ]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .expect("mutation conflicts with retained capture");
    assert_eq!(error.location.source.0, "main.tsr");
    assert!(!error.message.contains("__tsrust_module_"));
    let memory = Memory::new(&[
        (
            "main.tsr",
            "import { Row as Imported, make, touch } from './ops.tsr';type Row={other:int;};function main(){let row:Imported=make();touch(row.values);console.log(row.values.length);}",
        ),
        (
            "ops.tsr",
            "export type Row={values:int[];};export function make():Row{return {values:[1]};}export function touch(values:int[]){values.push(2);}",
        ),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "2\n");
}

#[test]
fn editor_header_loading_tolerates_bodies_but_preserves_module_boundaries() {
    use tsrust_compiler::project::load_project_headers;
    let entry = "import { value as local } from './dep.tsr';function main(){local(}";
    let memory = Memory::new(&[
        ("main.tsr", entry),
        ("dep.tsr", "export function value(n:int):int{return n;}"),
    ]);
    assert!(load_project(&memory, "main.tsr").is_err());
    let graph = load_project_headers(&memory, "main.tsr").unwrap();
    assert_eq!(graph.modules.len(), 2);
    assert_eq!(
        graph.modules[&SourceId("main.tsr".into())].source.text,
        entry
    );
    for (entry, dep, code) in [
        (
            "import { hidden } from './dep.tsr';function main(){",
            "function hidden(){}",
            "module-private",
        ),
        (
            "import { value as } from './dep.tsr';",
            "export function value(){}",
            "module-syntax",
        ),
        (
            "import { value } from './dep.tsr';function value(){",
            "export function value(){}",
            "module-duplicate",
        ),
    ] {
        let memory = Memory::new(&[("main.tsr", entry), ("dep.tsr", dep)]);
        let error = load_project_headers(&memory, "main.tsr").err().unwrap();
        assert_eq!(error.code, code, "{error:?}");
    }
}

#[test]
fn native_registry_imports_resolve_without_disk_modules_or_implicit_features() {
    let entry = "import { basename as fileName } from 'std:path';function main(){console.log(fileName('/tmp/file.txt'));}";
    let memory = Memory::new(&[("main.tsr", entry)]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    assert_eq!(graph.modules.len(), 1);
    let import = &graph.modules[&SourceId("main.tsr".into())].imports[0];
    assert_eq!(import.native, Some("std:path"));
    assert!(import.target.is_none());
    assert_eq!(import.bindings[0].exported, "basename");
    assert_eq!(import.bindings[0].local, "fileName");
    assert_eq!(
        &entry[import.bindings[0].span.start..import.bindings[0].span.end],
        "basename"
    );
    assert_eq!(memory.reads.borrow().len(), 1);
    let features = tsrust_compiler::native_bindings::selected_features(
        graph
            .modules
            .values()
            .flat_map(|module| module.imports.iter())
            .filter_map(|import| import.native.map(|native| (native, &import.bindings)))
            .flat_map(|(native, bindings)| {
                bindings
                    .iter()
                    .map(move |binding| (native, binding.exported.as_str()))
            }),
    )
    .unwrap();
    assert_eq!(features.into_iter().collect::<Vec<_>>(), vec!["path"]);
    assert!(tsrust_compiler::project::check_graph(graph).is_ok());
    for (path, name) in [("std:missing", "basename"), ("std:path", "missing")] {
        let text = format!("import {{ {name} }} from '{path}';function main(){{}}");
        let memory = Memory::new(&[("main.tsr", &text)]);
        let error = load_project(&memory, "main.tsr").err().unwrap();
        assert_eq!(error.code, "module-native");
        assert_eq!(
            &text[error.location.span.start..error.location.span.end],
            name
        );
    }
}

#[test]
fn std_path_calls_execute_with_borrowing_aliases_chains_and_type_errors() {
    let entry = "import { basename as fileName } from 'std:path';function main(){const path='/tmp/report.txt';console.log(fileName(path));console.log(fileName(path,'.txt').toUpperCase());console.log(path);}";
    let memory = Memory::new(&[("main.tsr", entry)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    let rust = checked.emit_rust();
    assert_eq!(rust.matches("mod tsrust_native_path {").count(), 1);
    for optimization in ["0", "3"] {
        assert_eq!(
            native_stdout(&rust, optimization),
            "report.txt\nREPORT\n/tmp/report.txt\n"
        );
    }
    for (call, code) in [
        ("fileName()", "arity"),
        ("fileName('a','b','c')", "arity"),
        ("fileName(3)", "type"),
        ("fileName('a',3)", "type"),
    ] {
        let source = format!(
            "import {{ basename as fileName }} from 'std:path';function main(){{console.log({call});}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .err()
                .unwrap();
        assert_eq!(error.code, code, "{error:?}");
        assert_eq!(error.location.source.0, "main.tsr");
    }
    let memory = Memory::new(&[(
        "main.tsr",
        "import { basename } from 'std:path';function main(){}",
    )]);
    let unused = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap()
        .emit_rust();
    assert!(
        !unused.contains("tsrust_native_path"),
        "unused imports must not emit path runtime"
    );
}

#[test]
fn native_runtime_selection_ignores_bridge_names_in_user_literals() {
    let source = "import { basename } from 'std:path';function main(){console.log('tsrust_std_path_basename(');}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    let (rust, features) = checked.emit_rust_with_native_features();
    assert!(features.is_empty());
    assert!(!rust.contains("mod tsrust_native_path"));
    assert_eq!(native_stdout(&rust, "0"), "tsrust_std_path_basename(\n");
}

#[test]
fn native_function_values_preserve_optional_calls_and_callback_adapters() {
    for arguments in ["", "'a','b','c'"] {
        let source = format!(
            "import {{ basename }} from 'std:path';function main(){{console.log((true?basename:basename)({arguments}));}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .err()
                .unwrap();
        assert_eq!(error.code, "arity");
    }
    let source = "import { basename as left,basename as right } from 'std:path';function predicate():boolean{console.log(1);return true;}function main(){const stored=left;console.log((predicate()?left:right)('/tmp/report.txt'));console.log((false?left:stored)('/tmp/report.txt','.txt'));console.log((true?stored:right)('/tmp/selected.txt'));console.log(stored('/tmp/reused.txt'));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(checked.emit_rust_with_native_features().1, vec!["path"]);
        assert_eq!(
            native_stdout(&checked.emit_rust(), optimization),
            "1\nreport.txt\nreport\nselected.txt\nreused.txt\n"
        );
    }
    for kind in ["Function<string, [path:ReadOnly<string>]>", "Once<Function<string, [path:ReadOnly<string>]>>"] {
        let source = format!(
            "import {{ basename }} from 'std:path';function identity(action:{kind}):(path:ReadOnly<string>)=>string{{return action;}}function main(){{const f=basename;const next=identity(f);console.log(next('/tmp/report.txt'));}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let checked =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .unwrap();
        assert_eq!(native_stdout(&checked.emit_rust(), "0"), "report.txt\n");
    }
    let source = "import { basename } from 'std:path';function use(action:Once<Function<string, [path:ReadOnly<string>]>>){console.log(action('/tmp/a'));console.log(action('/tmp/b'));}function main(){use(basename);}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .unwrap();
    assert_eq!(error.code, "callback-consumed");
    let source = "import { basename } from 'std:path';function identity(action:Once<Function<string, [path:ReadOnly<string>]>>):(path:ReadOnly<string>)=>string{return action;}function main(){const next=identity(basename);next('/tmp/a');next('/tmp/b');}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .unwrap();
    assert_eq!(error.code, "closure-consumed");
    for (contract, code) in [
        ("()=>string", "arity"),
        ("(path:int)=>string", "type"),
        ("(path:ReadOnly<string>)=>int", "type"),
    ] {
        let source = format!(
            "import {{ basename }} from 'std:path';function factory():{contract}{{return basename;}}function main(){{}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .err()
                .unwrap();
        assert_eq!(error.code, code);
    }
    for body in [
        "return basename;",
        "const selected=basename;return selected;",
    ] {
        let source = format!(
            "import {{ basename }} from 'std:path';function factory():(path:ReadOnly<string>)=>string{{{body}}}function main(){{const f=factory();console.log(f('/tmp/report.txt'));console.log(f('/tmp/other.txt'));}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let checked =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .unwrap();
        for optimization in ["0", "3"] {
            assert_eq!(
                native_stdout(&checked.emit_rust(), optimization),
                "report.txt\nother.txt\n"
            );
        }
    }

    for (usage, code) in [
        ("f();", "arity"),
        ("f(3);", "type"),
        ("f('a','b','c');", "arity"),
    ] {
        let source = format!(
            "import {{ basename }} from 'std:path';function main(){{const f=basename;{usage}}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error =
            tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
                .err()
                .unwrap();
        assert_eq!(error.code, code);
        assert!(!error.message.contains("@native:"));
    }
    let source = "import { basename } from 'std:path';function apply(action:Mutable<Function<string, [path:ReadOnly<string>]>>):string{return action('/tmp/report.txt');}function main(){const f=basename;console.log(f('/tmp/report.txt'));console.log(f('/tmp/report.txt','.txt'));console.log(apply(basename));console.log(apply(f));console.log(f('/tmp/report.txt','.txt'));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(
        native_stdout(&checked.emit_rust(), "0"),
        "report.txt\nreport\nreport.txt\nreport.txt\nreport\n"
    );
    let source = "import { basename as nameOf } from 'std:path';function apply(action:Mutable<Function<string, [path:ReadOnly<string>,suffix:ReadOnly<string>]>>):string{return action('/tmp/report.txt','.txt');}function main(){const f=nameOf;console.log(apply(f));console.log(f('/tmp/report.txt'));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(
            native_stdout(&checked.emit_rust(), optimization),
            "report\nreport.txt\n"
        );
    }
    let source = "import { basename } from 'std:path';function apply(action:Mutable<Function<string, [path:ReadOnly<string>]>>):string{return action('/tmp/report.txt');}function main(){const f=basename;const next=f;console.log(apply(f));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .err()
        .unwrap();
    assert_eq!(error.code, "closure-consumed");
    let source = "import { basename as nameOf } from 'std:path';function apply(action:Mutable<Function<string, [path:ReadOnly<string>]>>):string{return action('/tmp/report.txt');}function main(){const name=(path:string):string=>{return nameOf(path);};console.log(apply(name));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked =
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "report.txt\n");
}

#[test]
fn native_function_values_cross_capture_and_conditional_return_boundaries() {
    for body in [
        "const selected=basename;return (path:string):string=>{return selected(path);};",
        "return true?basename:basename;",
    ] {
        let source = format!(
            "import {{ basename }} from 'std:path';function factory():(path:ReadOnly<string>)=>string{{{body}}}function main(){{const action=factory();console.log(action('/tmp/first.txt'));console.log(action('/tmp/second.txt'));}}"
        );
        let memory = Memory::new(&[("main.tsr", &source)]);
        let checked = tsrust_compiler::project::check_graph(
            load_project(&memory, "main.tsr").unwrap(),
        ).unwrap();
        let (rust, features) = checked.emit_rust_with_native_features();
        assert_eq!(features, vec!["path"]);
        for optimization in ["0", "3"] {
            assert_eq!(native_stdout(&rust, optimization), "first.txt\nsecond.txt\n");
        }
    }
}

#[test]
fn native_function_storage_preserves_reassignment_and_move_rejections() {
    let source = "import {basename as first,basename as second} from 'std:path';function main(){let action=first;console.log(action('/tmp/first.txt'));action=second;console.log(action('/tmp/second.txt','.txt'));const moved=action;console.log(moved('/tmp/third.txt'));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked = tsrust_compiler::project::check_graph(
        load_project(&memory, "main.tsr").unwrap(),
    ).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "first.txt\nsecond\nthird.txt\n");
    }
    for (body, code) in [
        ("function main(){const f=basename;const next=f;const wrapper=(path:string):string=>{return f(path);};}", "closure-consumed"),
        ("function main(){let f=basename;const next=f;console.log(f('/tmp/a'));}", "closure-consumed"),
    ] {
        let source = format!("import {{basename}} from 'std:path';{body}");
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error = tsrust_compiler::project::check_graph(
            load_project(&memory, "main.tsr").unwrap(),
        ).err().unwrap();
        assert_eq!(error.code, code);
        assert_eq!(error.location.source.0, "main.tsr");
        assert!(!error.message.contains("@native:"));
    }
}

#[test]
fn borrowed_callable_record_fields_report_ownership_without_syntax_fixes() {
    // A borrowing field contract is a valid declaration.
    for contract in ["Mutable<Function<string, [path:ReadOnly<string>]>>", "ReadOnly<Function<int>>"] {
        let source = format!("import {{basename}} from 'std:path';type Handler={{run:{contract};}};function main(){{}}");
        let memory = Memory::new(&[("main.tsr", &source)]);
        tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
            .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    }
    // An owning field cannot hold a callback that borrows, and a record with a
    // borrowing callback cannot leave the owner of the borrowed variable.
    for body in [
        "type Handler={run:Function<int>;};function main(){const text=\"Pat\";const read=():int=>{return text.length;};const handler:Handler={run:read};}",
        "type Handler={run:ReadOnly<Function<int>>;};function make():Handler{const a=[3];return {run:():int=>a[0]};}function main(){}",
    ] {
        let source = format!("import {{basename}} from 'std:path';{body}");
        let memory = Memory::new(&[("main.tsr", &source)]);
        let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).err().unwrap();
        assert_eq!(error.code, "callback-escape");
        assert!(error.fixes.is_empty());
        assert_eq!(error.location.source.0, "main.tsr");
    }
}

#[test]
fn imported_callable_record_fields_preserve_private_nominal_types() {
    let memory = Memory::new(&[
        ("main.tsr", "import {Handler as Task} from './model.tsr';function main(){const task:Task={run:(value:Hidden):int=>{return 1;}};}"),
        ("model.tsr", "type Hidden={name:string;};export type Handler={run:Function<int, [value:ReadOnly<Hidden>]>;};"),
    ]);
    // The private type must remain inaccessible at the caller's source spelling.
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).err().unwrap();
    assert_eq!(error.code, "type");
    let memory = Memory::new(&[
        ("main.tsr", "import {Handler as Task, make} from './model.tsr';function main(){const task:Task=make();}"),
        ("model.tsr", "type Hidden={name:string;};export type Handler={run:Function<int, [value:ReadOnly<Hidden>]>;};export function make():Handler{return {run:(value:Hidden):int=>{return value.name.length;}};}"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "");
}

#[test]
fn structured_record_annotations_remap_nested_private_types_and_original_spans() {
    let memory = Memory::new(&[
        ("main.tsr", "import { Team as Group } from './model.tsr';type User={wrong:int;};function main(){const team:Group={members:[{name:'Pat'}]};console.log(team.members[0].name);}"),
        ("model.tsr", "type User={name:string;};export type Team={members:((User)[]);};"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "Pat\n");
    let dependency = "// 🦀\nexport type Team={value:Box<int>;};";
    let memory = Memory::new(&[
        ("main.tsr", "import {Team} from './model.tsr';function main(){}"),
        ("model.tsr", dependency),
    ]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    let error = tsrust_compiler::project::check_graph(graph).err().unwrap();
    assert_eq!(error.code, "type");
    assert!(error.message.contains("Unknown generic record type"));
    assert_eq!(error.location.source.0, "model.tsr");
    assert_eq!(&dependency[error.location.span.start..error.location.span.end], "Box");
    assert!(!error.message.contains("__tsrust"));
}

#[test]
fn structured_local_annotations_remap_import_aliases_and_dependency_errors() {
    let memory = Memory::new(&[
        ("main.tsr", "import {User as Person} from './model.tsr';type User={wrong:int;};function main(){const values:((Person)[])=[{name:'Pat'}];console.log(values[0].name);}"),
        ("model.tsr", "export type User={name:string;};"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "Pat\n");
    let dependency = "// 🦀\nexport function value():int{const item:Box<int>=7;return item;}";
    let memory = Memory::new(&[
        ("main.tsr", "import {value} from './model.tsr';function main(){console.log(value());}"),
        ("model.tsr", dependency),
    ]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    let error = tsrust_compiler::project::check_graph(graph).err().unwrap();
    assert_eq!(error.code, "type");
    assert!(error.message.contains("Unknown generic record type"));
    assert_eq!(error.location.source.0, "model.tsr");
    assert_eq!(&dependency[error.location.span.start..error.location.span.end], "Box");
}

#[test]
fn structured_signature_applications_report_original_nested_type_spans() {
    for declaration in [
        "export function sample(item:Box<int>):int{return 0;}",
        "export function sample():Box<int>{return 0;}",
        "export function sample(action:(item:ReadOnly<Box<int>>)=>int):int{return 0;}",
        "export function sample(action:(item:ReadOnly<string>)=>Box<int>):int{return 0;}",
    ] {
        let dependency = format!("// 🦀\n{declaration}");
        let memory = Memory::new(&[
            ("main.tsr", "import {sample} from './model.tsr';function main(){}"),
            ("model.tsr", &dependency),
        ]);
        let graph = load_project(&memory, "main.tsr").unwrap();
        let error = tsrust_compiler::project::check_graph(graph).err().unwrap();
        assert_eq!(error.code, "type");
        assert!(error.message.contains("Unknown generic record type"));
        assert_eq!(error.location.source.0, "model.tsr");
        assert_eq!(&dependency[error.location.span.start..error.location.span.end], "Box");
    }
}

#[test]
fn structured_annotations_preserve_existing_expression_depth_limits() {
    let source = format!("function main(){{const a=[0];console.log({}0{});const x:int|null=1;console.log(String(x));}}", "a[".repeat(100), "]".repeat(100));
    let memory = Memory::new(&[("main.tsr", &source)]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    let checked = tsrust_compiler::project::check_graph(graph).unwrap();
    checked.emit_rust();
    let source = format!("function main(){{const a=[0];console.log({}0{});}}", "a[".repeat(200), "]".repeat(200));
    let memory = Memory::new(&[("main.tsr", &source)]);
    assert_eq!(load_project(&memory, "main.tsr").err().unwrap().code, "unsupported");
}

#[test]
fn generic_function_inference_preserves_import_aliases() {
    let dependency="// 🦀\nexport function Box<T>(value:T):T{return value;}";
    let entry="import {Box} from './model.tsr';function main(){Box(1);}";
    let memory=Memory::new(&[("main.tsr",entry),("model.tsr",dependency)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    checked.emit_rust();
}

#[test]
fn generic_records_specialize_concrete_types_nested_arrays_and_nullable_payloads() {
    let source = "type Box<T>={value:T;};function read(box:Box<int>):int{return box.value;}function main(){const number:Box<int>={value:7};const same:Box<(int)>={value:8};const text:Box<string>={value:'Pat'};const copied=clone(text);const nested:Box<Box<int>>={value:{value:9}};const boxes:Box<string>[]=[{value:'Sam'}];const nullable:Box<int|null>={value:null};console.log(read(number),read(same));console.log(text.value,copied.value,nested.value.value,boxes[0].value,String(nullable.value));}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "7 8\nPat Pat 9 Sam null\n");
    }
}

#[test]
fn generic_record_errors_and_expansion_are_bounded() {
    for (source, code) in [
        ("type Box<T>={value:T;};function main(){const value:Box<int>={value:'wrong'};}", "type"),
        ("type Box<T>={value:T;};function main(){const value:Box<int,string>={value:1};}", "arity"),
        ("type Box<T>={value:T;};function main(){const value:Box={value:1};}", "arity"),
        ("type Box<T>={value:T;};type Phantom<T>={};function main(){const value:Phantom<Missing>={};}", "type"),
        ("type Bad<T>={value:Missing;};function main(){}", "type"),
        ("type Bad<T>={value:T;value:T;};function main(){}", "duplicate"),
        ("type Bad<T>={value:T<int>;};function main(){}", "type"),
        ("type Growing<T>={next:Growing<T[]>;};function use(value:Growing<int>){}function main(){}", "generic-limit"),
        ("type Cycle<T>={next:Cycle<T>;};function use(value:Cycle<int>){}function main(){}", "recursive-layout"),
        ("type Box<T>={value:T;};function main(){const value:Box<string>={value:'Pat'};const next=value;console.log(value.value);}", "transferred"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code, code, "{source}: {error:?}");
    }
}

#[test]
fn generic_record_aliases_recursive_arrays_and_canonical_union_arguments_execute() {
    let memory = Memory::new(&[
        ("main.tsr", "import {Box as Package,Node} from './model.tsr';type Box={wrong:int;};function observe(box:Package<int|null>):string{return String(box.value);}function main(){const value:Package<null|int>={value:5};const root:Node<int>={value:7,children:[]};console.log(observe(value),root.value,root.children.length);}"),
        ("model.tsr", "export type Box<T>={value:T;};export type Node<T>={value:T;children:Node<T>[];};"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "5 7 0\n");
    }
}

#[test]
fn generic_template_diagnostics_have_stable_declaration_order() {
    let source = "type Zebra<T>={value:MissingZ;};type Alpha<T>={value:MissingA;};function main(){}";
    let expected = source.find("MissingA").unwrap();
    for _ in 0..32 {
        let error = tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code, "type");
        assert_eq!(error.span.start, expected);
    }
}

#[test]
fn generic_record_specialization_count_boundaries_are_exact() {
    fn source(templates: usize, instances: usize, extra: bool) -> String {
        let mut output = String::new();
        for i in 0..=instances {
            output.push_str(&format!("type Argument{i}={{value:int;}};"));
        }
        for template in 0..templates {
            output.push_str(&format!("type Box{template}<T>={{value:T;}};"));
            for i in 0..instances {
                output.push_str(&format!("type Holder{template}_{i}={{value:Box{template}<Argument{i}>;}};"));
            }
        }
        if extra {
            if templates == 1 {
                output.push_str(&format!("type Extra={{value:Box0<Argument{instances}>;}};"));
            } else {
                output.push_str("type ExtraBox<T>={value:T;};type Extra={value:ExtraBox<int>;};");
            }
        }
        output.push_str("function main(){}");
        output
    }
    for templates in [1, 8] {
        let accepted = source(templates,128,false);
        assert!(tsrust_compiler::check(&accepted).is_ok(),"{templates} templates at budget");
        let rejected = source(templates,128,true);
        let error = tsrust_compiler::check(&rejected).err().unwrap();
        assert_eq!(error.code,"generic-limit","{error:?}");
        assert!(error.span.start < error.span.end);
        assert!(error.span.end <= rejected.len());
        assert!(rejected[error.span.start..error.span.end].contains('<'));
    }
}

#[test]
fn explicit_generic_call_arguments_parse_without_silent_erasure() {
    for argument in ["int", "Box<Box<int>>", "int|null", "(value:int)=>int"] {
        let source = format!("type Box<T>={{value:T;}};function identity(value:int):int{{return value;}}function main(){{console.log(identity<{argument}>(7));}}");
        let memory = Memory::new(&[("main.tsr",&source)]);
        let graph = load_project(&memory,"main.tsr").unwrap();
        let error = tsrust_compiler::project::check_graph(graph).err().unwrap();
        assert_eq!(error.code,"type","{error:?}");
        assert!(error.message.contains("declared generic function"));
        let span=error.location.span;
        assert_eq!(&source[span.start..span.end],argument);
    }
    let source="function main(){console.log(1 < 2,8 >> 1,16 >>> 2);}";
    assert!(tsrust_compiler::check(source).is_ok());
}

#[test]
fn generic_record_provenance_retains_structured_concrete_arguments() {
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let source="type Box<T>={value:T;};function main(){const nested:Box<Box<int>>={value:{value:7}};const same:Box<(int)>={value:8};const maybe:Box<int|null>={value:null};}";
    let checked=tsrust_compiler::check(source).unwrap();
    let records=checked.generic_records();
    assert_eq!(records.len(),3);
    let outer=records.iter().find(|record|record.name=="Box<Box<int>>").unwrap();
    assert_eq!(outer.declaration,"Box");
    assert_eq!(&source[outer.declaration_span.start..outer.declaration_span.end],"Box");
    assert!(matches!(&outer.arguments[0].kind,Kind::Named(name) if name.text=="Box<int>"));
    let nullable=records.iter().find(|record|record.name=="Box<int|null>").unwrap();
    assert!(matches!(&nullable.arguments[0].kind,Kind::Union(members) if members.len()==2));
    assert_eq!(checked.generic_records(),records);
}

#[test]
fn explicit_generic_functions_execute_scalars_owned_records_arrays_and_recursion() {
    let source="type Box<T>={value:T;};function identity<T>(value:T):T{return value;}function unwrap<T>(box:Box<T>):T{return box.value;}function repeat<T>(value:T,remaining:int):T{if(remaining===0){return value;}return repeat<T>(value,remaining-1);}function main(){const a=identity<int>(7);const text=identity<string>('Pat');const box:Box<string>={value:'Sam'};const array=identity<int[]>([2,3]);console.log(a,text,unwrap<string>(box),array.length,repeat<int>(9,3));}";
    let checked=tsrust_compiler::check(source).unwrap();
    for optimization in ["0","3"] { assert_eq!(native_stdout(&checked.emit_rust(),optimization),"7 Pat Sam 2 9\n"); }
}

#[test]
fn explicit_generic_function_errors_and_local_shadowing_are_source_diagnostics() {
    for (source,code) in [
        ("function identity<T>(value:T):T{return value;}function main(){identity<int,string>(1);}","arity"),
        ("function identity<T>(value:T):T{return value;}function main(){identity<int>('wrong');}","type"),
        ("function ignore<T>(){}function main(){ignore<Missing>();}","type"),
        ("function ignore<T>(){}function main(){ignore();}","generic-inference"),
        ("function identity<T>(value:T):T{return value;}function main(){const identity=(value:int):int=>{return value;};identity<int>(1);}","type"),
        ("function grow<T>(value:T){grow<T[]>([value]);}function main(){grow<int>(1);}","generic-limit"),
        ("function identity<T>(value:T):T{return value;}function main(){const text='Pat';const copy=identity<string>(text);console.log(text);}","transferred"),
        ("function identity<T>(value:T):T{return value;}function main(){const identity=(value:int):int=>{return value;};console.log(identity<int>(1));}","type"),
    ] {
        let error=tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code,code,"{source}: {error:?}");
        assert!(error.span.end<=source.len());
    }
}

#[test]
fn explicit_generic_functions_substitute_callbacks_and_preserve_import_aliases() {
    let memory=Memory::new(&[
        ("main.tsr","import {apply as invoke,identity as keep} from './model.tsr';function main(){const value=invoke<int>((item:int):int=>{return item+1;},7);console.log(value,keep<string>('Pat'));}"),
        ("model.tsr","export function identity<T>(value:T):T{return value;}export function apply<T>(action:(value:T)=>T,value:T):T{return action(value);}"),
    ]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"8 Pat\n");}
}

#[test]
fn unused_generic_function_annotations_are_lexically_validated() {
    for (source,code) in [
        ("function unused<T>(value:Missing):T{return value;}function main(){}","type"),
        ("function unused<T>(value:T):U{return value;}function main(){}","type"),
        ("function unused<T>(value:T):T{const x:Missing=value;return value;}function main(){}","type"),
        ("type Box<T>={value:T;};function unused<T>(value:Box):T{return value.value;}function main(){}","arity"),
        ("type Box<T>={value:T;};function unused<T>(value:Box<T,int>):T{return value.value;}function main(){}","arity"),
        ("function unused<T>(value:T<int>):T{return value;}function main(){}","type"),
        ("function unused<T>(value:T,value:T):T{return value;}function main(){}","duplicate"),
        ("function unused<T>(action:(value:T,value:T)=>T):T{return action(1,2);}function main(){}","duplicate"),
        ("function unused<T>(){unused<Missing>();}function main(){}","type"),
    ] {
        let error=tsrust_compiler::check(source).err().unwrap_or_else(||panic!("Expected error: {source}"));
        assert_eq!(error.code,code,"{source}: {error:?}");
        assert!(error.span.end<=source.len());
    }
    assert!(tsrust_compiler::check("type Box<T>={value:T;};function unused<T>(value:Box<T>):T{return value.value;}function main(){}").is_ok());
}

#[test]
fn generic_call_argument_linking_keeps_enclosing_type_parameters_lexical() {
    let memory=Memory::new(&[
        ("main.tsr","import {again} from './model.tsr';function main(){console.log(again<int>(7));}"),
        ("model.tsr","import {Thing as T} from './types.tsr';function identity<T>(value:T):T{return value;}export function again<T>(value:T):T{return identity<T>(value);}"),
        ("types.tsr","export type Thing={value:string;};"),
    ]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"7\n");}
}

#[test]
fn explicit_generic_nullable_record_signatures_use_checked_argument_identity() {
    let source="type Box<T>={value:T;};function retain<T>(value:Box<T>|null):Box<T>|null{return value;}function main(){const value:Box<int>|null={value:7};const saved=retain<int>(value);if(saved!==null){console.log(saved.value);}}";
    let checked=tsrust_compiler::check(source).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"7\n");}
}

#[test]
fn generic_function_specialization_count_limits_are_exact() {
    fn source(templates:usize,extra:bool)->String {
        let mut source=String::new();
        for i in 0..129 {source.push_str(&format!("type Argument{i}={{value:int;}};"));}
        for template in 0..templates {source.push_str(&format!("function identity{template}<T>(value:T):T{{return value;}}"));}
        if extra && templates>1 {source.push_str("function extra<T>(value:T):T{return value;}");}
        source.push_str("function main(){");
        for template in 0..templates {
            for i in 0..128 {source.push_str(&format!("identity{template}<Argument{i}>({{value:7}});"));}
        }
        // Repeated and grouped applications must reuse the cache even at both limits.
        for template in 0..templates {
            source.push_str(&format!("identity{template}<(Argument0)>({{value:8}});identity{template}<Argument127>({{value:9}});"));
        }
        if extra {
            source.push_str(if templates==1 {"identity0<Argument128>({value:7});"} else {"extra<int>(7);"});
        }
        source.push('}');
        source
    }
    for templates in [1,8] {
        let accepted=source(templates,false);
        assert!(tsrust_compiler::check(&accepted).is_ok(),"{templates} templates at limit");
        let rejected=source(templates,true);
        let error=tsrust_compiler::check(&rejected).err().unwrap();
        assert_eq!(error.code,"generic-limit","{error:?}");
        assert!(error.span.start<error.span.end && error.span.end<=rejected.len());
    }
}

#[test]
fn inferred_generic_functions_execute_checked_hints_context_and_recursion() {
    let source="type Box<T>={value:T;};function identity<T>(value:T):T{return value;}function wrap<T>(value:T):Box<T>{return {value:value};}function unwrap<T>(box:Box<T>):T{return box.value;}function repeat<T>(value:T,remaining:int):T{if(remaining===0){return value;}return repeat(value,remaining-1);}function main(){const a=identity(7);const text=identity('Pat');const box=wrap('Sam');const array=identity([2,3]);const empty:int[]=identity([]);console.log(a,text,unwrap(box),array.length,empty.length,repeat(9,3),identity(identity(11)));}";
    let checked=tsrust_compiler::check(source).unwrap();
    for optimization in ["0","3"] { assert_eq!(native_stdout(&checked.emit_rust(),optimization),"7 Pat Sam 2 0 9 11\n"); }
}

#[test]
fn inferred_generic_count_limits_reuse_explicit_instances_at_both_boundaries() {
    fn source(templates: usize, extra: bool) -> String {
        let mut source = String::new();
        for i in 0..129 {
            source.push_str(&format!("type Argument{i}={{value:int;}};"));
        }
        for template in 0..templates {
            source.push_str(&format!("function identity{template}<T>(value:T):T{{return value;}}"));
        }
        if extra && templates > 1 {
            source.push_str("function extra<T>(value:T):T{return value;}");
        }
        source.push_str("function main(){");
        for template in 0..templates {
            // An explicit prepass instance and later inferred calls share one budget.
            source.push_str(&format!("identity{template}<Argument0>({{value:0}});"));
            for i in 0..128 {
                source.push_str(&format!("const value{template}_{i}:Argument{i}={{value:7}};identity{template}(value{template}_{i});"));
            }
            source.push_str(&format!("identity{template}<(Argument0)>({{value:8}});identity{template}<Argument127>({{value:9}});"));
        }
        if extra {
            source.push_str(if templates == 1 {
                "const overflow:Argument128={value:7};identity0(overflow);"
            } else {
                "extra(7);"
            });
        }
        source.push('}');
        source
    }
    for templates in [1, 8] {
        let accepted = source(templates, false);
        assert!(tsrust_compiler::check(&accepted).is_ok(), "{templates} templates at limit");
        let rejected = source(templates, true);
        let error = tsrust_compiler::check(&rejected).err().unwrap();
        assert_eq!(error.code, "generic-limit", "{error:?}");
        assert!(error.span.start < error.span.end && error.span.end <= rejected.len());
    }
}

#[test]
fn inferred_generic_expanding_recursion_reports_a_bounded_source_error() {
    let source = "function expand<T>(value:T){expand([value]);}function main(){expand(1);}";
    let error = tsrust_compiler::check(source).err().unwrap();
    assert_eq!(error.code, "generic-limit", "{error:?}");
    assert!(error.span.start < error.span.end && error.span.end <= source.len());
    assert_eq!(&source[error.span.start..error.span.end], "expand([value])");
}

#[test]
fn generic_missing_return_diagnostics_use_original_function_declarations() {
    for call in ["missing(1)", "missing<int>(1)"] {
        let source = format!("function missing<T>(value:T):T{{}}function main(){{{call};}}");
        let error = tsrust_compiler::check(&source).err().unwrap();
        assert_eq!(error.code, "type");
        assert_eq!(error.message, "Function \"missing\" must return int.");
        assert_eq!(&source[error.span.start..error.span.end], "missing");
    }
}

#[test]
fn inferred_generic_ownership_preserves_captures_and_indexed_clone_independence() {
    let source = "type User={name:string;};function identity<T>(value:T):T{return value;}function make(user:User):()=>User{return ():User=>{return identity(user);};}function main(){const users:User[]=[{name:'Pat'}];let saved=identity(clone(users[0]));saved.name='Sam';const get=make({name:'Lee'});const captured=get();console.log(users[0].name,saved.name,captured.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat Sam Lee\n");
    }
    let repeated = "type User={name:string;};function identity<T>(value:T):T{return value;}function make(user:User):()=>User{return ():User=>{return identity(user);};}function main(){const get=make({name:'Pat'});get();get();}";
    let error = tsrust_compiler::check(repeated).err().unwrap();
    assert_eq!(error.code, "closure-consumed", "{error:?}");
    let indexed = "type User={name:string;};function identity<T>(value:T):T{return value;}function main(){const users:User[]=[{name:'Pat'}];const saved=identity(users[0]);console.log(users[0].name,saved.name);}";
    let error = tsrust_compiler::check(indexed).err().unwrap();
    assert!(error.span.start < error.span.end && error.span.end <= indexed.len());
    assert!(error.message.to_lowercase().contains("clone"), "{error:?}");
}

#[test]
fn inferred_generic_owned_callbacks_keep_consuming_arguments_and_transfer_contracts() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function main(){const action=(user:User):User=>{console.log('called');return user;};const saved=invoke(action,{name:'Pat'});console.log(saved.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "called\nPat\n");
    }
    let repeated = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function main(){const action=(user:User):User=>{return user;};const user:User={name:'Pat'};invoke(action,user);action({name:'Sam'});}";
    let error = tsrust_compiler::check(repeated).err().unwrap();
    assert_eq!(error.code, "closure-consumed", "{error:?}");
}

#[test]
fn generic_checked_callback_constraints_reject_conflicts_without_weakening_access() {
    let declaration = "type User={name:string;};type Other={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}";
    for (body, code) in [
        ("const action=(user:User):User=>{return user;};const other:Other={name:'Pat'};invoke(action,other);", "type"),
        ("const action=(user:User):string=>{return user.name;};invoke(action,{name:'Pat'});", "type"),
        ("const action=(left:User,right:User):User=>{return left;};invoke(action,{name:'Pat'});", "type"),
        ("let count:int=0;const action=(user:User):User=>{count=count+1;return user;};invoke(action,{name:'Pat'});", "callback-escape"),
        ("const action=(user:User):User=>{return clone(user);};invoke(action,{name:'Pat'});", "callback-access"),
        ("const action=(user:User):User=>{return user;};const users:User[]=[{name:'Pat'}];invoke(action,users[0]);", "borrowed-element"),
    ] {
        let source = format!("{declaration}function main(){{{body}}}");
        let error = tsrust_compiler::check(&source).err().unwrap();
        assert_eq!(error.code, code, "{body}: {error:?}");
        assert!(error.span.start < error.span.end && error.span.end <= source.len());
    }
}

#[test]
fn inferred_generic_callbacks_accept_returned_owned_mutable_environments() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function make():(value:Owned<User>)=>User{let calls:int=0;return (user:User):User=>{calls=calls+1;console.log(calls);return user;};}function main(){const action=make();const saved=invoke(action,{name:'Pat'});console.log(saved.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "1\nPat\n");
    }
}

#[test]
fn inline_callback_annotations_supply_generic_nominal_context_without_eager_effects() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function main(){const saved=invoke((user:User):User=>{console.log('called');return user;},{name:'Pat'});console.log(saved.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "called\nPat\n");
    }
    let contradictory = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function main(){invoke((user:User):string=>{return user.name;},{name:'Pat'});}";
    let error = tsrust_compiler::check(contradictory).err().unwrap();
    assert_eq!(error.code, "type", "{error:?}");
    let invalid_body = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function main(){invoke((user:User):User=>{return 7;},{name:'Pat'});}";
    let error = tsrust_compiler::check(invalid_body).err().unwrap();
    assert_eq!(error.code, "type", "{error:?}");
}

#[test]
fn nested_generic_callback_results_refresh_outer_inference_hints() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [value:Owned<T>]>,value:T):T{return action(value);}function make<T>(seed:T):(value:Owned<T>)=>T{return (value:T):T=>{console.log('called');return value;};}function main(){const seed:User={name:'seed'};const saved=invoke(make(seed),{name:'Pat'});console.log(saved.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "called\nPat\n");
    }
}

#[test]
fn generic_callback_facts_preserve_declared_parameter_names_and_access() {
    let source = "type User={name:string;};function invoke<T>(action:(person:ReadOnly<T>)=>int,value:T):int{return action(value);}function main(){const action=(user:User):int=>{return user.name.length;};const saved=invoke(action,{name:'Pat'});console.log(saved);}";
    let checked = tsrust_compiler::check(source).unwrap();
    let facts = checked.checked_type_facts().generic_calls;
    let call = facts.iter().find(|call| &source[call.callee_span.start..call.callee_span.end] == "invoke").unwrap();
    assert_eq!(call.parameters[0].ty.as_deref(), Some("Function<int, [person: ReadOnly<User>]>"));
    assert_eq!(call.result, "int");
}

#[test]
fn owned_generic_callback_facts_preserve_declared_contract_after_specialization() {
    let source = "type User={name:string;};function invoke<T>(action:Function<T, [person:Owned<T>]>,value:T):T{return action(value);}function main(){const action=(user:User):User=>{return user;};const saved=invoke(action,{name:'Pat'});console.log(saved.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    let facts = checked.checked_type_facts().generic_calls;
    let call = facts.iter().find(|call| &source[call.callee_span.start..call.callee_span.end] == "invoke").unwrap();
    assert_eq!(call.parameters[0].ty.as_deref(), Some("Function<User, [person: Owned<User>]>"));
    assert_eq!(call.parameters[0].access, tsrust_compiler::Mode::Consume);
    assert_eq!(call.result, "User");
}

#[test]
fn generic_callback_instances_keep_distinct_nominal_contracts() {
    let source = "type User={name:string;};type Other={name:string;};function invoke<T>(action:Function<T, [person:Owned<T>]>,value:T):T{return action(value);}function main(){const first=(user:User):User=>{return user;};const second=(other:Other):Other=>{return other;};const a=invoke(first,{name:'Pat'});const b=invoke(second,{name:'Sam'});console.log(a.name,b.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat Sam\n");
    }
    let facts = checked.checked_type_facts().generic_calls;
    let first = facts.iter().find(|call| source[call.span.start..call.span.end].starts_with("invoke(first")).unwrap();
    let second = facts.iter().find(|call| source[call.span.start..call.span.end].starts_with("invoke(second")).unwrap();
    assert_eq!(first.parameters[0].ty.as_deref(), Some("Function<User, [person: Owned<User>]>"));
    assert_eq!(second.parameters[0].ty.as_deref(), Some("Function<Other, [person: Owned<Other>]>"));
}

#[test]
fn generic_callable_results_keep_distinct_nominal_contracts() {
    let source = "type User={name:string;};type Other={name:string;};function make<T>(seed:T):(value:Owned<T>)=>T{return (value:T):T=>{return value;};}function main(){const user:User={name:'seed'};const other:Other={name:'seed'};const first=make(user);const second=make(other);const a=first({name:'Pat'});const b=second({name:'Sam'});console.log(a.name,b.name);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat Sam\n");
    }
}

#[test]
fn project_generic_callable_parameters_keep_structured_nominal_provenance() {
    use tsrust_compiler::project::ProjectType;
    let entry = "import {User as Person,invoke as apply} from './dep.tsr';function main(){const action=(user:Person):Person=>{return user;};apply(action,{name:'Pat'});}";
    let dependency = "export type User={name:string;};export function invoke<T>(action:Function<T, [person:Owned<T>]>,value:T):T{return action(value);}";
    let memory = Memory::new(&[("main.tsr", entry), ("dep.tsr", dependency)]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    let facts = checked.generic_calls();
    let call = facts.iter().find(|call| call.callee.source.0 == "main.tsr").unwrap();
    let Some(ProjectType::Callable { ownership, parameters, result }) = &call.parameters[0].ty else { panic!("checked callable contract") };
    assert_eq!(*ownership, tsrust_compiler::type_syntax::CallableOwnership::Owned);
    assert_eq!(parameters[0].name, "person");
    assert_eq!(parameters[0].access, tsrust_compiler::Mode::Consume);
    for ty in [parameters[0].ty.as_ref().unwrap(), result.as_ref()] {
        let ProjectType::Named { name, declaration: Some(declaration), .. } = ty else { panic!("nominal child") };
        assert_eq!(name, "User");
        assert_eq!(declaration.source.0, "dep.tsr");
        assert_eq!(&dependency[declaration.span.start..declaration.span.end], "User");
    }
}

#[test]
fn project_generic_callable_results_keep_nominal_identity_and_invocation() {
    use tsrust_compiler::project::ProjectType;
    let entry = "import {User as Person,make} from './dep.tsr';function main(){const seed:Person={name:'seed'};const action=make(seed);console.log(action({name:'Pat'}).name);}";
    let dependency = "export type User={name:string;};export function make<T>(seed:T):(value:Owned<T>)=>T{return (value:T):T=>{return value;};}";
    let memory = Memory::new(&[("main.tsr", entry), ("dep.tsr", dependency)]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    let facts = checked.generic_calls();
    let call = facts.iter().find(|call| call.callee.source.0 == "main.tsr").unwrap();
    let ProjectType::Callable { ownership, parameters, result } = &call.result else { panic!("callable result") };
    assert_eq!(*ownership, tsrust_compiler::type_syntax::CallableOwnership::Default);
    assert_eq!(parameters[0].name, "value");
    assert_eq!(parameters[0].access, tsrust_compiler::Mode::Consume);
    let ProjectType::Named { name, declaration: Some(declaration), .. } = result.as_ref() else { panic!("nominal result") };
    assert_eq!(name, "User");
    assert_eq!(declaration.source.0, "dep.tsr");
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat\n");
    }
}

#[test]
fn project_generic_once_results_preserve_consuming_environment_contracts() {
    use tsrust_compiler::project::ProjectType;
    let entry = "import {User as Person,make} from './dep.tsr';function main(){const seed:Person={name:'Pat'};const action=make(seed);console.log(action().name);}";
    let dependency = "export type User={name:string;};export function make<T>(seed:T):()=>T{return ():T=>{return seed;};}";
    let memory = Memory::new(&[("main.tsr", entry), ("dep.tsr", dependency)]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    let facts = checked.generic_calls();
    let call = facts.iter().find(|call| call.callee.source.0 == "main.tsr").unwrap();
    let ProjectType::Callable { ownership, parameters, .. } = &call.result else { panic!("callable result") };
    assert_eq!(*ownership, tsrust_compiler::type_syntax::CallableOwnership::Once);
    assert!(parameters.is_empty());
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat\n");
    }
    let repeated = entry.replace("console.log(action().name);", "console.log(action().name);action();");
    let memory = Memory::new(&[("main.tsr", &repeated), ("dep.tsr", dependency)]);
    let error = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).err().unwrap();
    assert_eq!(error.code, "closure-consumed");
}

#[test]
fn generic_annotation_closers_restore_adjacent_arrows_without_changing_operators() {
    let source = "type Box<T>={value:T;};function make<T>(seed:T):()=>Box<Box<T>>{return ():Box<Box<T>>=>{return {value:{value:seed}};};}function main(){const action=make(9);const saved=action();console.log(saved.value.value,8>>1,9>=8);}";
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "9 4 true\n");
    }
}

#[test]
fn registered_generic_function_pipeline_executes_at_both_native_optimizations() {
    let source = include_str!("../../../benchmarks/runtime/workloads/generic_function_record_pipeline.tsr");
    let checked = tsrust_compiler::check(source).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "22\n");
    }
}

#[test]
fn inferred_generics_preserve_order_ownership_callback_contracts_and_result_context() {
    let source="function identity<T>(value:T):T{return value;}function choose<T>(left:T|null,right:T):T{return right;}function apply<T>(action:(value:T)=>T,value:T):T{return action(value);}function main(){let i:int=0;const a=identity(i++);const b=identity(i++);const original=clone('Pat');const copied=identity(clone(original));const nested:int[]=identity(identity([]));const n:number=identity(7);const selected=choose(null,9);const mapped=apply((value:int):int=>{return value+1;},10);console.log(a,b,i,original,copied,nested.length,n,selected,mapped);}";
    let checked=tsrust_compiler::check(source).unwrap();
    for optimization in ["0","3"] { assert_eq!(native_stdout(&checked.emit_rust(),optimization),"0 1 2 Pat Pat 0 7 9 11\n"); }
    for (source, code) in [
        ("function identity<T>(value:T):T{return value;}function main(){const text='Pat';const moved=identity(text);console.log(text);}","transferred"),
        ("function identity<T>(value:T):T{return value;}function main(){identity([]);}","generic-inference"),
        ("function pair<T>(left:T,right:T){}function main(){pair(1,'Pat');}","type"),
        ("function identity<T>(value:T):T{return value;}function main(){const identity=(value:int):int=>{return value+1;};console.log(identity(7));}","pass"),
    ] {
        let checked=tsrust_compiler::check(source);
        if code=="pass" { assert!(checked.is_ok()); } else { assert_eq!(checked.err().unwrap().code,code); }
    }
}

#[test]
fn checked_generic_call_facts_preserve_concrete_contracts_and_shared_span_conflicts() {
    let source="function identity<T>(value:T):T{return value;}function forward<T>(value:T):T{return identity(value);}function main(){const n=identity(7);const s=identity<string>('Pat');forward(9);forward('Sam');}";
    let checked=tsrust_compiler::check(source).unwrap();
    let facts=checked.checked_type_facts().generic_calls;
    let direct=facts.iter().find(|fact| &source[fact.span.start..fact.span.end]=="identity(7)").unwrap();
    assert_eq!(direct.parameters[0].ty.as_deref(),Some("int"));
    assert_eq!(direct.result,"int");
    assert_eq!(&source[direct.callee_span.start..direct.callee_span.end],"identity");
    assert_eq!(&source[direct.declaration_span.start..direct.declaration_span.end],"identity");
    let shared=facts.iter().filter(|fact| &source[fact.span.start..fact.span.end]=="identity(value)").collect::<Vec<_>>();
    assert_eq!(shared.len(),2);
    assert!(shared.iter().any(|fact|fact.result=="int"));
    assert!(shared.iter().any(|fact|fact.result=="string"));
    assert_eq!(checked.checked_type_facts().generic_calls,facts);
}

#[test]
fn project_generic_facts_keep_each_source_identity_and_structured_nominal_types() {
    use tsrust_compiler::project::ProjectType;
    let entry="/* 🦀 */ import {User as Person,wrap as make} from './dep.tsr';function main(){const user:Person={name:'Pat'};const value=make(user);console.log(value.value.name);}";
    let dependency="export type User={name:string;};export type Box<T>={value:T;};export function wrap<T>(value:T):Box<T>{return {value:value};}";
    let memory=Memory::new(&[("main.tsr",entry),("dep.tsr",dependency)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    let facts=checked.generic_calls();
    let call=facts.iter().find(|fact|fact.callee.source.0=="main.tsr").unwrap();
    assert_eq!(&entry[call.callee.span.start..call.callee.span.end],"make");
    assert_eq!(call.declaration.source.0,"dep.tsr");
    assert_eq!(&dependency[call.declaration.span.start..call.declaration.span.end],"wrap");
    let ProjectType::Named{name,declaration:Some(declaration),arguments}=&call.result else {panic!("nominal result")};
    assert_eq!(name,"Box");
    assert_eq!(declaration.source.0,"dep.tsr");
    let ProjectType::Named{name,declaration:Some(argument),arguments:nested}=&arguments[0] else {panic!("nominal argument")};
    assert_eq!(name,"User");
    assert_eq!(argument.source.0,"dep.tsr");
    assert!(nested.is_empty());
    assert_eq!(checked.generic_calls(),facts);
}

#[test]
fn inferred_generics_use_numeric_array_context_and_later_nested_argument_hints() {
    let source="type User={name:string;};function identity<T>(value:T):T{return value;}function first<T>(left:T,right:T):T{return left;}function main(){const numeric:number[]=identity([1,2,3]);const nested:number[][]=identity([[1,2],[3]]);const fromLater=first(identity([]),identity([7]));const fromContext:User=identity({name:'Pat'});let i:int=0;const ordered=first(identity([]),identity([i++]));console.log(numeric.join('|'),nested[1][0],fromLater.length,fromContext.name,ordered.length,i);}";
    let checked=tsrust_compiler::check(source).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"1|2|3 3 0 Pat 0 1\n");}
    for (source, code) in [
        ("function identity<T>(value:T):T{return value;}function main(){const value:int=1;const numbers:number[]=identity([value]);}", "type"),
        ("function identity<T>(value:T):T{return value;}function first<T>(left:T,right:T):T{return left;}function main(){first(identity([]),identity([]));}", "generic-inference"),
    ] {
        let error=tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code,code,"{error:?}");
    }
}

#[test]
fn annotated_callable_locals_preserve_imported_nominal_contracts() {
    let memory = Memory::new(&[
        ("main.tsr", "import { User as Person, take as keep } from './model.tsr';function main(){const action:((Function<Person, [person:Owned<Person>]>))=keep;const saved=action({name:'Pat'});console.log(saved.name);}"),
        ("model.tsr", "export type User={name:string;};export function take(user:User):User{return user;}"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "Pat\n");
    }
}

#[test]
fn native_optional_functions_adapt_to_owned_record_field_contracts() {
    let source = "import {basename as nameOf} from 'std:path';type Paths={plain:Function<string, [path:ReadOnly<string>]>;without:Function<string, [path:ReadOnly<string>,suffix:ReadOnly<string>]>;once:Once<Function<string, [path:ReadOnly<string>]>>;};function main(){const paths:Paths={plain:nameOf,without:nameOf,once:nameOf};console.log(paths.plain('/tmp/report.txt'));console.log(paths.without('/tmp/report.txt','.txt'));console.log(paths.once('/tmp/once.txt'));paths.plain=(path:string):string=>{return nameOf(path,'.txt');};console.log(paths.plain('/tmp/replaced.txt'));}";
    let memory = Memory::new(&[("main.tsr", source)]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), optimization), "report.txt\nreport\nonce.txt\nreplaced\n");
    }
}

#[test]
fn imported_callable_arrays_preserve_private_nominal_inputs_and_field_families() {
    let memory = Memory::new(&[
        ("main.tsr", "import {Collection as Tasks, make, user} from './model.tsr';function main(){const tasks:Tasks=make();const value=user();console.log(tasks.actions[0](value));console.log(tasks.actions[1](value));}"),
        ("model.tsr", "type Hidden={name:string;};export type Collection={actions:((value:ReadOnly<Hidden>)=>int)[];};export function user():Hidden{return {name:'Pat'};}export function make():Collection{return {actions:[(value:Hidden):int=>{return value.name.length;},(value:Hidden):int=>{return value.name.length+1;}]};}"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    for level in ["0", "3"] {
        assert_eq!(native_stdout(&checked.emit_rust(), level), "3\n4\n");
    }
}

#[test]
fn imported_rest_parameters_preserve_nominal_aliases_generic_packs_and_actual_arity() {
    let memory = Memory::new(&[
        ("imported.tsr", r#"import { Item as Entry, pack, count, size } from "./library.tsr";
function main(){const rows=pack<Entry>({value:3},{value:7});console.log(rows[0].value,rows[1].value,rows.length,count({value:5},{value:6}),count());const action:Function<int, [value?:undefined]>=size;console.log(action(),action(undefined));}
"#),
        ("library.tsr", r#"export type Item = { value: int; };
export function pack<T>(...values:T[]):T[]{return values;}
export function count(...values:Item[]):int{return values.length;}
export function size(...values:undefined[]):int{return values.length;}
"#),
    ]);
    let checked = tsrust_compiler::project::check_graph(
        load_project(&memory, "imported.tsr").unwrap(),
    ).unwrap();
    let rust = checked.emit_rust();
    for optimization in ["0", "3"] {
        assert_eq!(native_stdout(&rust, optimization), "3 7 2 2 0\n0 1\n");
    }
}

#[test]
fn array_literal_spread_crosses_imported_generic_record_and_function_boundaries() {
    let memory = Memory::new(&[
        ("main.tsr", "import { Row, rows, append } from './data.tsr'; function main(){const first=rows();const values:Row[]=[...first,{value:9}];const joined=append<Row>(values,{value:11});for(const row of joined){console.log(row.value);}}"),
        ("data.tsr", "export type Row={value:int;};export function rows():Row[]{return [{value:7}];}export function append<T>(values:T[],item:T):T[]{return [...values,item];}"),
    ]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap())
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    for level in ["0", "3"] { assert_eq!(native_stdout(&checked.emit_rust(), level), "7\n9\n11\n"); }
}

#[test]
fn retained_capture_plans_map_diagnostics_after_prior_module_declarations() {
    let free = "export function free():int{function one():int{return 1;}return one();}";
    let captured = "export function run():int{function read():int{return data[0];}const result=read();const data=[7];return result;}function data():int{return 9;}";
    let memory = Memory::new(&[
        ("main.tsr", "import { free } from './free.tsr';import { run } from './captured.tsr';function main(){console.log(free(),run());}"),
        ("free.tsr", free),
        ("captured.tsr", captured),
    ]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    let free_position = graph.order.iter().position(|id| id.0 == "free.tsr").unwrap();
    let captured_position = graph.order.iter().position(|id| id.0 == "captured.tsr").unwrap();
    assert!(free_position < captured_position, "witness needs earlier declaration offsets");
    let error = tsrust_compiler::project::check_graph(graph).err().expect("capture read before initialization");
    assert_eq!(error.code, "unassigned");
    assert_eq!(error.location.source, SourceId("captured.tsr".into()));
    assert_eq!(&captured[error.location.span.start..error.location.span.end], "read");
    assert!(error.related.iter().any(|(location, _)| location.source == SourceId("captured.tsr".into()) && &captured[location.span.start..location.span.end] == "data"));
}

#[test]
fn typed_capture_context_keeps_private_types_and_rebased_error_locations() {
    let captured = "type User={name:string;};export function run():string{const user:User={name:'Pat'};function read():string{return user.missing;}return read();}";
    let memory = Memory::new(&[
        ("main.tsr", "import { free } from './free.tsr';import { run } from './captured.tsr';function main(){console.log(free(),run());}"),
        ("free.tsr", "export function free():int{function one():int{return 1;}return one();}"),
        ("captured.tsr", captured),
    ]);
    let graph = load_project(&memory, "main.tsr").unwrap();
    let error = tsrust_compiler::project::check_graph(graph).err().expect("invalid captured private field");
    assert_eq!(error.code, "type", "{}", error.message);
    assert!(error.message.contains("missing") && error.message.contains("User"));
    assert_eq!(error.location.source, SourceId("captured.tsr".into()));
    assert_eq!(&captured[error.location.span.start..error.location.span.end], "user.missing");
}

#[test]
fn imported_lexical_generic_instances_keep_rebased_owners_and_types() {
    let entry = support::format_round_trip("import {factory as left} from './left.tsr';import {factory as right} from './right.tsr';function main(){const number=left<int>(7);const text=right<string>('owned');console.log(number<boolean>(true),text());}");
    let left = support::format_round_trip("export function factory<T>(value:T){function take<U>(unused:U):T{function forward<V>(other:V):V{return other;}forward<U>(unused);return value;}return take;}");
    let right = support::format_round_trip("export function factory<T>(value:T){function take():T{return value;}return take;}");
    let memory = Memory::new(&[("main.tsr", &entry), ("left.tsr", &left), ("right.tsr", &right)]);
    let checked = tsrust_compiler::project::check_graph(load_project(&memory, "main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(), "0"), "7 owned\n");
}

#[test]
fn lexical_path_p1_project_contract() {
    let entry = support::format_round_trip("import {dirname as parent,extname} from 'std:path';function main(){const path='/tmp/é😊.txt/';const stored=parent;console.log(stored(path));console.log(extname(path));console.log(path);}");
    let memory=Memory::new(&[("main.tsr", &entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {
        assert_eq!(native_stdout(&checked.emit_rust(),optimization),"/tmp\n.txt\n/tmp/é😊.txt/\n");
    }
    for name in ["dirname","extname","isAbsolute","normalize"] {
        for (argument, code) in [("", "arity"),("'a','b'", "arity"),("3", "type")] {
            let source=format!("import {{{name}}} from 'std:path';function main(){{console.log({name}({argument}));}}");
            let memory=Memory::new(&[("main.tsr", &source)]);
            let error=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).err().unwrap();
            assert_eq!(error.code,code,"{error:?}");
            assert_eq!(error.location.source.0,"main.tsr");
        }
    }
}

#[test]
fn lexical_path_p2_primitive_boolean_native_contract() {
    let entry=support::format_round_trip("import {isAbsolute as absolute,normalize} from 'std:path';import {strictEqual} from 'std:testing';function main(){const path='/a//b/../';const stored=absolute;strictEqual(stored(path),true);strictEqual(absolute(''),false);strictEqual(normalize(path),'/a/');strictEqual(path,'/a//b/../');}");
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"");}
}

#[test]
fn os_identity_zero_argument_native_contract() {
    let entry = support::format_round_trip("import {platform as target,arch} from 'std:os';import {strictEqual} from 'std:testing';function main(){const stored=target;const system:string=stored();const cpu:string=arch();strictEqual(system,target());strictEqual(cpu,arch());console.log(system,cpu);}");
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    let reference=std::process::Command::new("/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node").args(["-e","console.log(require('node:os').platform(),require('node:os').arch())"]).output().unwrap();
    assert!(reference.status.success());
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),String::from_utf8(reference.stdout.clone()).unwrap());}
    for name in ["platform","arch"] {
        let source=format!("import {{{name}}} from 'std:os';function main(){{{name}(1);}}");
        let memory=Memory::new(&[("main.tsr",&source)]);
        let error=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).err().unwrap();
        assert_eq!(error.code,"arity");
        let source=format!("import {{{name}}} from 'std:os';function main(){{const value:int={name}();console.log(value);}}");
        let memory=Memory::new(&[("main.tsr",&source)]);
        assert_eq!(tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).err().unwrap().code,"type");
    }
    assert!(tsrust_compiler::native_bindings::lookup("std:os","hostname").unwrap_err().contains("does not export"));
    let memory=Memory::new(&[("main.tsr","import {hostname} from 'std:os';function main(){console.log(hostname());}")]);
    let error=load_project(&memory,"main.tsr").err().unwrap();
    assert_eq!(error.code,"module-native");
    assert!(error.message.contains("does not export") && error.message.contains("hostname"));
}

#[test]
fn os_identity_emits_only_referenced_target_components() {
    for (name,other) in [("platform","arch"),("arch","platform")] {
        let entry=support::format_round_trip(&format!("import {{{name}}} from 'std:os';function main(){{console.log({name}());}}"));
        let memory=Memory::new(&[("main.tsr",&entry)]);
        let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
        let rust=checked.emit_rust();
        assert!(rust.contains(&format!("fn tsrust_std_os_{name}()")));
        assert!(!rust.contains(&format!("tsrust_std_os_{other}")));
        assert!(rust.contains("#[cfg(not(any("));
    }
}

#[test]
fn variadic_native_path_join_borrows_arbitrary_string_arguments() {
    let entry=support::format_round_trip("import {join as combine} from 'std:path';import {strictEqual} from 'std:testing';function main(){const first='é😊';const last='report.txt';const stored=combine;strictEqual(combine(),'.');strictEqual(combine(first),first);strictEqual(stored('/tmp',first,'..',last),'/tmp/report.txt');strictEqual(first,'é😊');strictEqual(last,'report.txt');const parts=['tmp','é😊','report.txt'];strictEqual(combine(...parts),'tmp/é😊/report.txt');strictEqual(stored(...parts),'tmp/é😊/report.txt');strictEqual(parts[1],first);const annotated:Function<string,[...paths:string[]]>=combine;strictEqual(annotated('tmp','a','..',last),'tmp/report.txt');strictEqual(last,'report.txt');strictEqual(annotated(...parts),'tmp/é😊/report.txt');strictEqual(parts[1],first);}");
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"");}
}

#[test]
fn os_endianness_imported_zero_argument_contract() {
    let entry=support::format_round_trip("import {endianness as order} from 'std:os';import {strictEqual} from 'std:testing';function main(){const stored=order;const result:string=stored();strictEqual(result,order());console.log(result);}");
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    let expected=if cfg!(target_endian="big") {"BE\n"} else {"LE\n"};
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),expected);}
    for (body,code) in [("order(1);","arity"),("const result:int=order();console.log(result);","type")] {
        let source=format!("import {{endianness as order}} from 'std:os';function main(){{{body}}}");
        let memory=Memory::new(&[("main.tsr",&source)]);
        assert_eq!(tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).err().unwrap().code,code);
    }
}

#[test]
fn native_join_evaluates_once_in_order_and_cleans_failed_arguments() {
    let entry=support::format_round_trip("import {join} from 'std:path';import {strictEqual} from 'std:testing';function part(log:Mutable<int[]>,value:int):string{log.push(value);if(value===2){throw 'failed';}return 'x'.repeat(value);}function forwarded(action:Function<string,[...paths:string[]]>):string{return action('tmp','a','..','report');}function main(){const log:int[]=[];try{join(part(log,1),part(log,2),part(log,3));}catch(error){if(typeof error==='string'){strictEqual(error,'failed');}else{throw error;}}strictEqual(log.length,2);strictEqual(log[0],1);strictEqual(log[1],2);const result=join(part(log,3),part(log,4));strictEqual(result,'xxx/xxxx');strictEqual(log.length,4);strictEqual(log[2],3);strictEqual(log[3],4);strictEqual(forwarded(join),'tmp/report');}");
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    let mut rust=checked.emit_rust();
    assert!(!rust.contains("native_argument_0.clone"));
    let original_main=rust.rfind("fn main()").unwrap();
    // The process entry installs a panic hook that owns the previous hook.
    // Count only checked-program calls after that one-time entry initialization.
    let call="tsr_result_function0()";
    let suffix=rust[original_main..].replacen(call,&format!("{{let _join_guard=JoinLeakGuard(JOIN_LIVE.load(std::sync::atomic::Ordering::SeqCst));let first={call};assert!(first.is_ok());{call}}}"),1);
    rust.replace_range(original_main.., &suffix);
    rust.push_str(r#"
static JOIN_LIVE:std::sync::atomic::AtomicIsize=std::sync::atomic::AtomicIsize::new(0);
struct JoinLeakGuard(isize);
impl Drop for JoinLeakGuard {fn drop(&mut self){assert_eq!(JOIN_LIVE.load(std::sync::atomic::Ordering::SeqCst),self.0,"argument cleanup must release owned temporaries");}}
struct JoinAllocator;
unsafe impl std::alloc::GlobalAlloc for JoinAllocator {
 unsafe fn alloc(&self,layout:std::alloc::Layout)->*mut u8 {let p=unsafe{std::alloc::System.alloc(layout)};if !p.is_null(){JOIN_LIVE.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}p}
 unsafe fn dealloc(&self,p:*mut u8,layout:std::alloc::Layout){JOIN_LIVE.fetch_sub(1,std::sync::atomic::Ordering::SeqCst);unsafe{std::alloc::System.dealloc(p,layout)}}
 unsafe fn realloc(&self,p:*mut u8,layout:std::alloc::Layout,size:usize)->*mut u8 {unsafe{std::alloc::System.realloc(p,layout,size)}}
}
#[global_allocator] static JOIN_ALLOCATOR:JoinAllocator=JoinAllocator;
"#);
    for optimization in ["0","3"] {assert_eq!(native_stdout(&rust,optimization),"");}
}

#[test]
fn native_join_rejects_wrong_elements_and_has_no_fixed_arity_ceiling() {
    for arguments in ["1","'a',false","null","undefined","...[1,2]","...['a',1]"] {
        let entry=format!("import {{join}} from 'std:path';function main(){{console.log(join({arguments}));}}");
        let memory=Memory::new(&[("main.tsr",&entry)]);
        let error=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).err().unwrap();
        assert_eq!(error.code,"type","{arguments}: {error:?}");
    }
    let arguments=vec!["'a'";128].join(",");
    let entry=support::format_round_trip(&format!("import {{join}} from 'std:path';function main(){{console.log(join({arguments}));}}"));
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    assert_eq!(native_stdout(&checked.emit_rust(),"3"),vec!["a";128].join("/")+"\n");
    let binding=tsrust_compiler::native_bindings::lookup("std:path","join").unwrap();
    assert!(binding.accepts_arity(0) && binding.accepts_arity(128) && binding.accepts_arity(usize::MAX));
    assert_eq!(binding.parameters[0].access,tsrust_compiler::Mode::Read);
    assert!(binding.parameters[0].rest);
}

#[test]
fn native_join_reserves_composition_without_repeated_buffer_growth() {
    let rust=format!("mod path {{ {} }}\n{}",include_str!("../src/stdlib/path/runtime.rs"),r#"
static GROWTH:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
struct Counter;
unsafe impl std::alloc::GlobalAlloc for Counter {
 unsafe fn alloc(&self,l:std::alloc::Layout)->*mut u8 {unsafe{std::alloc::System.alloc(l)}}
 unsafe fn dealloc(&self,p:*mut u8,l:std::alloc::Layout){unsafe{std::alloc::System.dealloc(p,l)}}
 unsafe fn realloc(&self,p:*mut u8,l:std::alloc::Layout,n:usize)->*mut u8 {GROWTH.fetch_add(1,std::sync::atomic::Ordering::Relaxed);unsafe{std::alloc::System.realloc(p,l,n)}}
}
#[global_allocator] static ALLOCATOR:Counter=Counter;
fn main(){let mut paths=vec!["é😊"];paths.extend(std::iter::repeat_n("文",64));paths.extend(["..","résumé/"]);GROWTH.store(0,std::sync::atomic::Ordering::Relaxed);let result=path::join(&paths,false);let growth=GROWTH.load(std::sync::atomic::Ordering::Relaxed);assert_eq!(result,format!("é😊/{}résumé/","文/".repeat(63)));assert_eq!(growth,0,"composition should reserve text capacity once");}
"#);
    for optimization in ["0","3"] {assert_eq!(native_stdout(&rust,optimization),"");}
}

#[test]
fn imported_assertions_preserve_point_sensitive_nested_factory_contracts() {
    let entry=support::format_round_trip(r#"import {join} from "std:path";import {strictEqual} from "std:testing";function one<T>(value:T):T{return value;}
function two<T>(value:T,other:T):T{return other;}
function factory<A>(initial:A){let action=initial;return(restore:boolean)=>{const previous=action;if(restore){action=one;}else{action=two;}return previous;};}
function wrap<A>(initial:A){let action=factory(initial);return(restore:boolean)=>{return action(restore);};}
function main(){const saved=join;strictEqual(saved("a","b"),"a/b");const instances=[wrap(one)];for(let index=0;index<2;index+=1){instances.push(wrap(one));}const seed=instances.shift()!;const first=instances.shift()!;const second=instances.shift()!;const original=first(false);const independent=second(true);const changed=first(true);strictEqual(original<int>(1),1);strictEqual(independent<int>(2),2);strictEqual(changed<int>(3,9),9);}"#);
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"");}
}

#[test]
fn small_scalar_rest_forwarding_has_a_bounded_allocation_budget() {
    let entry=support::format_round_trip(r#"import {strictEqual} from 'std:testing';
function main(){const base=1;function sum(...values:int[]):int{let result=base;for(const value of values){result+=value;}return result;}const action=sum;const values=[1,2,3];for(let index=0;index<1000;index+=1){strictEqual(action(...values),7);}strictEqual(values[0],1);strictEqual(values.length,3);}"#);
    let memory=Memory::new(&[("main.tsr",&entry)]);
    let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
    let mut rust=checked.emit_rust().replacen("fn main()", "fn checked_entry()",1);
    rust.push_str(r#"
static SNAPSHOT_ALLOCATIONS: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
struct SnapshotAllocator;
unsafe impl std::alloc::GlobalAlloc for SnapshotAllocator {
    unsafe fn alloc(&self, layout:std::alloc::Layout)->*mut u8 {
        SNAPSHOT_ALLOCATIONS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
        unsafe {std::alloc::System.alloc(layout)}
    }
    unsafe fn dealloc(&self,pointer:*mut u8,layout:std::alloc::Layout) {
        unsafe {std::alloc::System.dealloc(pointer,layout)}
    }
    unsafe fn realloc(&self,pointer:*mut u8,layout:std::alloc::Layout,size:usize)->*mut u8 {
        SNAPSHOT_ALLOCATIONS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
        unsafe {std::alloc::System.realloc(pointer,layout,size)}
    }
}
#[global_allocator] static SNAPSHOT_ALLOCATOR:SnapshotAllocator=SnapshotAllocator;
fn main(){let before=SNAPSHOT_ALLOCATIONS.load(std::sync::atomic::Ordering::Relaxed);checked_entry();let used=SNAPSHOT_ALLOCATIONS.load(std::sync::atomic::Ordering::Relaxed)-before;assert!(used<=8,"small scalar rest snapshots must not allocate per call: {used}");}
"#);
    for optimization in ["0","3"] {assert_eq!(native_stdout(&rust,optimization),"");}
}

#[test]
fn caught_values_cross_generic_assertion_parameters() {
    for source in [
        r#"import {strictEqual} from 'std:testing';function main(){try{strictEqual(1,1);throw 3;}catch(error){strictEqual(error,3);}}"#,
        r#"import {strictEqual} from 'std:testing';function observe<T>(value:ReadOnly<T>):void{strictEqual(value,value);}function main(){try{strictEqual(1,1);throw 'payload';}catch(error){observe(error);strictEqual(error,'payload');}}"#,
        r#"import {strictEqual} from 'std:testing';function observe<T>(value:ReadOnly<T>):void{strictEqual(value,value);}function main(){try{const values=[1];console.log(values[9]);}catch(error){observe(error);strictEqual(error.code,'bounds');}}"#,
    ] {
        let entry=support::format_round_trip(source);
        let memory=Memory::new(&[("main.tsr",&entry)]);
        let checked=tsrust_compiler::project::check_graph(load_project(&memory,"main.tsr").unwrap()).unwrap();
        for optimization in ["0","3"] {assert_eq!(native_stdout(&checked.emit_rust(),optimization),"");}
    }
}
