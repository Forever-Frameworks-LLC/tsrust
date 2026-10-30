mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn records_own_scalar_string_record_and_nested_array_fields() {
    let rust = execute(
        include_str!("fixtures/array-fields/catalog.tsr"),
        "Pat\nadmin\nSam\nnew\n7\nadmin\n",
        Some(&[("append.catalog", Mode::Mutate)]),
    );
    assert!(rust.contains("Array<String>"));
    assert!(rust.contains("RecordArray0"));
    assert!(rust.contains("Array<NumericArray<i64>>"));
    assert_eq!(rust.matches("Clone::clone(").count(), 1);
}

#[test]
fn owner_parameter_modes_include_field_array_access() {
    execute(
        "type Box = { items: int[]; }; function read(box: Box): int { return box.items[0]; } function append(a: int[]) { a.push(7); } function change(box: Box) { append(box.items); } function copy(box: Box): int[] { return clone(box.items); } function take(box: Box): Box { return box; } function main() { const box: Box = { items: [1] }; change(box); console.log(read(box)); const a = copy(box); console.log(a[1]); const moved = take(box); console.log(moved.items.length); }",
        "1\n7\n2\n",
        Some(&[
            ("append.a", Mode::Mutate),
            ("read.box", Mode::Read),
            ("change.box", Mode::Mutate),
            ("copy.box", Mode::Read),
            ("take.box", Mode::Consume),
        ]),
    );
    reject(
        "type Box = { items: int[]; }; function main() { const box: Box = { items: [1] }; const moved = box; console.log(box.items.length); }",
        "transferred",
    );
}

#[test]
fn construction_and_replacement_move_arrays_with_exact_field_context() {
    execute(
        "type Box = { items: int[][]; }; function make(): Box { return { items: [] }; } function main() { const box = make(); box.items.push([]); box.items[0].push(7); const rows = [[1, 2]]; box.items = rows; console.log(box.items[0][1]); }",
        "2\n",
        None,
    );
    for body in [
        "const a = [1]; const box: Box = { items: a }; console.log(a.length);",
        "const box: Box = { items: [] }; const a = [1]; box.items = a; console.log(a.length);",
    ] {
        reject(
            &format!("type Box = {{ items: int[]; }}; function main() {{ {body} }}"),
            "transferred",
        );
    }
    for source in [
        "type Box = { items: int[]; }; function main() { const box: Box = { items: [true] }; }",
        "type Box = { items: int[][]; }; function main() { const box: Box = { items: [1] }; }",
        "type U = { n: int; }; type V = { n: int; }; type Box = { items: U[]; }; function main() { const v: V = { n: 1 }; const box: Box = { items: [v] }; }",
    ] {
        reject(source, "type");
    }
}

#[test]
fn field_extraction_moves_and_explicit_cloning_remains_independent() {
    for body in ["const a = box.items;", "take(box.items);"] {
        execute(
            &format!(
                "type Box = {{ items: int[]; }}; function take(a: int[]): int[] {{ return a; }} function main() {{ const box: Box = {{ items: [1] }}; {body} }}"
            ),
            "",
            None,
        );
    }
    execute(
        "type Box = { items: int[]; }; function get(box: Box): int[] { return box.items; } function main() {}",
        "",
        None,
    );
    execute(
        "type Box = { items: int[]; }; function take(a: int[]): int[] { return a; } function main() { const box: Box = { items: [1] }; const a = take(clone(box.items)); a[0] = 7; console.log(box.items[0]); console.log(a[0]); const copy = clone(box); copy.items[0] = 9; console.log(box.items[0]); console.log(copy.items[0]); }",
        "1\n7\n1\n9\n",
        None,
    );
}

