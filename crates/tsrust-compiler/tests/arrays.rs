mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn scalar_arrays_infer_elements_and_execute_indexed_loops() {
    let rust = execute(
        include_str!("fixtures/arrays/sum.tsr"),
        "10\n4\ntrue\nfalse\n",
        None,
    );
    assert!(rust.contains("Array<i64>"));
    assert!(rust.contains("[bool; 2]"));
    assert!(!rust.contains("Clone::clone("));
    execute(
        "function main() { const a = [1, 2,]; const b = [true, false]; console.log(a[0] + a[1]); console.log(b[0] && !b[1]); }",
        "3\ntrue\n",
        None,
    );
}

#[test]
fn empty_literals_use_annotation_call_return_and_assignment_context() {
    execute(
        "function empty(): int[] { return []; } function count(a: boolean[]): int { return a.length; } function main() { let a: int[] = []; a.push(7); console.log(a[0]); a = []; console.log(a.length); const b = empty(); console.log(b.length); console.log(count([])); }",
        "7\n0\n0\n0\n",
        None,
    );
    reject("function main() { const a = []; }", "array-type");
    reject("function main() { const a: void[] = []; }", "array-element");
    reject(
        "function main() { const a: void[][] = []; }",
        "array-element",
    );
}

#[test]
fn arrays_move_and_explicit_clones_remain_independent() {
    execute(
        "function take(a: int[]): int[] { return a; } function main() { const original = [1, 2]; const copy = clone(original); copy[0] = 7; const moved = take(original); console.log(moved[0]); console.log(copy[0]); }",
        "1\n7\n",
        None,
    );
    reject(
        "function main() { const original = [1]; const next = original; console.log(original.length); }",
        "transferred",
    );
    reject(
        "function take(a: int[]): int[] { return a; } function main() { const a = [1]; const next = take(a); console.log(a[0]); }",
        "transferred",
    );
    execute(
        "function main() { let a = [1]; const b = a; a = [2]; console.log(a[0]); console.log(b[0]); }",
        "2\n1\n",
        None,
    );
}

#[test]
fn parameter_access_is_inferred_for_reads_writes_pushes_and_moves() {
    let source = "function read(a: int[]): int { return a[0]; } function size(a: int[]): int { return a.length; } function change(a: int[]) { a[0] = 7; } function append(a: int[]) { a.push(8); } function take(a: int[]): int[] { return a; } function main() { const a = [1]; change(a); append(a); console.log(read(a)); console.log(size(a)); const b = take(a); console.log(b[1]); }";
    execute(
        source,
        "7\n2\n8\n",
        Some(&[
            ("read.a", Mode::Read),
            ("size.a", Mode::Read),
            ("change.a", Mode::Mutate),
            ("append.a", Mode::Mutate),
            ("take.a", Mode::Consume),
        ]),
    );
    reject(
        "function pair(a: int[], b: int[]) { a.push(1); console.log(b.length); } function main() { const a: int[] = []; pair(a, a); }",
        "conflicting-access",
    );
}

#[test]
fn local_capture_loans_and_owned_returned_environments_cover_arrays() {
    execute(
        "function main() { const a = [1]; if (true) { const change = (): void => { a.push(2); a[0] = 7; }; change(); } console.log(a[0]); console.log(a.length); }",
        "7\n2\n",
        None,
    );
    reject(
        "function main() { const a = [1]; const read = (): int => { return a[0]; }; a.push(2); }",
        "capture-conflict",
    );
    reject(
        "function main() { const a = [1]; const change = (): void => { a.push(2); }; console.log(a.length); }",
        "capture-conflict",
    );
    execute(
        "function make(a: int[]): (n: int) => int { return (n: int): int => { a.push(n); return a.length; }; } function main() { const f = make([]); console.log(f(7)); console.log(f(8)); }",
        "1\n2\n",
        None,
    );
    execute(
        "function make(a: int[]): () => int[] { return (): int[] => { return a; }; } function main() { const f = make([7]); const a = f(); console.log(a[0]); }",
        "7\n",
        None,
    );
}

#[test]
fn arrays_obey_explicit_callback_argument_and_return_contracts() {
    execute(
        "function use(action: (a: Mutable<int[]>) => int, a: int[]): int { return action(a); } function main() { const f = (a: int[]): int => { a.push(7); return a.length; }; const a: int[] = []; console.log(use(f, a)); console.log(a[0]); }",
        "1\n7\n",
        None,
    );
    execute(
        "function identity(action: Once<Function<int[], [a: Owned<int[]>]>>): (a: Owned<int[]>) => int[] { return action; } function main() { const f = identity((a: int[]): int[] => { return a; }); const a = f([7]); console.log(a[0]); }",
        "7\n",
        None,
    );
    execute(
        "function use(action: (a: ReadOnly<boolean[]>) => boolean): boolean { return action([true]); } function main() { console.log(use((a: boolean[]): boolean => { return a[0]; })); }",
        "true\n",
        None,
    );
    reject(
        "function use(action: (a: int[]) => int) {} function main() {}",
        "callback-access",
    );
    reject(
        "function use(action: (a: ReadOnly<int[]>) => int) {} function main() { use((a: int[]): int => { a.push(7); return a.length; }); }",
        "callback-access",
    );
}

