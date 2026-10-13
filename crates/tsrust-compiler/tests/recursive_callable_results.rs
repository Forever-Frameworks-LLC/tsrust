mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn recursive_factories_keep_mutable_environment_state_and_move_aliases() {
    let source = include_str!("fixtures/recursive-callable-results/counter.tsr");
    execute(
        source,
        "2\n3\n",
        Some(&[("counter.start", Mode::Copy), ("counter.depth", Mode::Copy)]),
    );
    let analysis = check(source).unwrap();
    assert_eq!(analysis.closures().len(), 1);
    assert_eq!(analysis.closures()[0].invocation, Mode::Mutate);
    assert!(analysis.closures()[0].escapes);
    assert!(!analysis.closures()[0].template);
}

#[test]
fn branch_and_declaration_order_do_not_change_mutual_callable_origins() {
    for forward_first in [true, false] {
        let f = if forward_first {
            "function f(n: int): () => int { if (n > 0) { return g(n - 1); } return (): int => { return 7; }; }"
        } else {
            "function f(n: int): () => int { if (n === 0) { return (): int => { return 7; }; } return g(n - 1); }"
        };
        let g = "function g(n: int): () => int { return f(n); }";
        for functions in [format!("{f}{g}"), format!("{g}{f}")] {
            execute(
                &format!(
                    "{functions} function main() {{ const action = g(4); console.log(action()); }}"
                ),
                "7\n",
                Some(&[("f.n", Mode::Copy), ("g.n", Mode::Copy)]),
            );
        }
    }
}

#[test]
fn lazy_recursive_factory_invocation_uses_definitive_callable_bodies() {
    let source = "function make(n: int): () => int { return (): int => { if (n === 0) { return 7; } const next = make(n - 1); return next(); }; } function main() { const action = make(3); console.log(action()); console.log(action()); }";
    let rust = execute(source, "7\n7\n", Some(&[("make.n", Mode::Copy)]));
    assert!(!rust.contains("result-shape"));
    assert_eq!(check(source).unwrap().closures().len(), 1);
}

#[test]
fn recursive_callable_results_own_data_and_preserve_consuming_invocation() {
    let source = "type A = { name: string; }; function make(a: A, n: int): () => string { if (n > 0) { return make(a, n - 1); } return (): string => { return a.name.trim(); }; } function main() { const a: A = { name: \"Pat\" }; const action = make(a, 3); console.log(action()); console.log(action()); }";
    execute(
        source,
        "Pat\nPat\n",
        Some(&[("make.a", Mode::Consume), ("make.n", Mode::Copy)]),
    );
    reject(
        &source.replace(
            "console.log(action()); console.log(action());",
            "console.log(a.name);",
        ),
        "transferred",
    );
    let source = "function make(text: string, n: int): () => string { if (n > 0) { return make(text, n - 1); } return (): string => { return text; }; } function main() { const action = make(\"Pat\", 3); console.log(action()); }";
    execute(
        source,
        "Pat\n",
        Some(&[("make.text", Mode::Consume), ("make.n", Mode::Copy)]),
    );
    reject(
        &source.replace("console.log(action());", "console.log(action()); action();"),
        "closure-consumed",
    );
}

