mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn field_moves_preserve_siblings_and_restore_whole_record_use() {
    let rust = execute(
        include_str!("fixtures/field-moves/profile.tsr"),
        "admin\n7\nNew\nPat\n",
        Some(&[("take.profile", Mode::Consume)]),
    );
    assert!(!rust.contains("Clone::clone"));
    assert!(!rust.contains("unsafe"));
    assert!(!rust.contains("Box<"));
}

#[test]
fn string_record_and_array_fields_move_with_exact_nominal_types() {
    execute(
        "type Child = { n: int; }; type Box = { name: string; child: Child; rows: int[][]; flag: boolean; }; function main() { const b: Box = { name: \"Pat\", child: { n: 7 }, rows: [[1]], flag: true }; const s = b.name; const c = b.child; const rows = b.rows; c.n = 8; rows[0].push(2); console.log(s); console.log(c.n); console.log(rows[0][1]); console.log(b.flag); b.name = \"Sam\"; b.child = c; b.rows = rows; const moved = b; console.log(moved.name); console.log(moved.child.n); }",
        "Pat\n8\n2\ntrue\nSam\n8\n",
        None,
    );
    reject(
        "type A = { n: int; }; type B = { n: int; }; type Box = { a: A; }; function main() { const box: Box = { a: { n: 1 } }; const b: B = box.a; }",
        "type",
    );
    reject(
        "type Box = { a: int[][]; }; function main() { const box: Box = { a: [[1]] }; const a: int[] = box.a; }",
        "type",
    );
    execute(
        "type Child = { n: int; }; type Box = { name: string; child: Child; values: string[]; }; function main() { const b: Box = { name: \"Pat\", child: { n: 7 }, values: [\"Sam\"] }; const rebuilt: Box = { name: b.name, child: b.child, values: b.values }; const children = [rebuilt.child]; console.log(rebuilt.name); console.log(children[0].n); console.log(rebuilt.values[0]); }",
        "Pat\n7\nSam\n",
        None,
    );
}

#[test]
fn scalar_fields_copy_without_creating_missing_storage() {
    execute(
        "type Box = { n: int; flag: boolean; name: string; }; function read(b: Box): int { const n = b.n; const flag = b.flag; return b.n; } function main() { const b: Box = { n: 7, flag: true, name: \"Pat\" }; const n = b.n; const flag = b.flag; console.log(read(b)); const moved = b; console.log(moved.n); console.log(moved.flag); }",
        "7\n7\ntrue\n",
        Some(&[("read.b", Mode::Read)]),
    );
}

#[test]
fn missing_fields_reject_repeat_ancestor_and_whole_record_access() {
    for use_site in [
        "console.log(b.name);",
        "const again = b.name;",
        "const moved = b;",
        "const copy = clone(b);",
        "read(b);",
        "b.name = b.name;",
    ] {
        let source = format!(
            "type Box = {{ name: string; n: int; }}; function read(b: Box): int {{ return b.n; }} function main() {{ const b: Box = {{ name: \"Pat\", n: 1 }}; const s = b.name; {use_site} }}"
        );
        let error = reject(&source, "partial-transfer");
        assert!(!error.related.is_empty());
        assert_eq!(
            &source[error.related[0].0.start..error.related[0].0.end],
            "b.name"
        );
    }
    reject(
        "type Child = { name: string; n: int; }; type Box = { child: Child; }; function main() { const b: Box = { child: { name: \"Pat\", n: 1 } }; const name = b.child.name; const child = b.child; }",
        "partial-transfer",
    );
    reject(
        "type Child = { name: string; }; type Box = { child: Child; }; function main() { const b: Box = { child: { name: \"Pat\" } }; const child = b.child; console.log(b.child.name); }",
        "partial-transfer",
    );
}

