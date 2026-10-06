mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn removal_moves_records_without_cloning_and_keeps_remaining_order() {
    let rust = execute(
        include_str!("fixtures/array-removal/queue.tsr"),
        "Sam\nsaved\n2\nPat\nAlex\nSam\n",
        Some(&[("take.users", Mode::Mutate), ("take.index", Mode::Copy)]),
    );
    assert!(rust.contains(".remove(removal_index)"));
    assert!(!rust.contains("Clone::clone"));
    assert!(!rust.contains("unsafe"));
    assert!(!rust.contains("Box<"));
}

#[test]
fn scalar_and_owned_results_match_exact_leaf_and_rank_types() {
    execute(
        "function main() { const ints = [1, 2, 3]; const n: int = ints.remove(1); console.log(n); console.log(ints[1]); const flags = [true, false]; const flag: boolean = flags.remove(0); console.log(flag); console.log(flags[0]); const rows: string[][] = [[\"Pat\"], [\"Sam\"]]; const row: string[] = rows.remove(0); row.push(\"Alex\"); console.log(row[1]); console.log(rows[0][0]); }",
        "2\n3\ntrue\nfalse\nAlex\nSam\n",
        None,
    );
    for body in [
        "const n: boolean = values.remove(0);",
        "const row: int[] = values.remove(0);",
    ] {
        reject(
            &format!("function main() {{ const values = [1]; {body} }}"),
            "type",
        );
    }
    reject(
        "type A = { n: int; }; type B = { n: int; }; function main() { const values: A[] = [{ n: 1 }]; const b: B = values.remove(0); }",
        "type",
    );
}

#[test]
fn removed_strings_move_into_calls_records_and_arrays() {
    execute(
        "type Box = { name: string; }; function take(s: string): string { return s; } function main() { const names = [\"Pat\", \"Sam\", \"Alex\"]; console.log(take(names.remove(0))); const box: Box = { name: names.remove(0) }; const saved = [names.remove(0)]; console.log(box.name); console.log(saved[0]); console.log(names.length); names.push(\"New\"); console.log(names[0]); }",
        "Pat\nSam\nAlex\n0\nNew\n",
        Some(&[("take.s", Mode::Consume)]),
    );
    reject(
        "function main() { const a = [\"Pat\"]; const removed = a.remove(0); const moved = removed; console.log(removed); }",
        "transferred",
    );
}

#[test]
fn removal_supports_nested_rows_and_mixed_record_storage_paths() {
    execute(
        "type Inner = { rows: string[][]; }; type Outer = { inner: Inner; }; function take(box: Outer): string[] { return box.inner.rows.remove(0); } function nested(boxes: Outer[]): string { return boxes[0].inner.rows[0].remove(0); } function main() { const box: Outer = { inner: { rows: [[\"Pat\"], [\"Sam\", \"Alex\"]] } }; const row = take(box); console.log(row[0]); const boxes = [box]; console.log(nested(boxes)); console.log(boxes[0].inner.rows[0][0]); console.log(boxes[0].inner.rows.length); row.push(\"New\"); console.log(row[1]); }",
        "Pat\nSam\nAlex\n1\nNew\n",
        Some(&[("take.box", Mode::Mutate), ("nested.boxes", Mode::Mutate)]),
    );
    execute(
        "type Node = { n: int; children: Node[]; }; function main() { const root: Node = { n: 0, children: [{ n: 1, children: [{ n: 2, children: [] }] }] }; const child = root.children.remove(0); const grandchild = child.children.remove(0); console.log(grandchild.n); }",
        "2\n",
        None,
    );
}

#[test]
fn receiver_restrictions_and_borrowed_indexing_remain_explicit() {
    for source in [
        "function main() { [1].remove(0); }",
        "function make(): int[] { return [1]; } function main() { make().remove(0); }",
        "function main() { const a = [1]; clone(a).remove(0); }",
    ] {
        reject(source, "unsupported");
    }
    // Indexing an owned element borrows it; the binding is a read view.
    assert!(check("function main() { const a = [\"Pat\"]; const s = a[0]; }").is_ok());
    reject(
        "type Box = { names: string[]; }; function main() { const box: Box = { names: [\"Pat\"] }; const a = box.names; console.log(box.names.length); }",
        "partial-transfer",
    );
}

