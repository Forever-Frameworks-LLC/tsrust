mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn nested_scalar_arrays_execute_as_owned_vectors() {
    let rust = execute(
        include_str!("fixtures/nested-arrays/matrix.tsr"),
        "16\n7\n9\n",
        Some(&[("total.rows", Mode::Read)]),
    );
    assert!(rust.contains("Array<NumericArray<i64>>"));
    execute(
        "function main() { const flags = [[true, false], []]; for (const row of flags) { for (const flag of row) { console.log(flag); } } console.log(flags[1].length); }",
        "true\nfalse\n0\n",
        None,
    );
}

#[test]
fn ranks_and_nominal_leaves_match_in_all_type_contexts() {
    execute(
        "type U = { name: string; }; function empty(): U[][] { return []; } function count(a: string[][][]): int { return a.length; } function main() { let a = empty(); a = [[{ name: \"Pat\" }], []]; console.log(a[0][0].name); a[0].push({ name: \"Sam\" }); console.log(a[0][1].name); const b: string[][][] = [[[\"Alex\"]]]; console.log(b[0][0][0]); console.log(count([])); }",
        "Pat\nSam\nAlex\n0\n",
        None,
    );
    for source in [
        "function main() { const a: int[][] = [1]; }",
    ] {
        reject(source, "type");
    }
    reject("function main() { const a = [[], [1]]; }", "array-type");
    let error = reject("function main() { const a: string[][][] = [[1]]; }", "type");
    assert!(error.message.contains("string[]"));
}

#[test]
fn whole_rows_move_into_arrays_and_require_explicit_clones_on_extraction() {
    execute(
        "function take(a: int[][]): int[][] { return a; } function main() { const row = [1, 2]; const a = [row]; const next = [3]; a.push(next); a[0] = [7]; const saved = clone(a[0]); const moved = take(a); console.log(saved[0]); console.log(moved[1][0]); }",
        "7\n3\n",
        Some(&[("take.a", Mode::Consume)]),
    );
    for body in [
        "const row = [1]; const a = [row]; console.log(row.length);",
        "const a: int[][] = []; const row = [1]; a.push(row); console.log(row.length);",
        "const a = [[1]]; a.push(a[0]);",
    ] {
        reject(
            &format!("function main() {{ {body} }}"),
            if body.contains("const row") {
                "transferred"
            } else {
                "borrowed-element"
            },
        );
    }
    reject(
        "function get(a: int[][]): int[] { return a[0]; } function main() {}",
        "borrowed-element",
    );
}

#[test]
fn child_arrays_and_record_leaves_accept_mutable_arguments() {
    execute(
        "type U = { n: int; }; function append(row: int[]) { row.push(7); } function change(u: U) { u.n = 8; } function size(a: U[][]): int { return a.length; } function main() { const a = [[1]]; append(a[0]); console.log(a[0][1]); const users: U[][] = [[{ n: 1 }]]; change(users[0][size(users) - 1]); users[0][0].n = 9; console.log(users[0][0].n); }",
        "7\n9\n",
        Some(&[
            ("append.row", Mode::Mutate),
            ("change.u", Mode::Mutate),
            ("size.a", Mode::Read),
        ]),
    );
    reject(
        "function pair(a: int[], b: int[]) { a.push(b[0]); } function main() { const a = [[1], [2]]; pair(a[0], a[1]); }",
        "conflicting-access",
    );
    execute(
        "function pair(a: int[], b: int[]): int { return a[0] + b[0]; } function main() { const a = [[1], [2]]; console.log(pair(a[0], a[1])); }",
        "3\n",
        None,
    );
}

