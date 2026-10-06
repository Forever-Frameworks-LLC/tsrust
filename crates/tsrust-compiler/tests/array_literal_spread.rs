mod support;
use tsrust_compiler::check;

fn recorded(
    directory: &std::path::Path,
    label: &str,
    command: &mut std::process::Command,
    timeout: u64,
) -> std::process::Output {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    let stamp = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|value| format!("{:?}", value.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(",");
    let start = stamp();
    let elapsed = Instant::now();
    let stdout = directory.join(format!("{label}.stdout"));
    let stderr = directory.join(format!("{label}.stderr"));
    command
        .stdout(std::fs::File::create(&stdout).unwrap())
        .stderr(std::fs::File::create(&stderr).unwrap());
    let mut child = command.spawn().unwrap();
    let pid = child.id();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if elapsed.elapsed() > Duration::from_secs(timeout) {
            child.kill().unwrap();
            panic!("native child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    std::fs::write(directory.join(format!("{label}.json")),format!("{{\"argv\":[{argv}],\"pid\":{pid},\"startUtcEpochMillis\":{start},\"endUtcEpochMillis\":{},\"exitCode\":{}}}\n",stamp(),status.code().map_or("null".into(),|value|value.to_string()))).unwrap();
    std::process::Output {
        status,
        stdout: std::fs::read(stdout).unwrap(),
        stderr: std::fs::read(stderr).unwrap(),
    }
}

fn native(source: &str, expected: &str) {
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let persistent = std::env::var_os("TSR_GATE101_NATIVE_DIR");
    let temporary = persistent.is_none().then(support::TempDirectory::new);
    let directory = if let Some(root) = persistent {
        let root = std::path::PathBuf::from(root);
        let case = CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if case == 0 {
            root
        } else {
            root.join(format!("case-{case}"))
        }
    } else {
        temporary.as_ref().unwrap().0.clone()
    };
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("input.tsr"), source).unwrap();
    std::fs::write(directory.join("expected.txt"), expected).unwrap();
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    std::fs::write(directory.join("generated.rs"), &rust).unwrap();
    assert!(!rust.contains("unsafe"));
    if std::env::var_os("TSR_GATE101_EMIT_ONLY").is_some() {
        return;
    }
    for level in ["0", "3"] {
        let path = directory.join(format!("native-O{level}"));
        let build = recorded(
            &directory,
            &format!("build-O{level}"),
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(directory.join("generated.rs"))
                .arg("-o")
                .arg(&path),
            60,
        );
        std::fs::write(
            directory.join(format!("build-O{level}-stdout.txt")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-O{level}-stderr.txt")),
            &build.stderr,
        )
        .unwrap();
        support::success(&build);
        let output = recorded(
            &directory,
            &format!("run-O{level}"),
            &mut std::process::Command::new(path),
            15,
        );
        std::fs::write(
            directory.join(format!("stdout-O{level}.txt")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("stderr-O{level}.txt")),
            &output.stderr,
        )
        .unwrap();
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn dense_literal_spreads_snapshot_in_source_order() {
    native(r#"
function grow(values: Mutable<int[]>): int { values.push(9); return values.length; }
function main() {
    const values = [1, 2];
    const result = [0, ...values, grow(values), ...values, 8];
    for (const value of result) { console.log(value); }
    console.log(values.length);
    const empty: int[] = [];
    const nested = [...[...empty, ...[4, 5]], 6];
    for (const value of nested) { console.log(value); }
}
"#, "0\n1\n2\n3\n1\n2\n9\n8\n3\n4\n5\n6\n");
}

#[test]
fn owning_literal_spreads_move_strings_records_and_nested_arrays() {
    native(r#"
type Row = { value: int; };
function main() {
    const strings = ["a", "b"];
    const joined = [...strings, "c"];
    for (const value of joined) { console.log(value); }
    const rows: Row[] = [{ value: 7 }];
    const moved = [...rows, { value: 8 }];
    for (const value of moved) { console.log(value.value); }
    const arrays = [[10], [11, 12]];
    const flattened = [...arrays, [13]];
    for (const row of flattened) { for (const value of row) { console.log(value); } }
}
"#, "a\nb\nc\n7\n8\n10\n11\n12\n13\n");
}

#[test]
fn literal_spread_preserves_function_call_expansion_counts() {
    native(r#"
function add(a: int, b: int): int { return a + b; }
function main() { const values = [4,5]; console.log(add(...[...values])); }
"#, "9\n");
}

#[test]
fn literal_spread_rejects_non_arrays_mixed_types_and_moved_reuse() {
    for source in [
        "function main() { const values = [...1]; }",
        "function main() { const values = [...[1], true]; }",
        "function main() { const values = [\"a\"]; const moved = [...values]; console.log(values.length); }",
    ] { assert!(check(source).is_err(), "{source}"); }
}

#[test]
fn contextual_empty_literal_spread_and_generic_boundary() {
    native(r#"
function size<T>(values: ReadOnly<T[]>): int { return values.length; }
function main() {
    const empty: int[] = [...[]];
    console.log(empty.length);
    const values = [3, 4];
    console.log(size([...values, 5]));
}
"#, "0\n3\n");
}

#[test]
fn number_context_is_inferred_across_scalar_and_spread_segments() {
    native(r#"
function main() {
    const numbers: number[] = [2.5, 3.5];
    const values = [1, ...numbers, 4];
    for (const value of values) { console.log(value); }
    console.log(numbers.length);
    const flags = [true, false];
    const copied = [...flags, true];
    for (const value of copied) { console.log(value); }
    console.log(flags.length);
}
"#, "1\n2.5\n3.5\n4\n2\ntrue\nfalse\ntrue\n2\n");
}

#[test]
fn stored_callable_literal_spreads_keep_captures_and_invocation_contracts() {
    native(r#"
function make(value: int): Function<int> { return (): int => { return value; }; }
function main() {
    const actions: Function<int>[] = [make(7), make(9)];
    const moved: Function<int>[] = [...actions, (): int => { return 11; }];
    console.log(moved[0](), moved[1](), moved[2]());
    const single: Once<Function<int>>[] = [(): int => { return 13; }];
    const once: Once<Function<int>>[] = [...single];
    const finish = once.remove(0);
    console.log(finish());
}
"#, "7 9 11\n13\n");
}

#[test]
fn null_undefined_and_narrowed_sources_preserve_dense_values() {
    native(r#"
function main() {
    const nulls: null[] = [null, null];
    const combined: null[] = [...nulls, null];
    console.log(combined.length, nulls.length);
    const absents: undefined[] = [undefined];
    const expanded: undefined[] = [...absents, undefined];
    console.log(expanded.length, absents.length);
    const maybe: int[] | null = [6, 7];
    if (maybe !== null) { const values = [...maybe, 8]; console.log(values.join("|")); }
}
"#, "3 2\n2 1\n6|7|8\n");
}

#[test]
fn owned_sources_and_borrowed_owners_cannot_escape_or_be_reused() {
    for source in [
        "function copy(values: ReadOnly<string[]>): string[] { return [...values]; } function main() {}",
        "function main() { const actions: Function<int>[] = [(): int => { return 1; }]; const moved = [...actions]; console.log(actions.length); }",
        "function main() { const maybe: int[] | null = null; const values = [...maybe]; }",
        "function main() { const values = [...[null], undefined]; }",
        "function main() { const values = [1, , ...[2]]; }",
    ] { assert!(check(source).is_err(), "{source}"); }
}

#[test]
fn published_language_examples_match_native_output() {
    native(include_str!("fixtures/array-literal-spread/language-examples.tsr"), include_str!("fixtures/array-literal-spread/language-examples.stdout"));
}