#[test]
fn recursive_owned_parameter_returns_preserve_reusable_and_once_origins() {
    for kind in ["Function<int>", "Once<Function<int>>"] {
        execute(
            &format!(
                "function f(c: {kind}, n: int): () => int {{ if (n > 0) {{ return g(c, n - 1); }} return c; }} function g(c: {kind}, n: int): () => int {{ return f(c, n); }} function main() {{ const action = f((): int => {{ return 7; }}, 3); console.log(action()); }}"
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

#[test]
fn unresolved_origins_and_recursive_environment_storage_are_source_errors() {
    for source in [
        "function f(): () => int { return f(); } function main() {}",
        "function f(): () => int { return g(); } function g(): () => int { return f(); } function main() {}",
        "function f(n: int): () => int { const next = f(n - 1); return (): int => { return next(); }; } function main() {}",
    ] {
        assert!(
            reject(source, "unsupported")
                .message
                .contains("environment")
        );
    }
}

#[test]
fn callable_results_preserve_exact_scalar_and_owned_argument_contracts() {
    execute(
        "function make(seed: int, n: int): (x: int) => int { if (n > 0) { return make(seed, n - 1); } return (x: int): int => { return seed + x; }; } function main() { const action = make(7, 2); console.log(action(1)); console.log(action(2)); }",
        "8\n9\n",
        Some(&[("make.seed", Mode::Copy), ("make.n", Mode::Copy)]),
    );
    execute(
        "type A = { n: int; }; function make(n: int): (a: Mutable<A>) => void { if (n > 0) { return make(n - 1); } return (a: A): void => { a.n = 7; }; } function main() { const a: A = { n: 1 }; const action = make(3); action(a); console.log(a.n); }",
        "7\n",
        Some(&[("make.n", Mode::Copy)]),
    );
    execute(
        "type A = { n: int; }; function write(a: A) { a.n = a.n + 1; } function make(a: A, n: int): () => void { if (n > 0) { return make(a, n - 1); } return (): void => { write(a); console.log(a.n); }; } function main() { const a: A = { n: 7 }; const action = make(a, 2); action(); action(); }",
        "8\n9\n",
        Some(&[
            ("write.a", Mode::Mutate),
            ("make.a", Mode::Consume),
            ("make.n", Mode::Copy),
        ]),
    );
    execute(
        "type A = { n: int; }; function make(n: int): (a: Owned<A>) => A { if (n > 0) { return make(n - 1); } return (a: A): A => { return a; }; } function main() { const action = make(3); const a: A = action({ n: 7 }); console.log(a.n); }",
        "7\n",
        Some(&[("make.n", Mode::Copy)]),
    );
}

#[test]
fn nested_callable_result_annotations_keep_owned_capture_depths() {
    let source = "function make(seed: int, n: int): (x: int) => () => int { if (n > 0) { return make(seed, n - 1); } return (x: int): () => int => { return (): int => { return seed + x; }; }; } function main() { const factory = make(7, 3); const first = factory(1); const second = factory(2); console.log(first()); console.log(second()); }";
    execute(
        source,
        "8\n9\n",
        Some(&[("make.seed", Mode::Copy), ("make.n", Mode::Copy)]),
    );
    let closures = check(source).unwrap().closures();
    assert_eq!(closures.len(), 2);
    assert!(closures.iter().any(|c| c.depth == 2));
    for forward_first in [true, false] {
        let base = "return (): int => { return 7; };";
        let forward = "const factory = make(n - 1); const next = factory(); return next;";
        let body = if forward_first {
            format!("if (n > 0) {{ {forward} }} {base}")
        } else {
            format!("if (n === 0) {{ {base} }} {forward}")
        };
        let source = format!(
            "function make(n: int): () => () => int {{ return (): () => int => {{ {body} }}; }} function main() {{ const factory = make(3); const action = factory(); console.log(action()); }}"
        );
        execute(&source, "7\n", Some(&[("make.n", Mode::Copy)]));
        assert_eq!(check(&source).unwrap().closures().len(), 2);
    }
}

#[test]
fn recursive_owned_factories_return_retained_wrappers_and_preserve_once() {
    let definitions =
        "function counter(): () => int { let n = 0; return (): int => { n = n + 1; return n; }; }";
    for kind in ["Function<int>", "Once<Function<int>>"] {
        let source = format!(
            "{definitions} function keep(c: {kind}, n: int): () => int {{ if (n > 0) {{ return keep(c, n - 1); }} return (): int => {{ return c(); }}; }} function main() {{ const action = keep(counter(), 3); console.log(action()); }}"
        );
        execute(
            &source,
            "1\n",
            Some(&[("keep.c", Mode::Consume), ("keep.n", Mode::Copy)]),
        );
        if kind == "Function<int>" {
            execute(
                &source.replace(
                    "console.log(action());",
                    "console.log(action()); console.log(action());",
                ),
                "1\n2\n",
                None,
            );
        } else {
            reject(
                &source.replace("console.log(action());", "console.log(action()); action();"),
                "closure-consumed",
            );
        }
    }
}

#[test]
fn lazy_consuming_results_refresh_provisional_invocation_contracts() {
    let source = "function make(text: string, n: int): () => string { return (): string => { if (n === 0) { return text; } const next = make(text, n - 1); return next(); }; } function main() { const action = make(\"Pat\", 3); console.log(action()); }";
    execute(
        source,
        "Pat\n",
        Some(&[("make.text", Mode::Consume), ("make.n", Mode::Copy)]),
    );
    assert_eq!(
        check(source).unwrap().closures()[0].invocation,
        Mode::Consume
    );
    reject(
        &source.replace("return next();", "next(); return next();"),
        "closure-consumed",
    );
}

#[test]
fn recursive_factory_signature_and_origin_errors_remain_source_diagnostics() {
    reject(
        "function f(n: int): () => int { if (n > 0) { return f(n - 1); } return (): boolean => { return true; }; } function main() {}",
        "type",
    );
    reject(
        "type A = { n: int; }; function f(n: int): (a: ReadOnly<A>) => void { if (n > 0) { return f(n - 1); } return (a: A): void => { a.n = 7; }; } function main() {}",
        "callback-access",
    );
    reject(
        "function f(n: int): () => int { if (n > 0) { return f(n - 1); } if (n === 0) { return (): int => { return 7; }; } return (): int => { return 8; }; } function main() {}",
        "unsupported",
    );
    reject(
        "function f(c: Mutable<Function<int>>, n: int): () => int { if (n > 0) { return f(c, n - 1); } return (): int => { return c(); }; } function main() {}",
        "callback-escape",
    );
}

#[test]
fn recursive_factory_capture_transfers_preserve_caller_loans() {
    reject(
        "type A = { n: int; }; function make(a: A, n: int): () => int { if (n > 0) { return make(a, n - 1); } return (): int => { return a.n; }; } function main() { const a: A = { n: 7 }; const read = (): int => { return a.n; }; const action = make(a, 3); console.log(read()); }",
        "capture-conflict",
    );
    reject(
        "type A = { n: int; }; function make(a: A, n: int): () => int { const action = (): int => { return a.n; }; if (n > 0) { return make(a, n - 1); } return action; } function main() {}",
        "transferred",
    );
}

#[test]
fn recursive_result_cycles_use_dependency_worklists() {
    for length in [20, 200] {
        let mut source = "function f0(n: int): () => int { if (n === 0) { return (): int => { return 7; }; } return f1(n - 1); }".to_owned();
        for i in 1..length {
            source.push_str(&format!(
                "function f{i}(n: int): () => int {{ return f{}(n); }}",
                (i + 1) % length
            ));
        }
        source.push_str("function main() { const action = f0(2); console.log(action()); }");
        execute(&source, "7\n", None);
    }
}

#[test]
fn recursive_result_metadata_and_cli_are_definitive_and_backend_independent() {
    execute(
        "function make(n: int): () => int { const unused = (): () => int => { return make(n); }; let value = n; if (n > 0) { return make(n - 1); } return (): int => { return value; }; } function main() { const action = make(3); console.log(action()); }",
        "0\n",
        Some(&[("make.n", Mode::Copy)]),
    );
    let source = "function make(n: int): () => int { return (): int => { if (n === 0) { return 7; } const next = make(n - 1); return next(); }; } function main() { const action = make(3); console.log(action()); }";
    let analysis = check(source).unwrap();
    assert_eq!(analysis.to_json(), check(source).unwrap().to_json());
    assert_eq!(analysis.emit_rust(), check(source).unwrap().emit_rust());
    assert_eq!(analysis.closures().len(), 1);
    assert_eq!(analysis.closures()[0].id, 0);
    assert!(!analysis.to_json().contains("result-shape"));
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
fn ordinary_callbacks_remain_non_escaping_during_callable_factory_recursion() {
    execute(
        "function make(c: Mutable<Function<void>>, n: int): () => int { c(); if (n > 0) { return make(c, n - 1); } return (): int => { return 7; }; } function main() { let calls = 0; if (true) { const c = (): void => { calls = calls + 1; }; const action = make(c, 3); console.log(action()); } console.log(calls); }",
        "7\n4\n",
        Some(&[("make.c", Mode::Mutate), ("make.n", Mode::Copy)]),
    );
    execute(
        "function make(c: Once<Function<int>>, n: int): () => int { if (n > 0) { return make(c, n - 1); } const value = c(); return (): int => { return value; }; } function main() { const action = make((): int => { return 7; }, 3); console.log(action()); console.log(action()); }",
        "7\n7\n",
        Some(&[("make.c", Mode::Consume), ("make.n", Mode::Copy)]),
    );
}

#[test]
fn recursive_callable_specialization_limits_reject_growing_return_environments() {
    let definitions = "function keep(c: Function<int>, n: int): () => int { if (n > 0) { return keep(c, n - 1); } return (): int => { return c(); }; }";
    let calls = (0..128)
        .map(|i| format!("const action{i} = keep((): int => {{ return {i}; }}, 1);"))
        .collect::<String>();
    execute(
        &format!("{definitions} function main() {{ {calls} console.log(action127()); }}"),
        "127\n",
        None,
    );
    let error = reject(
        &format!(
            "{definitions} function main() {{ {calls} keep((): int => {{ return 128; }}, 1); }}"
        ),
        "unsupported",
    );
    assert!(error.message.contains("specialization count"));
    reject(
        "function keep(c: Function<int>, n: int): () => int { if (n > 0) { return keep((): int => { return c(); }, n - 1); } return (): int => { return c(); }; } function main() { const action = keep((): int => { return 7; }, 2); console.log(action()); }",
        "unsupported",
    );
}
