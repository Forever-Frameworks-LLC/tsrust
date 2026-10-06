mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn owned_returns_move_the_original_environment_and_keep_mutable_state() {
    let source = include_str!("fixtures/callback-returns/identity.tsr");
    let rust = execute(source, "1\n2\n1\n", None);
    assert_eq!(
        check(source).unwrap().contracts()["identity.action"],
        Mode::Consume
    );
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
    reject(
        "function identity(action: Function<int>): () => int { return action; } function main() { const f = (): int => { return 7; }; const next = identity(f); f(); }",
        "closure-consumed",
    );
}

#[test]
fn once_returns_keep_the_declared_contract_for_all_actual_invocations() {
    for body in ["return text;", "return text.trim();"] {
        let source = format!(
            "function identity(action: Once<Function<string>>): () => string {{ return action; }} function main() {{ const text = \"Pat\"; const next = identity((): string => {{ {body} }}); console.log(next()); }}"
        );
        execute(&source, "Pat\n", None);
        reject(
            &source.replace("console.log(next());", "next(); next();"),
            "closure-consumed",
        );
        reject(
            &source.replace("console.log(next());", "console.log(text);"),
            "transferred",
        );
    }
    execute(
        "function identity(action: Once<Function<int>>): () => int { return action; } function main() { let n = 0; const f = identity((): int => { n = n + 1; return n; }); console.log(f()); }",
        "1\n",
        None,
    );
    let source = "function identity(action: Once<Function<int>>): () => int { return action; } function use(action: () => int) {} function main() { const f = identity((): int => { return 7; }); use(f); }";
    reject(source, "callback-once");
}