#[test]
fn named_receiver_uses_final_storage_after_index_side_effects() {
    execute(
        "function grow(a: int[]): int { a.push(9); return 1; } function shrink(a: int[]): int { console.log(a.remove(0)); return 0; } function main() { const a = [1]; console.log(a.remove(grow(a))); console.log(a[0]); a.push(2); console.log(a.remove(shrink(a))); console.log(a.length); const b = [7, 8]; b.push(b.remove(0)); console.log(b[0]); console.log(b[1]); }",
        "9\n1\n1\n2\n0\n8\n7\n",
        Some(&[("grow.a", Mode::Mutate), ("shrink.a", Mode::Mutate)]),
    );
    execute(
        "function size(a: int[]): int { return a.length - 1; } function main() { const a = [1, 2]; console.log(a.remove(size(a))); console.log(a[0]); }",
        "2\n1\n",
        Some(&[("size.a", Mode::Read)]),
    );
    reject(
        "function take(a: int[]): int { const moved = a; return 0; } function main() { const a = [1]; a.remove(take(a)); }",
        "transferred",
    );
}

#[test]
fn mixed_receiver_indices_evaluate_once_before_removal_index() {
    execute(
        "type Box = { rows: int[][]; }; function mark(n: int): int { console.log(n); return n; } function main() { const boxes: Box[] = [{ rows: [[1, 2]] }]; console.log(boxes[mark(0)].rows[mark(0)].remove(mark(1))); console.log(boxes[0].rows[0][0]); }",
        "0\n0\n1\n2\n1\n",
        Some(&[("mark.n", Mode::Copy)]),
    );
    execute(
        "type Box = { rows: int[][]; }; function read(box: Box): int { return box.rows[0].length - 1; } function main() { const box: Box = { rows: [[1, 2]] }; console.log(box.rows[0].remove(read(box))); }",
        "2\n",
        Some(&[("read.box", Mode::Read)]),
    );
}

#[test]
fn selection_and_argument_loans_protect_receivers() {
    for body in [
        "box.rows.remove(change(box));",
        "box.rows[0].remove(change(box));",
    ] {
        let diagnostic = reject(
            &format!(
                "type Box = {{ rows: int[][]; }}; function change(box: Box): int {{ box.rows.push([7]); return 0; }} function main() {{ const box: Box = {{ rows: [[1]] }}; {body} }}"
            ),
            "conflicting-access",
        );
        assert!(!diagnostic.related.is_empty());
    }
    reject(
        "function change(a: int[][]): int { a.remove(0); return 0; } function main() { const a = [[1]]; a[0].remove(change(a)); }",
        "conflicting-access",
    );
    reject(
        "function pair(a: string[], s: string) {} function main() { const a = [\"Pat\"]; pair(a, a.remove(0)); }",
        "conflicting-access",
    );
    execute(
        "function pair(s: string, a: string[]) { console.log(s); console.log(a.length); } function main() { const a = [\"Pat\"]; pair(a.remove(0), a); }",
        "Pat\n0\n",
        Some(&[("pair.s", Mode::Read), ("pair.a", Mode::Read)]),
    );
}

#[test]
fn shared_iteration_and_capture_loans_reject_removal() {
    for body in [
        "a.remove(0);",
        "const remove = (): int => { return a.remove(0); };",
    ] {
        reject(
            &format!("function main() {{ const a = [1]; for (const n of a) {{ {body} }} }}"),
            "iteration-conflict",
        );
    }
    reject(
        "function main() { const a = [[1]]; for (const row of a) { row.remove(0); } }",
        "borrowed-element",
    );
    reject(
        "function main() { const a = [1]; const read = (): int => { return a[0]; }; a.remove(0); }",
        "capture-conflict",
    );
    reject(
        "function main() { const a = [1]; const remove = (): int => { return a.remove(0); }; console.log(a.length); }",
        "capture-conflict",
    );
    execute(
        "function main() { const a = [[1]]; for (const row of a) { const saved = clone(row); console.log(saved.remove(0)); } console.log(a[0][0]); for (const row of clone(a)) { const saved = a.remove(0); console.log(saved[0]); } }",
        "1\n1\n1\n",
        None,
    );
}