#[test]
fn field_iteration_holds_a_shared_loan_on_the_owner() {
    execute(
        "type Box = { items: int[][]; count: int; }; function main() { const box: Box = { items: [[1, 2]], count: 7 }; for (const row of box.items) { for (const n of row) { console.log(n + box.count); } } box.items[0].push(3); console.log(box.items[0].length); for (const row of clone(box.items)) { box.items.push(clone(row)); } console.log(box.items.length); }",
        "8\n9\n3\n2\n",
        None,
    );
    for body in [
        "box.items.push(2);",
        "box.items = [2];",
        "box.count = 2;",
        "const moved = box;",
        "const change = (): void => { box.count = 2; };",
    ] {
        reject(
            &format!(
                "type Box = {{ items: int[]; count: int; }}; function main() {{ let box: Box = {{ items: [1], count: 0 }}; for (const n of box.items) {{ {body} }} }}"
            ),
            "iteration-conflict",
        );
    }
}

#[test]
fn field_argument_loans_follow_static_field_paths() {
    execute(
        "type Box = { left: int[]; right: int[]; }; function pair(a: int[], b: int[]): int { return a[0] + b[0]; } function main() { const box: Box = { left: [1], right: [2] }; console.log(pair(box.left, box.right)); }",
        "3\n",
        None,
    );
    // Sibling fields are separate places; one field lent twice is not.
    execute(
        "type Box = { left: int[]; right: int[]; }; function pair(a: int[], b: int[]) { a.push(b[0]); } function main() { const box: Box = { left: [1], right: [2] }; pair(box.left, box.right); console.log(box.left.length); }",
        "2\n",
        None,
    );
    reject(
        "type Box = { left: int[]; right: int[]; }; function pair(a: int[], b: int[]) { a.push(b[0]); } function main() { const box: Box = { left: [1], right: [2] }; pair(box.left, box.left); }",
        "conflicting-access",
    );
    for body in [
        "console.log(box.items[grow(box)]);",
        "box.items[0] = grow(box);",
        "box.items.push(grow(box));",
    ] {
        reject(
            &format!(
                "type Box = {{ items: int[]; }}; function grow(box: Box): int {{ box.items.push(7); return 0; }} function main() {{ const box: Box = {{ items: [1] }}; {body} }}"
            ),
            "conflicting-access",
        );
    }
    execute(
        "type Box = { items: int[]; }; function read(box: Box): int { return box.items.length; } function main() { const box: Box = { items: [1] }; box.items[0] = read(box); box.items.push(box.items[0]); console.log(box.items[1]); }",
        "1\n",
        None,
    );
}

#[test]
fn captures_returns_and_callbacks_preserve_field_storage_access() {
    execute(
        "type Box = { items: int[]; }; function make(box: Box): (n: int) => int { return (n: int): int => { box.items.push(n); return box.items.length; }; } function use(f: Mutable<Function<void, [a: Mutable<int[]>]>>, box: Box) { f(box.items); } function main() { const box: Box = { items: [1] }; use((values: int[]): void => { values.push(2); }, box); const f = make(box); console.log(f(7)); console.log(f(8)); }",
        "3\n4\n",
        Some(&[
            ("make.box", Mode::Consume),
            ("use.box", Mode::Mutate),
            ("use.f", Mode::Mutate),
        ]),
    );
    reject(
        "type Box = { items: int[]; }; function main() { const box: Box = { items: [1] }; const read = (): int => { return box.items[0]; }; box.items.push(2); }",
        "capture-conflict",
    );
    execute(
        "type Box = { items: string[]; }; function keep(f: Once<Function<string>>): () => string { return f; } function main() { const box: Box = { items: [\"Pat\"] }; for (const item of box.items) { const saved = clone(item); const f = keep((): string => { return saved; }); console.log(f()); } }",
        "Pat\n",
        None,
    );
}

#[test]
fn mixed_storage_paths_evaluate_indices_once_and_replace_after_operands() {
    execute(
        "type Box = { items: int[]; }; function mark(n: int): int { console.log(n); return n; } function replacement(a: Box[]): int[] { a.push({ items: [9] }); return [7]; } function main() { const a: Box[] = [{ items: [1] }]; a[mark(0)].items[mark(0)] = mark(2); console.log(a[0].items[0]); a[mark(0)].items.push(mark(3)); a[mark(0)].items = replacement(a); console.log(a[0].items[0]); console.log(a.length); }",
        "0\n0\n2\n2\n0\n3\n0\n7\n2\n",
        None,
    );
    execute(
        "type Box = { items: int[]; count: int; }; function replacement(box: Box): int[] { box.count = 7; return [2]; } function main() { const box: Box = { items: [1], count: 0 }; box.items = replacement(box); console.log(box.items[0]); console.log(box.count); }",
        "2\n7\n",
        None,
    );
}