#[test]
fn restoring_a_parent_clears_descendants_but_missing_parents_block_child_writes() {
    execute(
        "type Child = { left: string; right: string; }; type Box = { child: Child; n: int; }; function main() { const b: Box = { child: { left: \"Pat\", right: \"Sam\" }, n: 7 }; const left = b.child.left; const right = b.child.right; console.log(b.n); b.child = { left: left, right: right }; const moved = b; console.log(moved.child.left); console.log(moved.child.right); }",
        "7\nPat\nSam\n",
        None,
    );
    reject(
        "type Child = { name: string; }; type Box = { child: Child; }; function main() { const b: Box = { child: { name: \"Pat\" } }; const child = b.child; b.child.name = \"Sam\"; }",
        "partial-transfer",
    );
    reject(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; const moved = b; b.name = \"Sam\"; }",
        "transferred",
    );
}

#[test]
fn assignment_checks_current_rhs_and_commits_restoration_after_evaluation() {
    execute(
        "type Box = { left: string; right: string; }; function take(s: string): string { return s; } function main() { const b: Box = { left: \"Pat\", right: \"Sam\" }; b.left = take(b.left); console.log(b.left); const s = b.left; b.left = b.right; console.log(b.left); b.right = s; const moved = b; console.log(moved.right); let c: Box = { left: \"A\", right: \"B\" }; const a = c.left; c = { left: a, right: \"C\" }; console.log(c.left); console.log(c.right); }",
        "Pat\nSam\nPat\nA\nC\n",
        Some(&[("take.s", Mode::Consume)]),
    );
    reject(
        "type Child = { name: string; }; type Box = { child: Child; }; function take(c: Child): string { return c.name; } function main() { const b: Box = { child: { name: \"Pat\" } }; b.child.name = take(b.child); }",
        "conflicting-access",
    );
    reject(
        "type Box = { name: string; }; function take(b: Box): string { return b.name; } function main() { const b: Box = { name: \"Pat\" }; b.name = take(b); }",
        "transferred",
    );
    reject(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; const s = b.name; b.name = 1; }",
        "type",
    );
}

#[test]
fn branch_joins_union_missing_fields_and_exclude_exited_paths() {
    execute(
        "type Box = { name: string; n: int; }; function use(b: Box, flag: boolean): int { if (flag) { const s = b.name; return b.n; } console.log(b.name); return b.n; } function main() { const b: Box = { name: \"Pat\", n: 7 }; if (true) { const s = b.name; b.name = s; } else { const s = b.name; b.name = s; } console.log(b.name); console.log(use(b, true)); }",
        "Pat\n7\n",
        Some(&[("use.b", Mode::Consume), ("use.flag", Mode::Copy)]),
    );
    for body in [
        "if (true) { const s = b.left; } console.log(b.left);",
        "if (true) { const s = b.left; } else { const s = b.right; } const moved = b;",
        "const s = b.left; if (true) { b.left = s; } console.log(b.left);",
    ] {
        reject(
            &format!(
                "type Box = {{ left: string; right: string; }}; function main() {{ const b: Box = {{ left: \"Pat\", right: \"Sam\" }}; {body} }}"
            ),
            "partial-transfer",
        );
    }
    reject(
        "type Box = { name: string; }; function take(s: string): boolean { const moved = s; return true; } function main() { const b: Box = { name: \"Pat\" }; const flag = false && take(b.name); console.log(b.name); }",
        "partial-transfer",
    );
}

