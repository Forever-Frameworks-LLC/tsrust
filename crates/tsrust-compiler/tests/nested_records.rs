mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn nested_records_have_contextual_literals_and_independent_clone_storage() {
    let rust = execute(
        include_str!("fixtures/nested-records/profile.tsr"),
        "Pat\n7\nSam\n9\n",
        Some(&[("update.user", Mode::Mutate)]),
    );
    assert!(rust.contains("field0: Record1"));
    assert_eq!(rust.matches("Clone::clone(").count(), 1);
    assert!(!rust.contains("Box<"));
    assert!(!rust.contains("Rc<"));
}

#[test]
fn nested_field_context_requires_exact_nominal_types_and_complete_shapes() {
    for (body, code) in [
        ("const b: Box = { child: {} };", "type"),
        ("const b: Box = { child: { n: 1, n: 2 } };", "duplicate"),
        ("const b: Box = { child: { n: 1, other: 2 } };", "type"),
        ("const b: Box = { child: true };", "type"),
        (
            "const v: Other = { n: 1 }; const b: Box = { child: v };",
            "type",
        ),
        (
            "const b: Box = { child: { n: 1 } }; const v: Other = { n: 2 }; b.child = v;",
            "type",
        ),
        (
            "const b: Box = { child: { n: 1 } }; console.log(b.child.missing);",
            "type",
        ),
    ] {
        reject(
            &format!(
                "type Child = {{ n: int; }}; type Other = {{ n: int; }}; type Box = {{ child: Child; }}; function main() {{ {body} }}"
            ),
            code,
        );
    }
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function make(): Box { return { child: { n: 7 } }; } function take(b: Box): int { return b.child.n; } function main() { console.log(take(make())); }",
        "7\n",
        Some(&[("take.b", Mode::Read)]),
    );
}

#[test]
fn owned_children_move_into_fields_and_whole_owner_reinitialization_joins() {
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function take(b: Box): Box { return b; } function main() { let b: Box = { child: { n: 0 } }; let i = 0; while (i < 2) { const saved = take(b); if (i === 0) { b = { child: { n: 1 } }; } else { b = { child: { n: 2 } }; } console.log(saved.child.n); i = i + 1; } console.log(b.child.n); const c: Child = { n: 7 }; b.child = c; console.log(b.child.n); }",
        "0\n1\n2\n7\n",
        Some(&[("take.b", Mode::Consume)]),
    );
    for body in [
        "const c: Child = { n: 1 }; const b: Box = { child: c }; console.log(c.n);",
        "const b: Box = { child: { n: 1 } }; const c: Child = { n: 2 }; b.child = c; console.log(c.n);",
        "const b: Box = { child: { n: 1 } }; const moved = b; console.log(b.child.n);",
        "let b: Box = { child: { n: 1 } }; if (true) { const moved = b; } console.log(b.child.n);",
    ] {
        reject(
            &format!(
                "type Child = {{ n: int; }}; type Box = {{ child: Child; }}; function main() {{ {body} }}"
            ),
            "transferred",
        );
    }
}

#[test]
fn nested_record_arguments_borrow_and_owned_extraction_moves() {
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function read(c: Child): int { return c.n; } function change(c: Child) { c.n = 7; } function copy(b: Box): Child { return clone(b.child); } function main() { const b: Box = { child: { n: 1 } }; change(b.child); console.log(read(b.child)); const c = copy(b); c.n = 9; console.log(b.child.n); console.log(c.n); }",
        "7\n7\n9\n",
        Some(&[
            ("read.c", Mode::Read),
            ("change.c", Mode::Mutate),
            ("copy.b", Mode::Read),
        ]),
    );
    for body in ["const c = b.child;", "take(b.child);", "b.child = b.child;"] {
        execute(
            &format!(
                "type Child = {{ n: int; }}; type Box = {{ child: Child; }}; function take(c: Child): Child {{ return c; }} function main() {{ const b: Box = {{ child: {{ n: 1 }} }}; {body} }}"
            ),
            "",
            None,
        );
    }
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function get(b: Box): Child { return b.child; } function main() {}",
        "",
        None,
    );
}