#[test]
fn returned_elements_and_mutating_factories_keep_independent_ownership() {
    let source = "function make(a: string[]): () => string { return (): string => { return a.remove(0); }; } function save(a: string[]): () => string { const s = a.remove(0); return (): string => { return s; }; } function main() { const a = [\"Pat\", \"Sam\"]; const f = save(a); console.log(a[0]); console.log(f()); const g = make(a); console.log(g()); }";
    execute(
        source,
        "Sam\nPat\nSam\n",
        Some(&[("make.a", Mode::Consume), ("save.a", Mode::Mutate)]),
    );
    let info = check(source).unwrap();
    assert_eq!(info.closures()[0].invocation, Mode::Mutate);
    assert_eq!(info.closures()[1].invocation, Mode::Consume);
    reject(
        "function save(a: string[]): () => string { const s = a.remove(0); return (): string => { return s; }; } function main() { const a = [\"Pat\"]; const f = save(a); console.log(f()); console.log(f()); }",
        "closure-consumed",
    );
    execute(
        "function main() { const a = [1, 2]; if (true) { const f = (): int => { return a.remove(0); }; console.log(f()); } console.log(a.remove(0)); }",
        "1\n2\n",
        None,
    );
}

#[test]
fn callback_and_recursive_access_contracts_include_removal() {
    execute(
        "function use(f: Mutable<Function<string, [a: Mutable<string[]>]>>, a: string[]): string { return f(a); } function step(a: string[], n: int): string { if (n > 0) { return step(a, n - 1); } return a.remove(0); } function main() { const a = [\"Pat\", \"Sam\"]; console.log(use((items: string[]): string => { return items.remove(0); }, a)); console.log(step(a, 3)); console.log(a.length); }",
        "Pat\nSam\n0\n",
        Some(&[
            ("use.f", Mode::Mutate),
            ("use.a", Mode::Mutate),
            ("step.a", Mode::Mutate),
            ("step.n", Mode::Copy),
        ]),
    );
    reject(
        "function use(f: Mutable<Function<int, [a: ReadOnly<int[]>]>>, a: int[]): int { return f(a); } function main() { const a = [1]; use((items: int[]): int => { return items.remove(0); }, a); }",
        "callback-access",
    );
    execute(
        "function retain(f: Function<int>): () => int { return f; } function one(f: Once<Function<string, [a: Mutable<string[]>]>>, a: string[]): string { return f(a); } function main() { const a = [1, 2]; const f = retain((): int => { return a.remove(0); }); console.log(f()); console.log(f()); const names = [\"Pat\"]; console.log(one((values: string[]): string => { return values.remove(0); }, names)); console.log(names.length); }",
        "1\n2\nPat\n0\n",
        Some(&[
            ("retain.f", Mode::Consume),
            ("one.f", Mode::Consume),
            ("one.a", Mode::Mutate),
        ]),
    );
    execute(
        "function keep(f: Once<Function<string>>): () => string { return f; } function main() { const a = [\"Pat\"]; const f = keep((): string => { return a.remove(0); }); console.log(f()); }",
        "Pat\n",
        Some(&[("keep.f", Mode::Consume)]),
    );
}

#[test]
fn branch_and_loop_removal_preserve_root_ownership() {
    execute(
        "function main() { const a = [\"Pat\", \"Sam\", \"Alex\"]; if (true) { console.log(a.remove(1)); } else { console.log(a.remove(0)); } while (a.length > 0) { const s = a.remove(0); console.log(s); } a.push(\"New\"); console.log(a[0]); const moved = a; console.log(moved.length); }",
        "Sam\nPat\nAlex\nNew\n1\n",
        None,
    );
    reject(
        "function main() { const a = [1]; if (true) { const moved = a; } console.log(a.remove(0)); }",
        "transferred",
    );
}