#[test]
fn loops_require_field_restoration_on_each_continuing_edge() {
    execute(
        "type Box = { name: string; n: int; }; function main() { const b: Box = { name: \"Pat\", n: 0 }; while (b.n < 2) { const s = b.name; console.log(s); b.name = s; b.n = b.n + 1; continue; } console.log(b.name); while (true) { const s = b.name; console.log(s); break; } b.name = \"Sam\"; console.log(b.name); }",
        "Pat\nPat\nPat\nPat\nSam\n",
        None,
    );
    for body in [
        "while (true) { const s = b.name; }",
        "while (true) { const s = b.name; continue; }",
        "for (const n of [1]) { const s = b.name; }",
    ] {
        reject(
            &format!(
                "type Box = {{ name: string; }}; function main() {{ const b: Box = {{ name: \"Pat\" }}; {body} }}"
            ),
            "loop-transfer",
        );
    }
    reject(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; while (true) { const s = b.name; break; } console.log(b.name); }",
        "partial-transfer",
    );
    reject(
        "type Box = { left: string; right: string; }; function main() { const b: Box = { left: \"Pat\", right: \"Sam\" }; const left = b.left; while (true) { const right = b.right; } }",
        "loop-transfer",
    );
    execute(
        "type Box = { name: string; n: int; }; function main() { const b: Box = { name: \"Pat\", n: 0 }; const name = b.name; while (b.n < 2) { b.name = \"Sam\"; const removed = b.name; console.log(removed); b.n = b.n + 1; } b.name = name; console.log(b.name); }",
        "Sam\nSam\nPat\n",
        None,
    );
}

#[test]
fn captures_keep_whole_owner_storage_and_consuming_invocation() {
    let source = "type Box = { name: string; n: int; }; function make(b: Box): () => string { return (): string => { const s = b.name; console.log(b.n); return s; }; } function main() { const b: Box = { name: \"Pat\", n: 7 }; const f = make(b); console.log(f()); }";
    execute(source, "7\nPat\n", Some(&[("make.b", Mode::Consume)]));
    assert_eq!(
        check(source).unwrap().closures()[0].invocation,
        Mode::Consume
    );
    reject(
        "type Box = { name: string; n: int; }; function main() { const b: Box = { name: \"Pat\", n: 7 }; const s = b.name; const f = (): int => { return b.n; }; }",
        "partial-transfer",
    );
    execute(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; const name = b.name; const f = (): string => { return name; }; console.log(f()); b.name = \"Sam\"; console.log(b.name); }",
        "Pat\nSam\n",
        None,
    );
    reject(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; const f = (): string => { return b.name; }; console.log(b.name); }",
        "transferred",
    );
    reject(
        "type Box = { name: string; }; function main() { const b: Box = { name: \"Pat\" }; const f = (): string => { return b.name; }; console.log(f()); console.log(f()); }",
        "closure-consumed",
    );
}

#[test]
fn callable_owned_argument_access_matches_partial_field_consumption() {
    execute(
        "type Box = { name: string; }; function use(f: Mutable<Function<string, [b: Owned<Box>]>>, b: Box): string { return f(b); } function one(f: Once<Function<string, [b: Owned<Box>]>>, b: Box): string { return f(b); } function main() { const a: Box = { name: \"Pat\" }; const b: Box = { name: \"Sam\" }; console.log(use((box: Box): string => { return box.name; }, a)); console.log(one((box: Box): string => { return box.name; }, b)); }",
        "Pat\nSam\n",
        Some(&[
            ("use.f", Mode::Mutate),
            ("use.b", Mode::Consume),
            ("one.f", Mode::Consume),
            ("one.b", Mode::Consume),
        ]),
    );
    reject(
        "type Box = { name: string; }; function use(f: Mutable<Function<string, [b: ReadOnly<Box>]>>, b: Box): string { return f(b); } function main() { const b: Box = { name: \"Pat\" }; use((box: Box): string => { return box.name; }, b); }",
        "callback-access",
    );
    execute(
        "type Box = { name: string; }; function keep(f: Once<Function<string>>): () => string { return f; } function main() { const b: Box = { name: \"Pat\" }; const f = keep((): string => { return b.name; }); console.log(f()); }",
        "Pat\n",
        Some(&[("keep.f", Mode::Consume)]),
    );
    execute(
        "type Box = { name: string; }; function keep(f: Function<string, [b: Owned<Box>]>): (b: Owned<Box>) => string { return f; } function main() { const f = keep((b: Box): string => { return b.name; }); const a: Box = { name: \"Pat\" }; const b: Box = { name: \"Sam\" }; console.log(f(a)); console.log(f(b)); }",
        "Pat\nSam\n",
        Some(&[("keep.f", Mode::Consume)]),
    );
}