#[test]
fn nested_iteration_preserves_shared_access_and_scoped_row_borrows() {
    execute(
        "function main() { const a = [[1, 2]]; for (const n of clone(a[0])) { a[0].push(n); } console.log(a[0].length); }",
        "4\n",
        None,
    );
    execute(
        "function main() { const a = [[1, 2], [3]]; for (const row of a) { for (const n of row) { if (n === 2) { continue; } console.log(n); } } for (const n of a[0]) { console.log(n); break; } a[0].push(4); console.log(a[0].length); }",
        "1\n3\n1\n3\n",
        None,
    );
    for body in ["row.push(2);", "row = [2];"] {
        reject(
            &format!("function main() {{ const a = [[1]]; for (let row of a) {{ {body} }} }}"),
            "borrowed-element",
        );
    }
    reject(
        "function main() { const a = [[1]]; for (const n of a[0]) { a[0].push(2); } }",
        "iteration-conflict",
    );
    reject(
        "function main() { const a = [[1]]; for (const row of a) { a[0][0] = 2; } }",
        "iteration-conflict",
    );
    execute(
        "function main() { for (const row of [[1], [2]]) { const owned = clone(row); owned.push(7); console.log(owned[1]); } }",
        "7\n7\n",
        None,
    );
}

#[test]
fn selected_parents_stay_stable_during_later_index_and_value_evaluation() {
    execute(
        "function grow(a: int[][]): int { a.push([3]); return 0; } function main() { const a = [[7, 8], [1]]; console.log(a[grow(a)].length); console.log(a[0][a[1][0]]); a[0][0] = a[1][0]; a[0].push(a[1].length); console.log(a[0][0]); console.log(a[0][2]); }",
        "2\n8\n1\n1\n",
        None,
    );
    for body in [
        "console.log(a[0][grow(a)]);",
        "a[0][0] = grow(a);",
        "a[0].push(grow(a));",
        "a[0][0] = take(a);",
    ] {
        let error = reject(
            &format!(
                "function grow(a: int[][]): int {{ a.push([2]); return 0; }} function take(a: int[][]): int {{ const moved = a; return 0; }} function main() {{ const a = [[1]]; {body} }}"
            ),
            "conflicting-access",
        );
        assert_eq!(error.related.len(), 1);
        let location = error.related[0].0;
        assert_eq!(location.end - location.start, "a[0]".len());
    }
    execute(
        "function grow(a: int[][]): int[] { a.push([3]); return [7]; } function main() { const a = [[1]]; a[0] = grow(a); console.log(a[0][0]); console.log(a.length); }",
        "7\n2\n",
        None,
    );
}

#[test]
fn nested_indices_and_push_receivers_evaluate_once_in_source_order() {
    execute(
        "function mark(n: int): int { console.log(n); return n; } function main() { const a = [[1, 2]]; a[mark(0)][mark(1)] = mark(7); console.log(a[mark(0)][mark(1)]); a[mark(0)].push(mark(8)); console.log(a[0][2]); const b = [[[1]]]; b[mark(0)][mark(0)][mark(0)] = mark(9); console.log(b[0][0][0]); }",
        "0\n1\n7\n0\n1\n7\n0\n8\n8\n0\n0\n0\n9\n9\n",
        None,
    );
    execute(
        "type U = { name: string; }; function main() { const a: U[][] = [[{ name: \"Pat\" }], [{ name: \"Sam\" }]]; a[0][0].name = clone(a[1][0].name); console.log(a[0][0].name); }",
        "Sam\n",
        None,
    );
}

#[test]
fn nested_arrays_support_captures_owned_returns_and_callback_contracts() {
    execute(
        "function make(a: int[][]): () => int { return (): int => { let total = 0; for (const row of a) { const read = (): int => { return row[0]; }; total = total + read(); } return total; }; } function identity(f: Function<string[][], [a: Owned<string[][]>]>): (a: Owned<string[][]>) => string[][] { return f; } function main() { const f = make([[1], [2]]); console.log(f()); console.log(f()); const g = identity((a: string[][]): string[][] => { return a; }); const a = g([[\"Pat\"]]); console.log(a[0][0]); }",
        "3\n3\nPat\n",
        None,
    );
    reject(
        "function keep(f: Function<int>) {} function main() { const a = [[1]]; for (const row of a) { keep((): int => { return row[0]; }); } }",
        "borrowed-element",
    );
    execute(
        "function keep(f: Function<int>): () => int { return f; } function main() { const a = [[1]]; for (const row of a) { const saved = clone(row); const f = keep((): int => { return saved[0]; }); console.log(f()); } }",
        "1\n",
        None,
    );
    execute(
        "function use(f: (a: Mutable<int[][]>) => void, a: int[][]) { f(a); } function main() { const a = [[1]]; use((values: int[][]): void => { values[0].push(7); }, a); console.log(a[0][1]); }",
        "7\n",
        None,
    );
}

