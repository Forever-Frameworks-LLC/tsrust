mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn recursive_owned_forwarding_reuses_native_specialization() {
    let source = "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } const next = c; return call(next, n - 1); } function main() { console.log(call((): int => { return 7; }, 3)); }";
    let rust = execute(
        source,
        "7\n",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
    assert!(!rust.contains("Box<"));
    assert!(!rust.contains("Clone::clone("));
    assert_eq!(rust.matches("fn function").count(), 1);
    let analysis = check(source).unwrap();
    assert_eq!(analysis.to_json(), check(source).unwrap().to_json());
    assert_eq!(analysis.emit_rust(), check(source).unwrap().emit_rust());
}

#[test]
fn owned_once_recursion_preserves_single_use_and_capture_transfer() {
    execute(
        "function call(c: Once<Function<string>>, n: int): string { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const name = \"Pat\"; console.log(call((): string => { return name; }, 3)); }",
        "Pat\n",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
    reject(
        "function call(c: Once<Function<int>>, n: int): int { if (n === 0) { return c(); } call(c, n - 1); return c(); } function main() {}",
        "callback-consumed",
    );
}

#[test]
fn mutable_owned_state_survives_recursive_moves_and_repeated_factory_types() {
    let rust = execute(
        include_str!("fixtures/recursive-owned-callbacks/countdown.tsr"),
        "10\n23\n",
        Some(&[
            ("counter.start", Mode::Copy),
            ("countdown.action", Mode::Consume),
            ("countdown.remaining", Mode::Copy),
        ]),
    );
    // One counter factory and one countdown specialization for both calls.
    assert_eq!(rust.matches("fn function").count(), 2);
    reject(
        "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const c = (): int => { return 7; }; call(c, 2); c(); }",
        "closure-consumed",
    );
}

#[test]
fn mutual_owned_recursion_preserves_contracts_and_declaration_order() {
    for kind in ["Function<int>", "Once<Function<int>>"] {
        let f = format!(
            "function f(c: {kind}, n: int): int {{ if (n === 0) {{ return c(); }} const next = c; return g(next, n - 1); }}"
        );
        let g = format!("function g(c: {kind}, n: int): int {{ return f(c, n); }}");
        for functions in [format!("{f}{g}"), format!("{g}{f}")] {
            execute(
                &format!(
                    "{functions} function main() {{ console.log(f((): int => {{ return 7; }}, 4)); }}"
                ),
                "7\n",
                Some(&[
                    ("f.c", Mode::Consume),
                    ("f.n", Mode::Copy),
                    ("g.c", Mode::Consume),
                    ("g.n", Mode::Copy),
                ]),
            );
        }
    }
}

#[test]
fn recursive_owned_parameters_can_feed_local_retained_wrappers() {
    for kind in ["Function<int>", "Once<Function<int>>"] {
        execute(
            &format!(
                "function call(c: {kind}, n: int): int {{ if (n === 0) {{ const invoke = (): int => {{ return c(); }}; return invoke(); }} return call(c, n - 1); }} function main() {{ console.log(call((): int => {{ return 7; }}, 3)); }}"
            ),
            "7\n",
            Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
        );
    }
    execute(
        "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } const invoke = (): int => { return call(c, n - 1); }; return invoke(); } function main() { console.log(call((): int => { return 7; }, 3)); }",
        "7\n",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
}

#[test]
fn growing_recursive_wrapper_types_are_rejected_before_native_compilation() {
    for kind in ["Function<int>", "Once<Function<int>>"] {
        let source = format!(
            "function call(c: {kind}, n: int): int {{ if (n === 0) {{ return c(); }} return call((): int => {{ return c(); }}, n - 1); }} function main() {{ console.log(call((): int => {{ return 7; }}, 2)); }}"
        );
        let error = reject(&source, "unsupported");
        assert!(error.message.contains("128"));
        let source = format!(
            "function retain(c: {kind}): () => int {{ return (): int => {{ return c(); }}; }} function call(c: {kind}, n: int): int {{ if (n === 0) {{ return c(); }} return call(retain(c), n - 1); }} function main() {{ console.log(call((): int => {{ return 7; }}, 2)); }}"
        );
        assert!(
            reject(&source, "unsupported")
                .message
                .contains("specialization count")
        );
    }
}

#[test]
fn mixed_owned_and_borrowed_callbacks_preserve_independent_native_contracts() {
    execute(
        "function call(c: Function<int>, b: Mutable<Function<int, [n: int]>>, n: int): int { if (n === 0) { return c() + b(n); } return call(c, b, n - 1); } function main() { let seen = 0; if (true) { const b = (n: int): int => { seen = seen + 1; return n; }; console.log(call((): int => { return 7; }, b, 2)); console.log(b(1)); } console.log(seen); }",
        "7\n1\n2\n",
        Some(&[
            ("call.c", Mode::Consume),
            ("call.b", Mode::Mutate),
            ("call.n", Mode::Copy),
        ]),
    );
    execute(
        "function call(c: Once<Function<string>>, b: Once<Function<int>>, n: int): string { if (n === 0) { console.log(b()); return c(); } return call(c, b, n - 1); } function main() { const name = \"Pat\"; console.log(call((): string => { return name; }, (): int => { return 7; }, 2)); }",
        "7\nPat\n",
        Some(&[
            ("call.c", Mode::Consume),
            ("call.b", Mode::Consume),
            ("call.n", Mode::Copy),
        ]),
    );
}

#[test]
fn recursive_owned_callbacks_preserve_exact_owned_argument_access() {
    execute(
        "type A = { n: int; }; function change(c: Function<void, [item: Mutable<A>]>, a: A, n: int) { if (n === 0) { c(a); return; } change(c, a, n - 1); } function main() { const a: A = { n: 1 }; change((item: A): void => { item.n = 7; }, a, 2); console.log(a.n); }",
        "7\n",
        Some(&[
            ("change.c", Mode::Consume),
            ("change.a", Mode::Mutate),
            ("change.n", Mode::Copy),
        ]),
    );
    execute(
        "type A = { n: int; }; function choose(c: Function<A, [item: Owned<A>]>, a: A, b: A, n: int): A { if (n === 0) { return c(a); } return choose(c, b, a, n - 1); } function main() { const a: A = { n: 1 }; const b: A = { n: 7 }; const selected = choose((item: A): A => { return item; }, a, b, 1); console.log(selected.n); }",
        "7\n",
        Some(&[
            ("choose.c", Mode::Consume),
            ("choose.a", Mode::Consume),
            ("choose.b", Mode::Consume),
            ("choose.n", Mode::Copy),
        ]),
    );
    execute(
        "type A = { name: string; }; function read(c: Function<string, [item: ReadOnly<A>]>, a: A, n: int): string { if (n === 0) { return c(a); } return read(c, a, n - 1); } function main() { const a: A = { name: \"Pat\" }; const text = \"Sam\"; console.log(read((item: A): string => { console.log(text.trim()); return item.name.trim(); }, a, 2)); console.log(a.name); }",
        "Sam\nPat\nPat\n",
        Some(&[
            ("read.c", Mode::Consume),
            ("read.a", Mode::Read),
            ("read.n", Mode::Copy),
        ]),
    );
}

#[test]
fn recursive_retention_preserves_capture_conflicts_and_signature_errors() {
    reject(
        "type A = { n: int; }; function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const a: A = { n: 7 }; const c = (): int => { return a.n; }; call(c, 2); }",
        "callback-escape",
    );
    reject(
        "type A = { n: int; }; function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const a: A = { n: 7 }; const read = (): int => { return a.n; }; call((): int => { return a.n; }, 2); read(); }",
        "capture-conflict",
    );
    reject(
        "function call(c: Function<string>, n: int): string { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const text = \"Pat\"; call((): string => { return text; }, 2); }",
        "callback-once",
    );
    reject(
        "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function forward(c: Once<Function<int>>): int { return call(c, 2); } function main() {}",
        "callback-contract",
    );
    reject(
        "type A = { n: int; }; function call(c: Function<void, [item: ReadOnly<A>]>, a: A, n: int) { if (n === 0) { c(a); return; } call(c, a, n - 1); } function main() { const a: A = { n: 1 }; call((item: A): void => { item.n = 7; }, a, 2); }",
        "callback-access",
    );
}

#[test]
fn borrowed_iteration_and_loop_joins_remain_enforced_in_owned_cycles() {
    execute(
        "function visit(c: Function<void, [n: int]>, a: int[], n: int) { for (const item of a) { c(item); } if (n > 0) { visit(c, a, n - 1); } } function main() { const a: int[] = [1, 2]; visit((n: int): void => { console.log(n); }, a, 1); console.log(a.length); }",
        "1\n2\n1\n2\n2\n",
        Some(&[
            ("visit.c", Mode::Consume),
            ("visit.a", Mode::Read),
            ("visit.n", Mode::Copy),
        ]),
    );
    reject(
        "function call(c: Function<int>, n: int): int { while (n > 0) { call(c, n - 1); } return c(); } function main() {}",
        "loop-transfer",
    );
    reject(
        "function call(c: Once<Function<void, [n: int]>>, n: int) { for (const item of [1, 2]) { c(item); } if (n > 0) { call(c, n - 1); } } function main() {}",
        "loop-transfer",
    );
}

#[test]
fn finite_owned_type_permutations_and_fresh_factory_results_use_cached_functions() {
    let rust = execute(
        "function choose(a: Function<int>, b: Function<int>, n: int): int { if (n === 0) { return a(); } return choose(b, a, n - 1); } function main() { console.log(choose((): int => { return 1; }, (): int => { return 7; }, 3)); }",
        "7\n",
        Some(&[
            ("choose.a", Mode::Consume),
            ("choose.b", Mode::Consume),
            ("choose.n", Mode::Copy),
        ]),
    );
    assert_eq!(rust.matches("fn function").count(), 2);
    let rust = execute(
        "function make(n: int): () => int { return (): int => { return n; }; } function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(make(n), n - 1); } function main() { console.log(call(make(7), 3)); }",
        "1\n",
        Some(&[
            ("make.n", Mode::Copy),
            ("call.c", Mode::Consume),
            ("call.n", Mode::Copy),
        ]),
    );
    assert_eq!(rust.matches("fn function").count(), 2);
    for kind in ["Function<int>", "Once<Function<int>>"] {
        let source = format!(
            "function call(c: {kind}, n: int): int {{ if (n === 0) {{ return c(); }} return call((): int => {{ return n; }}, n - 1); }} function main() {{ console.log(call((): int => {{ return 7; }}, 3)); }}"
        );
        let rust = execute(
            &source,
            "1\n",
            Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
        );
        assert_eq!(rust.matches("fn function").count(), 2);
        assert_eq!(check(&source).unwrap().closures().len(), 3);
    }
    // A swap and rotation generate all 120 vectors without unfolding those
    // native contexts on the compiler's source-analysis stack.
    let parameters = (0..5)
        .map(|i| format!("c{i}: Function<int>"))
        .collect::<Vec<_>>()
        .join(", ");
    let declarations = (0..5)
        .map(|i| format!("const c{i} = (): int => {{ return {i}; }};"))
        .collect::<String>();
    let source = format!(
        "function choose({parameters}, n: int): int {{ if (n === 0) {{ return c0(); }} if (n === 1) {{ return choose(c1, c0, c2, c3, c4, n - 1); }} return choose(c1, c2, c3, c4, c0, n - 1); }} function main() {{ {declarations} console.log(choose(c0, c1, c2, c3, c4, 2)); }}"
    );
    let rust = execute(&source, "2\n", None);
    assert_eq!(rust.matches("fn function").count(), 120);
}

#[test]
fn recursive_owned_specialization_limits_remain_global_and_frontend_only() {
    let definitions = "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); }";
    let calls = (0..128)
        .map(|i| format!("call((): int => {{ return {i}; }}, 1);"))
        .collect::<String>();
    let rust = execute(
        &format!("{definitions} function main() {{ {calls} }}"),
        "",
        None,
    );
    assert_eq!(rust.matches("fn function").count(), 128);
    let source = format!(
        "{definitions} function main() {{ {calls} call((): int => {{ return 128; }}, 1); }}"
    );
    assert!(
        reject(&source, "unsupported")
            .message
            .contains("specialization count")
    );
    let dir = support::TempDirectory::new();
    let path = dir.source(&source);
    let result = support::run(
        support::cli()
            .arg("build")
            .arg(&path)
            .arg("-o")
            .arg(dir.0.join("program"))
            .env("PATH", ""),
        10,
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("specialization count")
    );
}

#[test]
fn recursive_owned_specializations_retain_borrowed_vector_limits() {
    let width = 7;
    let parameters = (0..width)
        .map(|i| format!("b{i}: Mutable<Function<int>>"))
        .collect::<Vec<_>>()
        .join(", ");
    let swap = (0..width)
        .map(|i| format!("b{}", if i < 2 { 1 - i } else { i }))
        .collect::<Vec<_>>()
        .join(", ");
    let rotate = (0..width)
        .map(|i| format!("b{}", (i + 1) % width))
        .collect::<Vec<_>>()
        .join(", ");
    let definitions = format!(
        "function make(n: int): () => int {{ return (): int => {{ return n; }}; }} function choose(c: Function<int>, {parameters}, n: int): int {{ if (n === 0) {{ return c() + b0(); }} if (n === 1) {{ return choose(c, {swap}, n - 1); }} return choose(c, {rotate}, n - 1); }}"
    );
    let arguments = (0..width)
        .map(|i| format!("b{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    for repeated in [true, false] {
        let declarations = (0..width)
            .map(|i| {
                if repeated {
                    format!("const b{i} = make({i});")
                } else {
                    format!("const b{i} = (): int => {{ return {i}; }};")
                }
            })
            .collect::<String>();
        let source = format!(
            "{definitions} function main() {{ {declarations} console.log(choose((): int => {{ return 7; }}, {arguments}, 2)); }}"
        );
        if repeated {
            execute(&source, "9\n", None);
        } else {
            let error = reject(&source, "unsupported");
            assert_eq!(
                error.message,
                "Recursive callback instantiations exceed the prototype limit of 128 per native target."
            );
            assert!(!error.related.is_empty());
        }
    }
}

#[test]
fn owned_recursive_templates_preserve_source_errors_guards_and_backend_independence() {
    reject(
        "function call(c: Once<Function<int>>, n: int): int { if (n === 0) { c(); return c(); } return call(c, n - 1); } function main() {}",
        "callback-consumed",
    );
    let source = "function call(c: Function<int>, n: int): () => int { return call(c, n - 1); } function main() {}";
    assert!(check(source).is_ok());
    let concrete = source.replace(
        "function main() {}",
        "function main() { call((): int => { return 7; }, 1); }",
    );
    let error = reject(&concrete, "unsupported");
    assert!(error.message.contains("environment"));
    assert!(!error.related.is_empty());
    let unused = execute(
        "function call(c: Function<int>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() {}",
        "",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
    assert_eq!(unused.matches("fn function").count(), 0);
    let source = include_str!("fixtures/recursive-owned-callbacks/countdown.tsr");
    let analysis = check(source).unwrap();
    assert_eq!(analysis.contracts().len(), 3);
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

#[test]
fn recursive_owned_cycles_preserve_compiler_source_depth_guards() {
    for length in [20, 200] {
        let mut source = String::new();
        for i in 0..length {
            source.push_str(&format!("function f{i}(c: Function<int>, n: int): int {{ if (n === 0) {{ return c(); }} return f{}(c, n - 1); }}", (i+1)%length));
        }
        source.push_str("function main() { console.log(f0((): int => { return 7; }, 40)); }");
        if length == 20 {
            execute(&source, "7\n", None);
        } else {
            reject(&source, "unsupported");
        }
    }
}
