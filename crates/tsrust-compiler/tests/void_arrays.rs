mod support;
use tsrust_compiler::{check, project::{FileSourceProvider, check_graph, load_project}};

fn execute_project(source: &str) -> String {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let graph = load_project(&FileSourceProvider, path.to_str().unwrap()).unwrap();
    let checked = check_graph(graph).unwrap_or_else(|error| panic!("{}: {} at {:?}", error.code, error.message, error.location));
    assert!(checked.warnings().is_empty());
    let rust = checked.emit_rust();
    assert!(!rust.contains("unsafe"));
    let input = directory.0.join("unit-arrays.rs");
    let executable = directory.0.join(format!("unit-arrays{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&input, &rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(&input).arg("-o").arg(&executable), 180));
    let output = support::run(&mut std::process::Command::new(executable), 30);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    rust
}

#[test]
fn unit_arrays_preserve_count_nested_alias_and_generic_results() {
    let source = r#"
import { strictEqual } from 'std:testing';
type Units = void[];
type Batch<T> = T[];
function done(): void {}
function empty<T>(): T[] { return []; }
function keep<T>(values: T[]): T[] { return values; }
function main() {
    const values: Units = [done(), done()];
    values.push(done());
    strictEqual(values.length, 3);
    const copied = values.slice();
    copied.sort();
    copied.reverse();
    strictEqual(copied.length, 3);
    strictEqual(values.length, 3);
    let count = 0;
    for (const value of values) { count += 1; }
    strictEqual(count, 3);
    const nested: void[][] = [keep<void>(copied), empty<void>()];
    strictEqual(nested.length, 2);
    strictEqual(nested[0].length, 3);
    strictEqual(nested[1].length, 0);
    const generic: Batch<void> = empty<void>();
    strictEqual(generic.length, 0);
    const mapped = values.map(() => { done(); });
    strictEqual(mapped.length, 3);
    const filtered = values.filter(() => true);
    strictEqual(filtered.length, 3);
    const reduced = values.reduce((total) => total + 1, 0);
    strictEqual(reduced, 3);
    const flattened = values.flatMap(() => [done()]);
    strictEqual(flattened.length, 3);
    const entries = nested.entries();
    strictEqual(entries.length, 2);
    strictEqual(entries[0][1].length, 3);
    strictEqual(entries[1][1].length, 0);
    const absent: undefined[] = [undefined];
    strictEqual(absent.length, 1);
}
"#;
    let rust = execute_project(source);
    assert!(rust.contains("Vec<()>"), "unit arrays must use existing Rust unit storage");
    assert!(!rust.contains("unsafe"));
}

#[test]
fn unit_array_annotations_do_not_widen_void_or_undefined() {
    let capabilities = |ty: &str| {
        let analysis = check(&format!("function probe(values:{ty}){{}}function main(){{}}")).unwrap();
        analysis.checked_type_facts().array_capabilities[0].1
    };
    let discarded = "function probe(values:ReadOnly<void[]>){values;}function main(){}";
    let selected = discarded.rfind("values;").unwrap();
    let facts = check(discarded).unwrap().checked_type_facts();
    assert!(facts.array_capabilities.iter().any(|(span, _)| span.start == selected && span.end == selected + 6));
    let scalar = capabilities("void[]");
    for method in ["at", "pop", "shift", "find", "findLast", "entries", "join", "toString", "toLocaleString"] { assert!(!scalar.supports(method), "{method}"); }
    assert!(scalar.default_sort);
    let nested = capabilities("void[][]");
    for method in ["at", "pop", "shift", "find", "findLast", "entries"] { assert!(nested.supports(method), "{method}"); }
    assert!(!nested.supports("join"));
    assert!(!capabilities("Map<int,int>[]").default_sort);
    let aliases = check("type Pair=[void[]];type Pairs=Pair[];function probe(values:Pairs){}function main(){}").unwrap();
    assert!(!aliases.checked_type_facts().array_capabilities[0].1.supports("join"));
    for source in [
        "function main(){const values:void[]=[undefined];}",
        "function done():void{}function main(){const values:undefined[]=[done()];}",
        "function main(){const value:void=undefined;}",
        "function main(){const values:(void|undefined)[]=[];}",
    ] {
        let source = support::format_round_trip(source);
        assert!(check(&source).is_err(), "{source}");
    }
}

#[test]
fn unit_array_unsupported_optional_and_text_results_are_source_diagnostics() {
    for operation in ["values.at(0)", "values.pop()", "values.shift()"] {
        let source = support::format_round_trip(&format!("function main(){{const values:void[]=[];{operation};}}"));
        assert_eq!(check(&source).err().expect("source diagnostic").code, "void-element-option", "{source}");
    }
    let sorting = support::format_round_trip("function main(){const values:Map<int,int>[]=[];values.sort();}");
    assert_eq!(check(&sorting).err().expect("nonprimitive sort diagnostic").code, "type");
    let entries = support::format_round_trip("function main(){const values:void[]=[];values.entries();}");
    assert_eq!(check(&entries).err().expect("unit tuple diagnostic").code, "void-array-entry");
    let tuple = support::format_round_trip("function main(){const values:[void[]]=[[]];String(values);}");
    assert_eq!(check(&tuple).err().expect("tuple text diagnostic").code, "void-array-text");
    for operation in ["values.join()", "String(values)", "values.toString()", "values.toLocaleString()", "`${values}`"] {
        for ty in ["void[]", "void[][]"] {
            let source = support::format_round_trip(&format!("function main(){{const values:{ty}=[];{operation};}}"));
            assert_eq!(check(&source).err().expect("source diagnostic").code, "void-array-text", "{source}");
        }
    }
}

#[test]
fn unit_arrays_are_owned_task_and_job_results() {
    execute_project(r#"
import { strictEqual } from 'std:testing';
async function done(): Task<void> {}
async function collect(): Task<void[]> { return [await done(), await done()]; }
async function main(): Task<void> {
    const direct = await collect();
    strictEqual(direct.length, 2);
    const job = collect().start();
    const started = await job;
    strictEqual(started.length, 2);
}
"#);
}

#[test]
fn unit_tuples_do_not_register_join_text_for_unrelated_allowed_conversions() {
    support::execute(r#"
function main() {
    const units: [void[]] = [[]];
    const nested: [[void[]]] = [[[]]];
    const rows: [void[]][] = [[[]]];
    const text = [1, 2].join();
    console.log(text, units.length, nested.length, rows.length);
}
"#, "1,2 1 1 1\n", None);
}