#[test]
fn recursive_effects_propagate_field_consumption_before_definitive_checking() {
    execute(
        "type Box = { name: string; n: int; }; function left(b: Box, n: int): string { if (n > 0) { return right(b, n - 1); } return b.name; } function right(b: Box, n: int): string { return left(b, n); } function main() { const b: Box = { name: \"Pat\", n: 7 }; console.log(left(b, 3)); }",
        "Pat\n",
        Some(&[
            ("left.b", Mode::Consume),
            ("left.n", Mode::Copy),
            ("right.b", Mode::Consume),
            ("right.n", Mode::Copy),
        ]),
    );
    reject(
        "type Box = { name: string; }; function take(b: Box, n: int): string { if (n > 0) { return take(b, n - 1); } return b.name; } function main() { const b: Box = { name: \"Pat\" }; take(b, 1); console.log(b.name); }",
        "transferred",
    );
    execute(
        "type Box = { name: string; }; function go(f: Once<Function<string, [b: Owned<Box>]>>, b: Box, n: int): string { if (n > 0) { return go(f, b, n - 1); } return f(b); } function main() { const b: Box = { name: \"Pat\" }; console.log(go((box: Box): string => { return box.name; }, b, 3)); }",
        "Pat\n",
        Some(&[
            ("go.f", Mode::Consume),
            ("go.b", Mode::Consume),
            ("go.n", Mode::Copy),
        ]),
    );
}

#[test]
fn recursive_factories_can_return_environments_owning_detached_fields() {
    execute(
        "type Box = { name: string; n: int; }; function make(b: Box, n: int): () => string { if (n > 0) { return make(b, n - 1); } const name = b.name; console.log(b.n); return (): string => { return name; }; } function main() { const b: Box = { name: \"Pat\", n: 7 }; const f = make(b, 3); console.log(f()); }",
        "7\nPat\n",
        Some(&[("make.b", Mode::Consume), ("make.n", Mode::Copy)]),
    );
}

#[test]
fn detached_argument_values_release_root_loans_and_borrows_remain_conservative() {
    execute(
        "type Box = { left: string; right: string; n: int; }; function pair(a: string, b: string): string { console.log(b); return a; } function main() { const box: Box = { left: \"Pat\", right: \"Sam\", n: 7 }; console.log(pair(box.left, box.right)); console.log(box.n); console.log(box.right); }",
        "Sam\nPat\n7\nSam\n",
        Some(&[("pair.a", Mode::Consume), ("pair.b", Mode::Read)]),
    );
    execute(
        "type Box = { left: string; right: string; }; function pair(a: string, b: string) { const first = a; const second = b; console.log(first); console.log(second); } function main() { const box: Box = { left: \"Pat\", right: \"Sam\" }; pair(box.left, box.right); box.left = \"A\"; box.right = \"B\"; const whole = box; console.log(whole.left); }",
        "Pat\nSam\nA\n",
        Some(&[("pair.a", Mode::Consume), ("pair.b", Mode::Consume)]),
    );
    reject(
        "type Box = { left: string; right: string; }; function pair(a: string, b: string) { console.log(a); const s = b; } function main() { const box: Box = { left: \"Pat\", right: \"Sam\" }; pair(box.left, box.right); }",
        "conflicting-access",
    );
    reject(
        "type Box = { left: string; }; function pair(s: string, b: Box) { const a = s; const copy = clone(b); } function main() { const b: Box = { left: \"Pat\" }; pair(b.left, b); }",
        "partial-transfer",
    );
}