#[test]
fn sibling_field_access_uses_static_field_path_loans() {
    execute(
        "type Child = { n: int; }; type Box = { left: Child; right: Child; }; function sum(a: Child, b: Child): int { return a.n + b.n; } function main() { const b: Box = { left: { n: 1 }, right: { n: 2 } }; console.log(sum(b.left, b.right)); b.left.n = b.right.n; console.log(b.left.n); }",
        "3\n2\n",
        Some(&[("sum.a", Mode::Read), ("sum.b", Mode::Read)]),
    );
    // Sibling fields are separate places; the same field or its record is not.
    execute(
        "type Child = { n: int; }; type Box = { left: Child; right: Child; }; function change(a: Child, b: Child) { a.n = b.n; } function main() { const b: Box = { left: { n: 1 }, right: { n: 2 } }; change(b.left, b.right); console.log(b.left.n); }",
        "2\n",
        Some(&[("change.a", Mode::Mutate), ("change.b", Mode::Read)]),
    );
    reject(
        "type Child = { n: int; }; type Box = { left: Child; right: Child; }; function change(a: Child, b: Child) { a.n = b.n; } function main() { const b: Box = { left: { n: 1 }, right: { n: 2 } }; change(b.left, b.left); }",
        "conflicting-access",
    );
    reject(
        "type Child = { n: int; }; type Box = { child: Child; }; function change(b: Box): int { b.child.n = 7; return 9; } function main() { const b: Box = { child: { n: 1 } }; b.child.n = change(b); }",
        "conflicting-access",
    );
    reject(
        "type Child = { n: int; }; type Box = { child: Child; }; function take(b: Box): int { const moved = b; return moved.child.n; } function main() { const b: Box = { child: { n: 1 } }; b.child.n = take(b); }",
        "conflicting-access",
    );
}

#[test]
fn replacement_and_mixed_path_indices_evaluate_once_in_source_order() {
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function mark(n: int): int { console.log(n); return n; } function replace(b: Box): Child { b.child.n = 2; return { n: 7 }; } function main() { const b: Box = { child: { n: 1 } }; b.child = replace(b); console.log(b.child.n); const a: Box[][] = [[{ child: { n: 1 } }]]; a[mark(0)][mark(0)].child.n = mark(9); console.log(a[0][0].child.n); a[mark(0)][mark(0)].child = { n: mark(7) }; console.log(a[0][0].child.n); }",
        "7\n0\n0\n9\n9\n0\n0\n7\n7\n",
        Some(&[("mark.n", Mode::Copy), ("replace.b", Mode::Mutate)]),
    );
}

