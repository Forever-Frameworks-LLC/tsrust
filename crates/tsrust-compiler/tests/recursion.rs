mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn recursive_owned_tree_reads_and_mutation_use_inferred_access() {
    let rust = execute(
        include_str!("fixtures/recursion/tree.tsr"),
        "6\n9\n",
        Some(&[("total.node", Mode::Read), ("increment.node", Mode::Mutate)]),
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn copied_scalars_support_self_and_mutual_recursion() {
    execute(
        "function factorial(n: int): int { if (n === 0) { return 1; } return n * factorial(n - 1); } function odd(n: int): boolean { if (n === 0) { return false; } return even(n - 1); } function even(n: int): boolean { if (n === 0) { return true; } return odd(n - 1); } function main() { console.log(factorial(6)); console.log(even(10)); console.log(odd(9)); }",
        "720\ntrue\ntrue\n",
        Some(&[
            ("factorial.n", Mode::Copy),
            ("odd.n", Mode::Copy),
            ("even.n", Mode::Copy),
        ]),
    );
    execute(
        "function f(): int { return f(); } function main() {}",
        "",
        None,
    );
    execute(
        "function f() { f(); } function g() { h(); } function h() { g(); } function main() {}",
        "",
        None,
    );
}

#[test]
fn mutual_mutation_contracts_propagate_independent_of_declaration_order() {
    let f = "function f(a: A, n: int) { if (n === 0) { a.n = a.n + 1; return; } g(a, n - 1); }";
    let g = "function g(a: A, n: int) { f(a, n); }";
    for functions in [format!("{f}{g}"), format!("{g}{f}")] {
        execute(
            &format!(
                "type A = {{ n: int; }}; {functions} function main() {{ const a: A = {{ n: 1 }}; g(a, 4); console.log(a.n); }}"
            ),
            "2\n",
            Some(&[
                ("f.a", Mode::Mutate),
                ("f.n", Mode::Copy),
                ("g.a", Mode::Mutate),
                ("g.n", Mode::Copy),
            ]),
        );
    }
}

#[test]
fn recursive_owned_results_transfer_strings_records_and_nested_arrays() {
    execute(
        "type A = { n: int; }; function record(a: A, n: int): A { if (n === 0) { return a; } return record(a, n - 1); } function text(a: string, n: int): string { if (n === 0) { return a; } return text(a, n - 1); } function rows(a: int[][], n: int): int[][] { if (n === 0) { return a; } return rows(a, n - 1); } function main() { const a: A = { n: 7 }; const b = record(a, 4); console.log(b.n); const s = text(\"Pat\", 3); console.log(s); const r = rows([[1, 2]], 3); console.log(r[0][1]); }",
        "7\nPat\n2\n",
        Some(&[
            ("record.a", Mode::Consume),
            ("record.n", Mode::Copy),
            ("text.a", Mode::Consume),
            ("text.n", Mode::Copy),
            ("rows.a", Mode::Consume),
            ("rows.n", Mode::Copy),
        ]),
    );
}

#[test]
fn permuted_recursive_parameters_reach_a_multi_round_fixed_point() {
    for width in [4, 9] {
        let params = (0..width)
            .map(|i| format!("a{i}: A"))
            .collect::<Vec<_>>()
            .join(", ");
        let rotate = (0..width)
            .map(|i| format!("a{}", (i + 1) % width))
            .collect::<Vec<_>>()
            .join(", ");
        let literals = (0..width)
            .map(|i| format!("{{ n: {i} }}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "type A = {{ n: int; }}; function rotate({params}, n: int): A {{ if (n === 0) {{ return a0; }} return rotate({rotate}, n - 1); }} function main() {{ const a = rotate({literals}, {}); console.log(a.n); }}",
            width - 1
        );
        let expected = format!("{}\n", width - 1);
        let rust = execute(&source, &expected, None);
        let modes = check(&source).unwrap().contracts();
        for i in 0..width {
            assert_eq!(modes[&format!("rotate.a{i}")], Mode::Consume);
        }
        assert_eq!(modes["rotate.n"], Mode::Copy);
        assert!(!rust.contains("Clone::clone("));
    }
    let width = 9;
    let params = (0..width)
        .map(|i| format!("a{i}: A"))
        .collect::<Vec<_>>()
        .join(", ");
    let rotate = (0..width)
        .map(|i| format!("a{}", (i + 1) % width))
        .collect::<Vec<_>>()
        .join(", ");
    let declarations = (0..width)
        .map(|i| format!("const a{i}: A = {{ n: {i} }};"))
        .collect::<String>();
    let arguments = (0..width)
        .map(|i| format!("a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "type A = {{ n: int; }}; function change({params}, n: int) {{ if (n === 0) {{ a0.n = a0.n + 1; return; }} change({rotate}, n - 1); }} function main() {{ {declarations} change({arguments}, 8); console.log(a0.n); console.log(a8.n); }}"
    );
    execute(&source, "0\n9\n", None);
    let modes = check(&source).unwrap().contracts();
    for i in 0..width {
        assert_eq!(modes[&format!("change.a{i}")], Mode::Mutate);
    }
}

#[test]
fn unavailable_recursive_arguments_are_rejected_after_effect_inference() {
    for source in [
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } const b = take(a, n - 1); console.log(a.n); return b; } function main() {}",
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { const a: A = { n: 1 }; const b = take(a, 1); console.log(a.n); }",
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { let a: A = { n: 1 }; if (true) { const b = take(a, 1); } console.log(a.n); }",
    ] {
        reject(source, "transferred");
    }
}

#[test]
fn recursive_contracts_preserve_argument_and_parent_selection_loans() {
    execute(
        "type A = { n: int; }; function read(a: A, n: int): int { if (n === 0) { return a.n; } return read(a, n - 1); } function pair(a: A, b: A): int { return read(a, 2) + read(b, 2); } function main() { const a: A = { n: 7 }; console.log(pair(a, a)); console.log(a.n); }",
        "14\n7\n",
        Some(&[
            ("read.a", Mode::Read),
            ("read.n", Mode::Copy),
            ("pair.a", Mode::Read),
            ("pair.b", Mode::Read),
        ]),
    );
    reject(
        "type A = { n: int; }; function change(a: A, n: int): int { if (n === 0) { a.n = 7; return a.n; } return change(a, n - 1); } function pair(a: A, b: int) { a.n = b; } function main() { const a: A = { n: 1 }; pair(a, change(a, 2)); }",
        "conflicting-access",
    );
    reject(
        "type A = { n: int; }; type Box = { a: A; }; function change(b: Box, n: int): int { if (n === 0) { b.a.n = 7; return 9; } return change(b, n - 1); } function main() { const b: Box = { a: { n: 1 } }; b.a.n = change(b, 2); }",
        "conflicting-access",
    );
}

#[test]
fn recursive_effects_reach_local_and_returned_closure_captures() {
    let source = "type A = { n: int; }; function increment(a: A, n: int) { if (n === 0) { a.n = a.n + 1; return; } increment(a, n - 1); } function make(a: A): () => int { return (): int => { increment(a, 2); return a.n; }; } function main() { const a: A = { n: 1 }; const next = make(a); console.log(next()); console.log(next()); }";
    execute(
        source,
        "2\n3\n",
        Some(&[
            ("increment.a", Mode::Mutate),
            ("increment.n", Mode::Copy),
            ("make.a", Mode::Consume),
        ]),
    );
    let analysis = check(source).unwrap();
    assert_eq!(analysis.closures().len(), 1);
    assert_eq!(analysis.closures()[0].invocation, Mode::Mutate);
    assert_eq!(analysis.closures()[0].captures[0].access, Mode::Mutate);
    assert_eq!(analysis.to_json(), check(source).unwrap().to_json());
    reject(
        "type A = { n: int; }; function change(a: A, n: int) { if (n === 0) { a.n = 7; return; } change(a, n - 1); } function main() { const a: A = { n: 1 }; const read = (): int => { return a.n; }; change(a, 2); }",
        "capture-conflict",
    );
}

#[test]
fn speculative_capture_loans_do_not_reject_valid_owned_reinitialization() {
    execute(
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { let a: A = { n: 1 }; const once = (): A => { return take(a, 2); }; a = { n: 7 }; const b = once(); console.log(a.n); console.log(b.n); }",
        "7\n1\n",
        Some(&[("take.a", Mode::Consume), ("take.n", Mode::Copy)]),
    );
    // Provisional read summaries can create more borrowed lifetimes than the
    // final consuming environment. Only the stable layout must meet the guard.
    let width = 130;
    let params = (0..width)
        .map(|i| format!("a{i}: A"))
        .collect::<Vec<_>>()
        .join(", ");
    let rotate = (0..width)
        .map(|i| format!("a{}", (i + 1) % width))
        .collect::<Vec<_>>()
        .join(", ");
    let declarations = (0..width)
        .map(|i| format!("const a{i}: A = {{ n: {i} }};"))
        .collect::<String>();
    let args = (0..width)
        .map(|i| format!("a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "type A = {{ n: int; }}; function rotate({params}, n: int): A {{ if (n === 0) {{ return a0; }} return rotate({rotate}, n - 1); }} function main() {{ {declarations} const once = (): A => {{ return rotate({args}, 0); }}; const result = once(); console.log(result.n); }}"
    );
    execute(&source, "0\n", None);
    let closure = &check(&source).unwrap().closures()[0];
    assert_eq!(closure.invocation, Mode::Consume);
    assert_eq!(closure.captures.len(), width);
    assert!(closure.captures.iter().all(|c| c.storage == Mode::Consume));
    let read_source = source
        .replace("n: int): A", "n: int): int")
        .replace("return a0;", "return a0.n;")
        .replace("(): A =>", "(): int =>")
        .replace("console.log(result.n)", "console.log(result)");
    // Like rustc, closure environments have no lifetime-count limit.
    execute(&read_source, "0\n", None);
}

#[test]
fn callback_access_is_validated_only_after_recursive_contracts_stabilize() {
    execute(
        "type A = { n: int; }; function use(f: Mutable<Function<A, [a: Owned<A>]>>, a: A): A { return f(a); } function carry(a: A, n: int): A { if (n > 0) { return use((c: A): A => { return carry(c, n - 1); }, a); } return a; } function main() { const a = carry({ n: 7 }, 3); console.log(a.n); }",
        "7\n",
        Some(&[
            ("use.f", Mode::Mutate),
            ("use.a", Mode::Consume),
            ("carry.a", Mode::Consume),
            ("carry.n", Mode::Copy),
        ]),
    );
    reject(
        "type A = { n: int; }; function use(f: (a: ReadOnly<A>) => A, a: A): A { return f(a); } function carry(a: A, n: int): A { if (n === 0) { return a; } return carry(a, n - 1); } function main() { const a: A = { n: 7 }; const b = use((c: A): A => { return carry(c, 2); }, a); }",
        "callback-access",
    );
}

#[test]
fn recursive_calls_keep_loop_and_shared_iteration_ownership_rules() {
    execute(
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { let a: A = { n: 7 }; let i = 0; while (i < 3) { a = take(a, 2); i = i + 1; } console.log(a.n); }",
        "7\n",
        Some(&[("take.a", Mode::Consume), ("take.n", Mode::Copy)]),
    );
    reject(
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { const a: A = { n: 7 }; while (true) { const b = take(a, 2); } }",
        "loop-transfer",
    );
    reject(
        "type A = { n: int; }; function take(a: A, n: int): A { if (n === 0) { return a; } return take(a, n - 1); } function main() { const a: A[] = [{ n: 7 }]; for (const item of a) { const b = take(item, 2); } }",
        "borrowed-element",
    );
    reject(
        "function change(a: int[], n: int) { if (n === 0) { a.push(7); return; } change(a, n - 1); } function main() { const a = [1]; for (const n of a) { change(a, 2); } }",
        "iteration-conflict",
    );
}

#[test]
fn recursive_operands_execute_once_in_source_order() {
    execute(
        "type A = { n: int; }; function mark(n: int): int { console.log(n); return n; } function read(a: A, n: int): int { if (n === 0) { return a.n; } return read(a, mark(n - 1)); } function main() { const a: A = { n: 9 }; console.log(read(a, 3)); }",
        "2\n1\n0\n9\n",
        Some(&[
            ("mark.n", Mode::Copy),
            ("read.a", Mode::Read),
            ("read.n", Mode::Copy),
        ]),
    );
}

#[test]
fn recursive_runtime_failures_retain_source_ranges_under_optimization() {
    for (source, fragment, message) in [
        (
            "function product(a: int, n: int): int { if (n === 0) { return a * 2; } return product(a, n - 1); } function main() { console.log(product(9223372036854775807, 2)); }",
            "*",
            "integer multiplication overflow",
        ),
        (
            "function read(a: int[], n: int): int { if (n === 0) { return a[-1]; } return read(a, n - 1); } function main() { console.log(read([1], 2)); }",
            "a[-1]",
            "array index out of bounds",
        ),
    ] {
        let dir = support::TempDirectory::new();
        let path = dir.0.join("program.rs");
        std::fs::write(&path, check(source).unwrap().emit_rust()).unwrap();
        let start = source.find(fragment).unwrap();
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
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains(&format!("{message} at source bytes {start}..{end}"))
            );
        }
    }
}

#[test]
fn unsupported_recursive_environment_cycles_and_invalid_calls_have_source_diagnostics() {
    // Callback-argument origins now support self and mutual forwarding. These
    // nonterminating declarations are intentionally checked without invocation.
    for source in [
        "function f(c: Function<void>, n: int): () => void { return f(c, n - 1); } function main() {}",
        "function f(c: Function<void>, n: int): () => void { return g(c, n); } function g(c: Function<void>, n: int): () => void { return f(c, n); } function main() {}",
    ] {
        let checked = check(source).expect("Concrete callback argument origins support forwarding");
        assert_eq!(checked.contracts()["f.c"], Mode::Consume);
        if source.contains("function g") {
            assert_eq!(checked.contracts()["g.c"], Mode::Consume);
        }
    }
    let error = reject(
        "function f(n: int): () => int { return f(n - 1); } function main() {}",
        "unsupported",
    );
    assert!(error.message.contains("environment"));
    assert!(!error.related.is_empty());
    reject(
        "function f(n: int): int { if (n === 0) { return 1; } return f(); } function main() {}",
        "arity",
    );
    reject(
        "function f(n: int): int { if (n === 0) { return true; } return f(n - 1); } function main() {}",
        "type",
    );
    reject(
        "function f(n: int): int { if (n === 0) { return 1; } return f(true); } function main() {}",
        "type",
    );
    reject("function main() { main(); }", "unsupported");
}

#[test]
fn recursive_call_cycles_use_worklists_and_preserve_nested_literal_behavior() {
    let mut source = String::new();
    for i in 0..25 {
        source.push_str(&format!(
            "function f{i}(n: int): int {{ if (n === 0) {{ return 0; }} return f{}(n - 1); }}",
            (i + 1) % 25
        ));
    }
    source.push_str("function main() { console.log(f0(50)); }");
    execute(&source, "0\n", None);
    assert!(
        check(&source)
            .unwrap()
            .contracts()
            .values()
            .all(|m| *m == Mode::Copy)
    );
    let mut source = String::new();
    for i in 0..200 {
        source.push_str(&format!(
            "function f{i}(n: int): int {{ return f{}(n); }}",
            (i + 1) % 200
        ));
    }
    source.push_str("function main() {}");
    assert!(check(&source).is_ok());
    // Recursive inference clones its source tree. Exercise the deep literal
    // path through those passes as well as through parsing and final checking.
    let mut types = "type R0 = { n: int; };".to_owned();
    let mut literal = "{ n: 7 }".to_owned();
    for i in 1..=100 {
        types.push_str(&format!("type R{i} = {{ child: R{}; }};", i - 1));
        literal = format!("{{ child: {literal} }}");
    }
    let path = format!("b{}.n", ".child".repeat(100));
    let source = format!(
        "{types} function carry(n: int, depth: int): int {{ if (depth === 0) {{ return n; }} return carry(n, depth - 1); }} function main() {{ const b: R100 = {literal}; console.log(carry({path}, 2)); }}"
    );
    execute(
        &source,
        "7\n",
        Some(&[("carry.n", Mode::Copy), ("carry.depth", Mode::Copy)]),
    );
}

#[test]
fn recursive_library_and_cli_results_are_deterministic_without_backend_tools() {
    let source = include_str!("fixtures/recursion/tree.tsr");
    let analysis = check(source).unwrap();
    assert_eq!(analysis.to_json(), check(source).unwrap().to_json());
    assert_eq!(analysis.emit_rust(), check(source).unwrap().emit_rust());
    let dir = support::TempDirectory::new();
    let path = dir.source(source);
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
}