#[test]
fn evaluation_order_is_left_to_right_and_mutation_follows_operands() {
    execute(
        "function mark(n: int): int { console.log(n); return n; } function main() { const a = [mark(0), mark(1)]; a[mark(0)] = mark(7); a.push(a.length); a[1] = a[0]; console.log(a[1]); console.log(a[2]); }",
        "0\n1\n0\n7\n7\n2\n",
        None,
    );
    execute(
        "function grow(a: int[]): int { a.push(7); return 1; } function main() { const a = [0]; console.log(a[grow(a)]); a.push(grow(a)); console.log(a.length); }",
        "7\n4\n",
        None,
    );
    reject(
        "function take(a: int[]): int { const owned = a; return 0; } function main() { const a = [1]; console.log(a[take(a)]); }",
        "transferred",
    );
    reject(
        "function take(a: int[]): int { const owned = a; return 0; } function main() { const a = [1]; a.push(take(a)); }",
        "transferred",
    );
    reject(
        "function take(a: int[]): int { const owned = a; return 0; } function main() { const a = [1]; a[0] = take(a); }",
        "transferred",
    );
}

#[test]
fn branch_and_loop_ownership_join_array_moves_and_restoration() {
    execute(
        "function take(a: int[]) { const owned = a; console.log(owned[0]); } function main() { let a = [1]; let n = 0; while (n < 2) { take(a); a = [2]; n = n + 1; } console.log(a[0]); }",
        "1\n2\n2\n",
        None,
    );
    reject(
        "function take(a: int[]) { const owned = a; } function main() { const a = [1]; while (true) { take(a); } }",
        "loop-transfer",
    );
    reject(
        "function use(flag: boolean) { const a = [1]; if (flag) { const b = a; } console.log(a.length); } function main() {}",
        "transferred",
    );
}

#[test]
fn bounds_failures_include_source_ranges_in_debug_and_optimized_builds() {
    for statement in [
        "console.log(a[-1]);",
        "console.log(a[1]);",
        "a[-1] = 2;",
        "a[1] = 2;",
    ] {
        let source = format!("function main() {{ const a = [7]; {statement} }}");
        let analysis = check(&source).unwrap();
        let dir = support::TempDirectory::new();
        let path = dir.0.join("program.rs");
        std::fs::write(&path, analysis.emit_rust()).unwrap();
        let start = source.rfind("a[").unwrap();
        let end = start + source[start..].find(']').unwrap() + 1;
        for optimize in ["0", "3"] {
            let binary = dir.0.join("program");
            support::success(&support::run(
                support::rustc()
                    .arg("--edition=2024")
                    .arg(&path)
                    .arg("-C")
                    .arg(format!("opt-level={optimize}"))
                    .arg("-C")
                    .arg("overflow-checks=no")
                    .arg("-o")
                    .arg(&binary),
                60,
            ));
            let output = support::run(&mut std::process::Command::new(&binary), 10);
            assert!(!output.status.success());
            assert!(String::from_utf8(output.stderr).unwrap().contains(&format!(
                "array index out of bounds at source bytes {start}..{end}"
            )));
        }
    }
}

#[test]
fn element_types_indices_and_unsupported_collection_shapes_have_diagnostics() {
    for (body, code) in [
        ("const a: boolean[] = [1];", "type"),
        ("const a: Missing[] = [];", "array-element"),
        ("const a = [1]; a.push(true);", "type"),
        ("const a = [1]; a[true];", "type"),
        ("const a = [1]; a[0] = true;", "type"),
        ("const a = [1]; a.length = 0;", "unsupported"),
        ("const a = [1]; a.push();", "arity"),
        ("const a = [1]; const n = a.push(2);", "type"),
        ("const a = [1]; const value: int = a;", "type"),
        ("const a = [1]; console.log(a === a);", "type"),
        ("const a = [1]; a = [2];", "unsupported"),
        ("console.log([1][true]);", "type"),
        ("const a = [,];", "syntax"),
    ] {
        reject(&format!("function main() {{ {body} }}"), code);
    }
    reject(
        "type U = { items: void; }; function main() {}",
        "unsupported",
    );
}

#[test]
fn array_nesting_is_bounded_before_analysis() {
    let valid = format!(
        "function main() {{ const a = [0]; console.log({}0{}); }}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&valid).unwrap().emit_rust();
    let nested = format!(
        "function main() {{ const a = {}0{}; }}",
        "[".repeat(100),
        "]".repeat(100)
    );
    check(&nested).unwrap().emit_rust();
    let source = format!(
        "function main() {{ const a = {}0{}; }}",
        "[".repeat(200),
        "]".repeat(200)
    );
    reject(&source, "unsupported");
    let source = format!(
        "function main() {{ const a = [1]; console.log({}0{}); }}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    support::accept_deep(&source);
}

#[test]
fn scalar_array_analysis_does_not_invoke_the_backend() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/arrays/sum.tsr"));
    for command in ["check", "emit"] {
        let output = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&output);
    }
}