#[test]
fn nested_parent_bounds_and_final_field_replacement_keep_failure_order() {
    for (body, fragment, stdout) in [
        ("a[1].child.n = mark(7);", "a[1]", ""),
        ("a[1].child = make();", "a[1]", "7\n"),
        (
            "a[0].child.values[1] = mark(7);",
            "a[0].child.values[1]",
            "",
        ),
        ("a[1].child.values.push(mark(7));", "a[1]", ""),
    ] {
        let source = format!(
            "type Child = {{ n: int; values: int[]; }}; type Box = {{ child: Child; }}; function mark(n: int): int {{ console.log(n); return n; }} function make(): Child {{ console.log(7); return {{ n: 7, values: [] }}; }} function main() {{ const a: Box[] = [{{ child: {{ n: 1, values: [1] }} }}]; {body} }}"
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
fn nested_field_captures_keep_lexical_loans_and_owned_environment_state() {
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function make(b: Box): () => int { return (): int => { b.child.n = b.child.n + 1; return b.child.n; }; } function main() { const b: Box = { child: { n: 1 } }; if (true) { const read = (): int => { return b.child.n; }; console.log(read()); } b.child.n = 2; const next = make(b); console.log(next()); console.log(next()); }",
        "1\n3\n4\n",
        Some(&[("make.b", Mode::Consume)]),
    );
    for body in ["b.child.n = 2;", "b.child = { n: 2 };", "const moved = b;"] {
        reject(
            &format!(
                "type Child = {{ n: int; }}; type Box = {{ child: Child; }}; function main() {{ const b: Box = {{ child: {{ n: 1 }} }}; const read = (): int => {{ return b.child.n; }}; {body} }}"
            ),
            "capture-conflict",
        );
    }
}

#[test]
fn callback_contracts_and_retained_owned_children_preserve_access() {
    execute(
        "type Child = { n: int; }; type Box = { child: Child; }; function use(f: Mutable<Function<void, [c: Mutable<Child>]>>, b: Box) { f(b.child); } function keep(f: Once<Function<Child>>): () => Child { return f; } function main() { const b: Box = { child: { n: 1 } }; use((c: Child): void => { c.n = 7; }, b); console.log(b.child.n); const saved = clone(b.child); const retained = keep((): Child => { return saved; }); const c = retained(); console.log(c.n); console.log(b.child.n); }",
        "7\n7\n7\n",
        Some(&[
            ("use.f", Mode::Mutate),
            ("use.b", Mode::Mutate),
            ("keep.f", Mode::Consume),
        ]),
    );
    reject(
        "type Child = { n: int; }; type Box = { child: Child; }; function use(f: Mutable<Function<void, [c: ReadOnly<Child>]>>, b: Box) { f(b.child); } function main() { const b: Box = { child: { n: 1 } }; use((c: Child): void => { c.n = 7; }, b); }",
        "callback-access",
    );
}

#[test]
fn iteration_of_nested_storage_borrows_the_whole_root_owner() {
    execute(
        "type Child = { values: int[]; }; type Box = { child: Child; }; function main() { const b: Box = { child: { values: [1, 2] } }; for (const n of b.child.values) { console.log(n); } b.child.values.push(7); const a: Box[] = [clone(b)]; for (const item of a) { for (const n of item.child.values) { console.log(n); } } }",
        "1\n2\n1\n2\n7\n",
        None,
    );
    for body in [
        "b.child.values.push(7);",
        "b.child = { values: [] };",
        "const moved = b;",
    ] {
        reject(
            &format!(
                "type Child = {{ values: int[]; }}; type Box = {{ child: Child; }}; function main() {{ const b: Box = {{ child: {{ values: [1] }} }}; for (const n of b.child.values) {{ {body} }} }}"
            ),
            "iteration-conflict",
        );
    }
    reject(
        "type Child = { n: int; }; type Box = { child: Child; }; function main() { const a: Box[] = [{ child: { n: 1 } }]; for (const b of a) { b.child.n = 7; } }",
        "borrowed-element",
    );
}

#[test]
fn direct_layout_cycles_receive_source_diagnostics_even_when_unused() {
    for source in [
        "type Node = { next: Node; }; function main() {}",
        "type A = { b: B; }; type B = { a: A; }; function main() {}",
        "type A = { b: B[]; }; type B = { c: C; }; type C = { b: B; }; function main() {}",
    ] {
        let error = reject(source, "recursive-layout");
        assert!(!error.related.is_empty());
        assert!(!source[error.span.start..error.span.end].contains("[]"));
    }
}

#[test]
fn array_indirection_breaks_layout_cycles_and_shared_type_graphs_are_valid() {
    execute(
        "type A = { b: B; }; type B = { children: A[]; n: int; }; type Pair = { left: A; right: A; }; function main() { let a: A = { b: { children: [], n: 1 } }; a = { b: { children: [a], n: 2 } }; const p: Pair = { left: a, right: { b: { children: [], n: 7 } } }; const saved = clone(p); p.left.b.children[0].b.n = 9; console.log(saved.left.b.children[0].b.n); console.log(p.left.b.children[0].b.n); console.log(saved.right.b.n); }",
        "1\n9\n7\n",
        None,
    );
    execute(
        "type Empty = {}; type Pair = { left: Empty; right: Empty; }; function main() { const p: Pair = { left: {}, right: {} }; const copy = clone(p); console.log(true); }",
        "true\n",
        None,
    );
    // Like rustc, inline records are limited only by Rust's maximum object size.
    let doubling = |depth: usize| {
        let mut source = "type R0 = { n: int; };".to_owned();
        for i in 1..=depth {
            source.push_str(&format!(
                "type R{i} = {{ left: R{}; right: R{}; }};",
                i - 1,
                i - 1
            ));
        }
        source + "function main() {}"
    };
    support::accept_deep(&doubling(28));
    for depth in [60, 61] {
        assert!(
            reject(&doubling(depth), "unsupported")
                .message
                .contains("isize::MAX")
        );
    }
}

#[test]
fn deep_record_literals_and_storage_paths_compile_with_existing_guards() {
    let mut types = "type R0 = { n: int; };".to_owned();
    let mut literal = "{ n: 1 }".to_owned();
    for i in 1..=100 {
        types.push_str(&format!("type R{i} = {{ child: R{}; }};", i - 1));
        literal = format!("{{ child: {literal} }}");
    }
    let path = format!("b{}.n", ".child".repeat(100));
    let saved = format!("saved{}.n", ".child".repeat(100));
    execute(
        &format!(
            "{types} function main() {{ const b: R100 = {literal}; const saved = clone(b); {path} = 7; console.log({saved}); console.log({path}); }}"
        ),
        "1\n7\n",
        None,
    );
    let mut boundary = types.clone();
    for i in 101..128 {
        boundary.push_str(&format!("type R{i} = {{ child: R{}; }};", i - 1));
    }
    execute(&format!("{boundary} function main() {{}}"), "", None);
    // Like rustc, record layout nesting has no depth limit.
    boundary.push_str("type R128 = { child: R127; }; function main() {}");
    support::accept_deep(&boundary);
    // Recursive arrays keep type layouts finite, so deeply nested literals compile.
    let mut value = "{ children: [] }".to_owned();
    for _ in 0..150 {
        value = format!("{{ children: [{value}] }}");
    }
    support::accept_deep(&format!(
        "type Node = {{ children: Node[]; }}; function main() {{ const n: Node = {value}; }}"
    ));
}

#[test]
fn nested_record_frontend_checks_need_no_backend_and_validate_annotations() {
    for (source, code) in [
        (
            "type Box = { child: Missing; }; function main() {}",
            "unsupported",
        ),
        (
            "type Child = { n: int; }; type Box = { child: Child; child: Child; }; function main() {}",
            "duplicate",
        ),
        (
            "type Box = { child: void; }; function main() {}",
            "unsupported",
        ),
        (
            "type Child = { n: int; }; type Box = { child: Child; }; function make(): Box { return { child: { n: 1 } }; } function main() { make().child.n = 2; }",
            "unsupported",
        ),
    ] {
        reject(source, code);
    }
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/nested-records/profile.tsr"));
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
}