#[test]
fn returned_arguments_match_read_mutate_consume_and_scalar_contracts() {
    execute(
        "type U = { name: string; }; function identity(action: Function<string, [u: Mutable<U>, n: int]>): (u: Mutable<U>, n: int) => string { return action; } function main() { const f = identity((u: U, n: int): string => { u.name = \"Sam\"; console.log(n); return u.name.trim(); }); const user: U = { name: \"Pat\" }; console.log(f(user, 7)); console.log(user.name); }",
        "7\nSam\nSam\n",
        None,
    );
    execute(
        "function identity(action: Once<Function<string, [a: ReadOnly<string>, b: Owned<string>]>>): (a: ReadOnly<string>, b: Owned<string>) => string { return action; } function main() { const f = identity((a: string, b: string): string => { console.log(a); return b; }); const a = \"Pat\"; console.log(f(a, \"Sam\")); console.log(a); }",
        "Pat\nSam\nPat\n",
        None,
    );
    execute(
        "type U = { name: string; }; function identity(action: Once<Function<void, [u: Mutable<U>]>>): (u: Mutable<U>) => void { return action; } function main() { const f = identity((u: U): void => { u.name = \"Sam\"; }); const u: U = { name: \"Pat\" }; f(u); console.log(u.name); }",
        "Sam\n",
        None,
    );
    for (source, code) in [
        (
            "function bad(action: Function<string, [s: ReadOnly<string>]>): (s: Owned<string>) => string { return action; } function main() {}",
            "callback-access",
        ),
        (
            "function bad(action: Function<int>): () => boolean { return action; } function main() {}",
            "type",
        ),
        (
            "function bad(action: Function<int, [n: int]>): () => int { return action; } function main() {}",
            "arity",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn direct_returns_follow_move_chains_branches_and_loop_exits() {
    for contract in ["Function<int>", "Once<Function<int>>"] {
        let source = format!(
            "function identity(action: {contract}, flag: boolean): () => int {{ const next = action; if (flag) {{ const last = next; return last; }} return next; }} function main() {{ const a = identity((): int => {{ return 7; }}, true); const b = identity((): int => {{ return 8; }}, false); console.log(a()); console.log(b()); }}"
        );
        execute(&source, "7\n8\n", None);
        execute(
            &format!(
                "function identity(action: {contract}, flag: boolean): () => int {{ while (flag) {{ return action; }} return action; }} function main() {{ const f = identity((): int => {{ return 7; }}, true); console.log(f()); }}"
            ),
            "7\n",
            None,
        );
    }
    reject(
        "function bad(action: Once<Function<int>>): () => int { action(); return action; } function main() {}",
        "callback-consumed",
    );
    reject(
        "function bad(action: Function<int>, flag: boolean): () => int { if (flag) { const next = action; } return action; } function main() {}",
        "callback-consumed",
    );
    reject(
        "function bad(a: Function<int>, b: Function<int>, flag: boolean): () => int { if (flag) { return a; } return b; } function main() {}",
        "unsupported",
    );
}

#[test]
fn forwarding_templates_substitute_returned_parameter_origins() {
    execute(
        "function identity(action: Function<int>): () => int { return action; } function use(action: () => int): int { return action(); } function forward(action: Function<int>): int { const next = identity(action); return use(next); } function main() { console.log(forward((): int => { return 7; })); }",
        "7\n",
        None,
    );
    execute(
        "function identity(action: Once<Function<int>>): () => int { return action; } function use(action: Once<Function<int>>): int { return action(); } function forward(action: Once<Function<int>>): int { const next = identity(action); return use(next); } function main() { console.log(forward((): int => { return 7; })); }",
        "7\n",
        None,
    );
    for contract in ["Function<int>", "Once<Function<int>>"] {
        let source = format!(
            "function identity(action: {contract}): () => int {{ return action; }} function forward(action: {contract}, flag: boolean): () => int {{ if (flag) {{ return identity(action); }} return action; }} function main() {{ const f = forward((): int => {{ return 7; }}, true); console.log(f()); }}"
        );
        execute(&source, "7\n", None);
    }
    execute(
        "function identity(action: Function<int>): () => int { return action; } function forward(action: Function<int>): () => int { const next = identity(action); return identity(next); } function main() { const f = forward((): int => { return 7; }); console.log(f()); console.log(f()); }",
        "7\n7\n",
        None,
    );
    // A concrete factory result used while checking a template must not create
    // a template-only restriction wrapper that later specializations reuse.
    execute(
        "function make(): () => int { return (): int => { return 7; }; } function identity(action: Once<Function<int>>): () => int { return action; } function forward(unused: Function<int>): () => int { return identity(make()); } function main() { const f = forward(make()); console.log(f()); }",
        "7\n",
        None,
    );
}

#[test]
fn arrow_factories_can_return_captured_owned_callbacks_directly() {
    for contract in ["Function<int>", "Once<Function<int>>"] {
        execute(
            &format!(
                "function make(action: {contract}): () => () => int {{ return (): () => int => {{ const next = action; return next; }}; }} function main() {{ const outer = make((): int => {{ return 7; }}); const inner = outer(); console.log(inner()); }}"
            ),
            "7\n",
            None,
        );
        reject(
            &format!(
                "function make(action: {contract}): () => () => int {{ return (): () => int => {{ return action; }}; }} function main() {{ const outer = make((): int => {{ return 7; }}); const first = outer(); const second = outer(); }}"
            ),
            "closure-consumed",
        );
        execute(
            &format!(
                "function make(action: {contract}): () => int {{ const forward = (): () => int => {{ return action; }}; return forward(); }} function main() {{ const f = make((): int => {{ return 7; }}); console.log(f()); }}"
            ),
            "7\n",
            None,
        );
    }
}

#[test]
fn returned_once_values_keep_restrictions_through_moves_captures_and_replacement() {
    let prefix = "function identity(action: Once<Function<int>>): () => int { return action; } function make(): () => int { return identity((): int => { return 7; }); }";
    execute(
        &format!(
            "{prefix} function main() {{ let f = make(); const next = f; console.log(next()); f = make(); console.log(f()); }}"
        ),
        "7\n7\n",
        None,
    );
    reject(
        &format!(
            "{prefix} function main() {{ const f = make(); const next = f; next(); next(); }}"
        ),
        "closure-consumed",
    );
    reject(
        &format!(
            "{prefix} function main() {{ const f = make(); const wrapper = (): int => {{ return f(); }}; wrapper(); wrapper(); }}"
        ),
        "closure-consumed",
    );
    reject(
        &format!("{prefix} function main() {{ const f = make(); while (true) {{ f(); }} }}"),
        "loop-transfer",
    );
}

#[test]
fn non_owned_returns_borrows_and_invalid_template_bodies_are_rejected() {
    for contract in ["ReadOnly<Function<int>>", "Mutable<Function<int>>"] {
        reject(
            &format!(
                "function bad(action: {contract}): () => int {{ const next = action; return next; }} function main() {{}}"
            ),
            "callback-escape",
        );
    }
    reject(
        "function identity(action: Function<string>): () => string { return action; } function main() { const text = \"Pat\"; const f = (): string => { return text.trim(); }; identity(f); }",
        "callback-escape",
    );
    reject(
        "function bad(action: Function<int>): () => int { const reader = (): int => { return action(); }; return action; } function main() {}",
        "capture-conflict",
    );
    reject(
        "function bad(action: Function<int>): () => int {} function main() {}",
        "type",
    );
    execute(
        "function unused(action: Once<Function<int>>): () => int { return action; } function main() {}",
        "",
        None,
    );
}

#[test]
fn once_wrapper_cache_is_deterministic_and_preserves_void_returns() {
    let source = "function make(): () => void { return (): void => { console.log(7); }; } function identity(action: Once<Function<void>>): () => void { return action; } function forward(action: Once<Function<void>>): () => void { return action; } function main() { const a = identity(make()); const b = forward(make()); a(); b(); }";
    execute(source, "7\n7\n", None);
    let analysis = check(source).unwrap();
    let rust = analysis.emit_rust();
    assert_eq!(rust, check(source).unwrap().emit_rust());
    assert_eq!(
        analysis
            .closures()
            .iter()
            .filter(|c| !c.template && c.invocation == Mode::Consume)
            .count(),
        1
    );
    assert_eq!(rust.matches("struct Closure").count(), 2);
}

#[test]
fn direct_return_analysis_and_emission_are_backend_independent() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/callback-returns/identity.tsr"));
    for command in ["check", "emit"] {
        let output = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&output);
    }
}