#[test]
fn removal_reports_precise_arity_type_and_transfer_diagnostics() {
    for (body, code) in [
        ("a.remove();", "arity"),
        ("a.remove(0, 1);", "arity"),
        ("a.remove(true);", "type"),
        ("a.remove(\"0\");", "type"),
        ("a.remove([0]);", "type"),
        ("const moved = a; a.remove(0);", "transferred"),
    ] {
        let source = format!("function main() {{ const a = [1]; {body} }}");
        let diagnostic = reject(&source, code);
        assert!(diagnostic.span.start >= source.find(body).unwrap());
    }
    for source in [
        "function main() { const a = 1; a.remove(0); }",
        "function main() { const a = \"Pat\"; a.remove(0); }",
        "type A = { n: int; }; function main() { const a: A = { n: 1 }; a.remove(0); }",
    ] {
        reject(source, "type");
    }
}

#[test]
fn bounds_and_failure_order_are_source_located_in_every_profile() {
    for (declaration, body, fragment, stdout) in [
        (
            "const a = [1];",
            "console.log(a.remove(-1));",
            "a.remove(-1)",
            "",
        ),
        ("const a = [1];", "a.remove(1);", "a.remove(1)", ""),
        ("const a: int[] = [];", "a.remove(0);", "a.remove(0)", ""),
        (
            "const a = [1];",
            "a.remove(mark(2));",
            "a.remove(mark(2))",
            "2\n",
        ),
        ("const a = [[1]];", "a[1].remove(mark(0));", "a[1]", ""),
        (
            "const a = [[1]];",
            "a[0].remove(mark(1));",
            "a[0].remove(mark(1))",
            "1\n",
        ),
        (
            "const a = [1];",
            "a.remove(empty(a));",
            "a.remove(empty(a))",
            "7\n",
        ),
        (
            "const a = [1];",
            "a.remove(9223372036854775807);",
            "a.remove(9223372036854775807)",
            "",
        ),
    ] {
        let source = format!(
            "function mark(n: int): int {{ console.log(n); return n; }} function empty(a: int[]): int {{ a.remove(0); console.log(7); return 0; }} function main() {{ {declaration} {body} }}"
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
fn deep_storage_and_array_type_guards_apply_to_removal() {
    let path = format!("node{}.values", ".children[0]".repeat(50));
    let source = format!(
        "type Node = {{ values: int[]; children: Node[]; }}; function main() {{ let node: Node = {{ values: [0], children: [] }}; let n = 0; while (n < 50) {{ node = {{ values: [n], children: [node] }}; n = n + 1; }} console.log({path}.remove(0)); console.log({path}.length); }}"
    );
    execute(&source, "0\n0\n", None);
    let source = format!(
        "type Node = {{ values: int[]; children: Node[]; }}; function main() {{ const node: Node = {{ values: [], children: [] }}; node{}.values.remove(0); }}",
        ".children[0]".repeat(100)
    );
    reject(&source, "unsupported");
    let rank = 100;
    let source = format!(
        "function main() {{ const a: int{} = {}0{}; const row = a.remove(0); console.log(row.length); console.log(a.length); }}",
        "[]".repeat(rank),
        "[".repeat(rank),
        "]".repeat(rank)
    );
    execute(&source, "1\n0\n", None);
    check(&format!(
        "function main() {{ const a: int{} = []; a.remove(0); }}",
        "[]".repeat(128)
    ))
    .unwrap();
    reject(
        &format!(
            "function main() {{ const a: int{} = []; a.remove(0); }}",
            "[]".repeat(129)
        ),
        "unsupported",
    );
}

#[test]
fn removal_metadata_is_deterministic_and_frontend_needs_no_backend() {
    let source = include_str!("fixtures/array-removal/queue.tsr");
    let first = check(source).unwrap();
    let second = check(source).unwrap();
    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.emit_rust(), second.emit_rust());
    let dir = support::TempDirectory::new();
    let path = dir.source(source);
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
}