#[test]
fn nested_clone_and_reinitialization_preserve_independent_storage() {
    execute(
        "function main() { const a = [[\"Pat\"]]; a[0].push(clone(a[0][0])); console.log(a[0][1]); }",
        "Pat\n",
        None,
    );
    execute(
        "type U = { name: string; }; function main() { let a: U[][] = [[{ name: \"Pat\" }]]; const b = clone(a); b[0][0].name = \"Sam\"; console.log(a[0][0].name); console.log(b[0][0].name); const moved = a; a = [[{ name: \"Alex\" }]]; console.log(moved[0][0].name); console.log(a[0][0].name); }",
        "Pat\nSam\nPat\nAlex\n",
        None,
    );
    reject(
        "function main() { const a = [[1]]; while (true) { const moved = a; } }",
        "loop-transfer",
    );
    reject(
        "function main() { const a = [[1]]; const read = (): int => { return a[0][0]; }; a[0].push(2); }",
        "capture-conflict",
    );
}

#[test]
fn nested_bounds_fail_at_the_correct_stage_under_optimization() {
    for (body, fragment, stdout) in [
        ("console.log(a[-1][0]);", "a[-1]", ""),
        ("console.log(a[0][2]);", "a[0][2]", ""),
        ("a[1][0] = mark(7);", "a[1]", ""),
        ("a[0][-1] = mark(7);", "a[0][-1]", ""),
        ("a[1].push(mark(7));", "a[1]", ""),
    ] {
        let source = format!(
            "function mark(n: int): int {{ console.log(n); return n; }} function main() {{ const a = [[1]]; {body} }}"
        );
        let dir = support::TempDirectory::new();
        let path = dir.0.join("program.rs");
        std::fs::write(&path, check(&source).unwrap().emit_rust()).unwrap();
        let start = source.rfind(fragment).unwrap();
        let end = start + fragment.len();
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
            assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
            assert!(String::from_utf8(output.stderr).unwrap().contains(&format!(
                "array index out of bounds at source bytes {start}..{end}"
            )));
        }
    }
}

#[test]
fn deep_valid_nested_arrays_compile_and_excessive_rank_is_rejected() {
    let source = format!(
        "function main() {{ const a = {}7{}; console.log(a{}); }}",
        "[".repeat(100),
        "]".repeat(100),
        "[0]".repeat(100)
    );
    let rust = execute(&source, "7\n", None);
    assert!(!rust.contains("Clone::clone("));
    let write = format!(
        "function main() {{ const a = {}7{}; a{} = 9; console.log(a{}); }}",
        "[".repeat(100),
        "]".repeat(100),
        "[0]".repeat(100),
        "[0]".repeat(100)
    );
    execute(&write, "9\n", None);
    let boundary = format!(
        "function empty(): int{} {{ return []; }} function main() {{ const a = empty(); const b = clone(a); console.log(b.length); }}",
        "[]".repeat(128)
    );
    execute(&boundary, "0\n", None);
    reject(
        &format!(
            "function main() {{ const a: int{} = []; }}",
            "[]".repeat(129)
        ),
        "unsupported",
    );
    reject(
        &format!(
            "function main() {{ const a = {}7{}; }}",
            "[".repeat(200),
            "]".repeat(200)
        ),
        "unsupported",
    );
}

#[test]
fn nested_array_analysis_and_emission_are_backend_independent() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/nested-arrays/matrix.tsr"));
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
    reject(
        "type U = { values: void; }; function main() {}",
        "unsupported",
    );
    reject(
        "function main() { const a: void[][] = []; }",
        "array-element",
    );
}