#[test]
fn nominal_array_fields_resolve_forward_and_recursive_record_names() {
    execute(
        "type Box = { users: User[]; }; type User = { tags: string[]; }; function main() { const box: Box = { users: [{ tags: [\"Pat\"] }] }; console.log(box.users[0].tags[0]); }",
        "Pat\n",
        None,
    );
    execute(
        "type A = { children: B[]; }; type B = { parents: A[]; }; function main() { const a: A = { children: [{ parents: [] }] }; console.log(a.children[0].parents.length); const b = clone(a); console.log(b.children.length); }",
        "0\n1\n",
        None,
    );
    execute(
        "type Node = { values: int[]; children: Node[]; }; function main() { let node: Node = { values: [0], children: [] }; let n = 0; while (n < 50) { node = { values: [n], children: [node] }; n = n + 1; } const copy = clone(node); copy.children[0].values[0] = 7; console.log(node.children[0].values[0]); console.log(copy.children[0].values[0]); }",
        "48\n7\n",
        None,
    );
}

#[test]
fn mixed_path_bounds_checks_and_failure_order_survive_optimization() {
    for (body, fragment, stdout) in [
        ("console.log(a[0].items[-1]);", "a[0].items[-1]", ""),
        ("a[1].items.push(mark(7));", "a[1]", ""),
        // Scalar assignment checks destination bounds before RHS evaluation.
        ("a[0].items[1] = mark(7);", "a[0].items[1]", ""),
        ("a[1].items = make();", "a[1]", "7\n"),
    ] {
        let source = format!(
            "type Box = {{ items: int[]; }}; function mark(n: int): int {{ console.log(n); return n; }} function make(): int[] {{ console.log(7); return []; }} function main() {{ const a: Box[] = [{{ items: [1] }}]; {body} }}"
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
fn deep_mixed_field_and_index_paths_have_bounded_native_compilation() {
    let path = format!("node{}values[0]", ".children[0]".repeat(50) + ".");
    let source = format!(
        "type Node = {{ values: int[]; children: Node[]; }}; function main() {{ let node: Node = {{ values: [0], children: [] }}; let n = 0; while (n < 50) {{ node = {{ values: [n], children: [node] }}; n = n + 1; }} console.log({path}); {path} = 7; console.log({path}); }}"
    );
    execute(&source, "0\n7\n", None);
    let source = format!(
        "type Node = {{ values: int[]; children: Node[]; }}; function main() {{ const node: Node = {{ values: [], children: [] }}; console.log(node{}.values.length); }}",
        ".children[0]".repeat(100)
    );
    support::accept_deep(&source);
    let source = format!(
        "type Box = {{ items: int{}; }}; function main() {{}}",
        "[]".repeat(129)
    );
    reject(&source, "unsupported");
}

#[test]
fn field_diagnostics_and_frontend_operation_need_no_backend() {
    for (source, code) in [
        (
            "type Box = { items: Missing[]; }; function main() {}",
            "array-element",
        ),
        (
            "type Box = { items: void[]; }; function main() {}",
            "array-element",
        ),
        (
            "type Box = { items: int[]; items: int[]; }; function main() {}",
            "duplicate",
        ),
        (
            "type U = { n: int; }; type Box = { value: void; }; function main() {}",
            "unsupported",
        ),
        (
            "type Box = { items: int[]; }; function main() { const box: Box = { items: [] }; box.items.length = 2; }",
            "unsupported",
        ),
        (
            "type Box = { items: int[]; }; function make(): Box { return { items: [] }; } function main() { make().items.length = 2; }",
            "unsupported",
        ),
    ] {
        reject(source, code);
    }
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/array-fields/catalog.tsr"));
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
}