#[test]
fn indexed_and_iteration_borrows_cannot_transfer_owned_fields() {
    for body in [
        "take(a[0].name);",
    ] {
        reject(
            &format!(
                "type Box = {{ name: string; tags: int[]; }}; function take(s: string): string {{ return s; }} function main() {{ const a: Box[] = [{{ name: \"Pat\", tags: [1] }}]; {body} }}"
            ),
            "borrowed-element",
        );
    }
    // A const taken from a loop element borrows it; a transfer is still rejected.
    execute(
        "type Box = { name: string; }; function main() { const a: Box[] = [{ name: \"Pat\" }]; for (const b of a) { const s = b.name; console.log(s, s.length); } console.log(a.length); }",
        "Pat 3\n1\n",
        None,
    );
    reject(
        "type Box = { name: string; }; function take(s: string): string { return s; } function main() { const a: Box[] = [{ name: \"Pat\" }]; for (const b of a) { const s = b.name; take(s); } }",
        "borrowed-place",
    );
    execute(
        "type Box = { name: string; n: int; }; function main() { const a: Box[] = [{ name: \"Pat\", n: 7 }]; const b = a.remove(0); const name = b.name; console.log(name); console.log(b.n); console.log(a.length); b.name = \"Sam\"; a.push(b); console.log(a[0].name); }",
        "Pat\n7\n0\nSam\n",
        None,
    );
}

#[test]
fn active_iteration_and_capture_loans_protect_whole_owner() {
    reject(
        "type Box = { name: string; values: int[]; }; function main() { const b: Box = { name: \"Pat\", values: [1] }; for (const n of b.values) { const name = b.name; } }",
        "iteration-conflict",
    );
    reject(
        "type Box = { name: string; n: int; }; function main() { const b: Box = { name: \"Pat\", n: 7 }; const read = (): int => { return b.n; }; const name = b.name; }",
        "capture-conflict",
    );
    execute(
        "type Box = { name: string; values: int[]; }; function main() { const b: Box = { name: \"Pat\", values: [1] }; const name = b.name; for (const n of b.values) { console.log(n); } console.log(name); b.values.push(2); console.log(b.values[1]); }",
        "1\nPat\n2\n",
        None,
    );
}

#[test]
fn mixed_paths_borrow_only_complete_selected_fields_and_keep_source_bounds() {
    execute(
        "type Inner = { name: string; values: int[][]; }; type Box = { inner: Inner; }; function main() { const b: Box = { inner: { name: \"Pat\", values: [[1]] } }; const name = b.inner.name; b.inner.values[0].push(2); console.log(b.inner.values[0][1]); console.log(b.inner.values[0].remove(0)); b.inner.name = name; const whole = b; console.log(whole.inner.name); }",
        "2\n1\nPat\n",
        None,
    );
    let source = "type Box = { name: string; values: int[]; }; function mark(): int { console.log(7); return 1; } function main() { const b: Box = { name: \"Pat\", values: [1] }; const name = b.name; b.values[mark()] = 2; }";
    let dir = support::TempDirectory::new();
    let path = dir.0.join("program.rs");
    std::fs::write(&path, check(source).unwrap().emit_rust()).unwrap();
    let start = source.find("b.values[mark()]").unwrap();
    let end = start + "b.values[mark()]".len();
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
        let result = support::run(&mut std::process::Command::new(&binary), 10);
        assert!(!result.status.success());
        assert_eq!(result.stdout, b"7\n");
        assert!(String::from_utf8(result.stderr).unwrap().contains(&format!(
            "array index out of bounds at source bytes {start}..{end}"
        )));
    }
}

#[test]
fn deep_partial_records_have_native_drop_and_reinitialization_support() {
    let mut types = "type R0 = { left: string; right: string; };".to_owned();
    let mut value = "{ left: \"Pat\", right: \"Sam\" }".to_owned();
    for i in 1..=100 {
        types.push_str(&format!("type R{i} = {{ child: R{}; }};", i - 1));
        value = format!("{{ child: {value} }}");
    }
    let path = format!("b{}", ".child".repeat(100));
    execute(
        &format!(
            "{types} function main() {{ const b: R100 = {value}; const left = {path}.left; console.log({path}.right); {path}.left = left; const whole = b; console.log(whole{}.left); }}",
            ".child".repeat(100)
        ),
        "Sam\nPat\n",
        None,
    );
}

#[test]
fn partial_field_metadata_is_deterministic_and_frontend_only() {
    let source = include_str!("fixtures/field-moves/profile.tsr");
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
