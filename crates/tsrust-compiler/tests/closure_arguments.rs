mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn scalar_arguments_work_directly_and_through_static_callbacks() {
    let rust = execute(
        include_str!("fixtures/callbacks/scalar_arguments.tsr"),
        "7\n0\n",
        Some(&[("apply.action", Mode::Mutate)]),
    );
    assert!(rust.contains("Callback<(i64, bool,), i64>"));
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
}

#[test]
fn closure_parameters_are_copy_bindings_and_never_captures() {
    let source = r#"function main() {
    const captured = 11;
    const choose = (value: int, enabled: boolean): int => {
        console.log(value); console.log(value); console.log(enabled);
        if (enabled) { return captured; } else { return value; }
    };
    console.log(choose(7, true)); console.log(choose(9, false));
    console.log(captured);
}"#;
    execute(source, "7\n7\ntrue\n11\n9\n9\nfalse\n9\n11\n", None);
    let analysis = check(source).unwrap();
    let closures = analysis.closures();
    assert_eq!(closures[0].invocation, Mode::Read);
    assert_eq!(closures[0].captures.len(), 1);
    assert_eq!(closures[0].captures[0].name, "captured");
    assert_eq!(closures[0].captures[0].storage, Mode::Copy);
}

#[test]
fn scalar_arguments_preserve_mutable_and_owned_environments() {
    execute(
        r#"type State = { value: int; };
function apply(action: Mutable<Function<int, [n: int]>>): int { return action(7); }
function main() {
    const state: State = { value: 0 };
    if (true) {
        const set = (next: int): int => { state.value = next; return state.value; };
        console.log(set(3)); console.log(apply(set));
    }
    console.log(state.value);
    console.log(apply((next: int): int => { state.value = next; return state.value; }));
    console.log(state.value);
}"#,
        "3\n7\n7\n7\n7\n",
        None,
    );
    execute(
        r#"type User = { name: string; };
function take(action: Once<Function<User, [flag: boolean, n: int]>>): User { return action(true, 7); }
function main() {
    const user: User = { name: "Pat" };
    const get = (enabled: boolean, count: int): User => { console.log(enabled); console.log(count); return user; };
    const saved = take(get); console.log(saved.name);
}"#,
        "true\n7\nPat\n",
        Some(&[("take.action", Mode::Consume)]),
    );
}

#[test]
fn typed_factory_results_preserve_signatures_and_invocation_modes() {
    execute(
        r#"type State = { value: int; };
function make(state: State): (next: int) => int {
    return (value: int): int => { state.value = value; return state.value; };
}
function forward(state: State): (ignoredName: int) => int { return make(state); }
function use(action: Mutable<Function<int, [input: int]>>) { console.log(action(9)); }
function main() {
    const action = forward({ value: 0 }); console.log(action(7)); use(action); console.log(action(11));
    use(make({ value: 1 }));
}"#,
        "7\n9\n11\n9\n",
        Some(&[
            ("make.state", Mode::Consume),
            ("forward.state", Mode::Consume),
            ("use.action", Mode::Mutate),
        ]),
    );
    execute(
        r#"function make(text: string): (flag: boolean) => string {
    return (enabled: boolean): string => { console.log(enabled); return text; };
}
function take(action: Once<Function<string, [value: boolean]>>): string { return action(true); }
function main() { console.log(take(make("Pat"))); }"#,
        "true\nPat\n",
        Some(&[("make.text", Mode::Consume), ("take.action", Mode::Consume)]),
    );
}

#[test]
fn forwarding_checks_ordered_types_but_not_signature_parameter_names() {
    execute(
        r#"function outer(action: Mutable<Function<int, [first: int, second: boolean]>>): int { return inner(action); }
function inner(action: Mutable<Function<int, [number: int, flag: boolean]>>): int { return action(7, false); }
function onceOuter(action: Once<Function<int, [number: int]>>): int { return onceInner(action); }
function onceInner(action: Once<Function<int, [value: int]>>): int { return action(9); }
function main() {
    console.log(outer((n: int, b: boolean): int => { console.log(b); return n; }));
    console.log(onceOuter((n: int): int => { return n; }));
}"#,
        "false\n7\n9\n",
        Some(&[
            ("outer.action", Mode::Mutate),
            ("inner.action", Mode::Mutate),
            ("onceOuter.action", Mode::Consume),
            ("onceInner.action", Mode::Consume),
        ]),
    );
    for (source, code) in [
        (
            "function use(f: (n: int) => int) {} function main() { use((b: boolean): int => { return 1; }); }",
            "type",
        ),
        (
            "function use(f: (n: int) => int) {} function main() { use((): int => { return 1; }); }",
            "arity",
        ),
        (
            "function use(f: (n: int, b: boolean) => int) {} function forward(f: (b: boolean, n: int) => int) { use(f); } function main() {}",
            "type",
        ),
        (
            "function make(): (n: int) => int { return (): int => { return 1; }; } function main() {}",
            "arity",
        ),
        (
            "function make(): (b: boolean) => int { return (n: int): int => { return n; }; } function main() {}",
            "type",
        ),
        (
            "function use(f: Once<Function<int, [n: int]>>) {} function forward(f: Once<Function<int, [b: boolean]>>) { use(f); } function main() {}",
            "type",
        ),
        (
            "function use(f: (n: int) => boolean) {} function main() { use((n: int): int => { return n; }); }",
            "type",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn receiver_access_is_live_during_argument_evaluation() {
    execute(
        "function main() { const identity = (n: int): int => { return n; }; console.log(identity(identity(7))); }",
        "7\n",
        None,
    );
    for (source, code) in [
        (
            "function bad(f: (n: int) => int): int { return f(f(7)); } function main() {}",
            "conflicting-access",
        ),
        (
            "function bad(f: Once<Function<int, [n: int]>>): int { return f(f(7)); } function main() {}",
            "callback-consumed",
        ),
        (
            "type S = { value: int; }; function main() { const state: S = { value: 0 }; const set = (n: int): int => { state.value = n; return n; }; set(set(7)); }",
            "conflicting-access",
        ),
        (
            "function take(f: Once<Function<int, [n: int]>>): int { return f(7); } function main() { const identity = (n: int): int => { return n; }; identity(take(identity)); }",
            "conflicting-access",
        ),
        (
            "function main() { const text = \"Pat\"; const get = (n: int): string => { console.log(n); return text; }; get(get(7)); }",
            "closure-consumed",
        ),
    ] {
        let error = reject(source, code);
        assert!(!error.related.is_empty());
    }
}

#[test]
fn scalar_arguments_evaluate_left_to_right_and_release_computed_loans() {
    execute(
        r#"type State = { value: int; };
function update(state: State, next: int): int { state.value = next; console.log(next); return next; }
function invoke(action: (a: int, b: int) => int, state: State): int { return action(update(state, 3), update(state, 5)); }
function main() {
    const state: State = { value: 1 };
    const choose = (first: int, second: int): int => { console.log(first); return second; };
    console.log(choose(state.value, update(state, 7)));
    console.log(invoke(choose, state)); console.log(state.value);
}"#,
        "7\n1\n7\n3\n5\n3\n5\n5\n",
        None,
    );
    reject(
        r#"type State = { value: int; }; function main() { const state: State = { value: 1 }; const set = (next: int): int => { state.value = next; return next; }; set(state.value); }"#,
        "capture-conflict",
    );
    reject(
        r#"type S = { value: int; }; function mutate(s: S): int { s.value = 2; return 2; } function use(s: S, n: int) { console.log(s.value); } function main() { const s: S = { value: 1 }; const id = (n: int): int => { return n; }; use(s, id(mutate(s))); }"#,
        "conflicting-access",
    );
}

#[test]
fn once_argument_calls_preserve_branch_ownership_and_reuse_errors() {
    execute(
        r#"function take(action: Once<Function<int, [n: int]>>, early: boolean): int {
    if (early) { return action(7); }
    return action(9);
}
function main() {
    console.log(take((n: int): int => { return n; }, true));
    console.log(take((n: int): int => { return n; }, false));
}"#,
        "7\n9\n",
        None,
    );
    reject(
        "function bad(f: Once<Function<int, [n: int]>>, flag: boolean) { if (flag) { f(7); } f(9); } function main() {}",
        "callback-consumed",
    );
    reject(
        "function take(f: Once<Function<int, [n: int]>>) { f(7); } function main() { const f = (n: int): int => { return n; }; take(f); f(9); }",
        "closure-consumed",
    );
    reject(
        "function use(f: (n: int) => string) {} function main() { const text = \"Pat\"; use((n: int): string => { console.log(n); return text; }); }",
        "callback-once",
    );
}

#[test]
fn parameter_scopes_grouping_and_declarations_are_checked() {
    execute(
        "function main() { const f = (n: int): int => { return ((n)); }; console.log((f)(7)); }",
        "7\n",
        None,
    );
    for source in [
        "function main() { const n = 7; const f = (n: int): int => { return n; }; }",
        "function main() { const f = (n: int, n: boolean): int => { return 0; }; }",
        "function use(f: (n: int, n: int) => int) {} function main() {}",
        "function make(): (n: int, n: int) => int { return (): int => { return 0; }; } function main() {}",
        "function main() { const f = (console: int): int => { return console; }; }",
        "function use(f: (clone: int) => int) {} function main() {}",
        "function main() { const f = (n: int): int => { const n = 9; return n; }; }",
    ] {
        reject(source, "duplicate");
    }
    reject(
        "function main() { const f = (n: int): int => { return n; }; console.log(n); }",
        "name",
    );
    reject(
        "function main() { const f = (n: int): int => { n = 2; return n; }; }",
        "unsupported",
    );
}

#[test]
fn argument_type_and_arity_errors_are_source_diagnostics() {
    for (source, code) in [
        (
            "function main() { const f = (n: int): int => { return n; }; f(); }",
            "arity",
        ),
        (
            "function main() { const f = (n: int): int => { return n; }; f(1, 2); }",
            "arity",
        ),
        (
            "function main() { const f = (n: int): int => { return n; }; f(true); }",
            "type",
        ),
        (
            "function use(f: (n: int) => int) { f(false); } function main() {}",
            "type",
        ),
        (
            "function use(f: Once<Function<int, [n: int]>>) { f(); } function main() {}",
            "arity",
        ),
        (
            "function main() { const f = (n: Missing): int => { return 1; }; }",
            "type",
        ),
        (
            "function main() { const f = (n: int): int => { if (false) { return n; } }; }",
            "type",
        ),
    ] {
        reject(source, code);
    }
    for ty in ["string", "void", "User"] {
        let prefix = "type User = { name: string; };";
        let direct =
            format!("{prefix} function main() {{ const f = (n: {ty}): int => {{ return 1; }}; }}");
        if ty == "void" {
            reject(&direct, "unsupported");
        } else {
            assert!(check(&direct).is_ok());
        }
        for source in [
            format!("{prefix} function use(f: (n: {ty}) => int) {{}} function main() {{}}"),
            format!("{prefix} function use(f: Once<Function<int, [n: {ty}]>>) {{}} function main() {{}}"),
            format!(
                "{prefix} function make(): (n: {ty}) => int {{ return (): int => {{ return 1; }}; }} function main() {{}}"
            ),
        ] {
            reject(
                &source,
                if ty == "void" {
                    "unsupported"
                } else {
                    "callback-access"
                },
            );
        }
    }
}

#[test]
fn typed_arguments_preserve_non_escape_and_syntax_boundaries() {
    execute(
        "function capture(f: Once<Function<int, [n: int]>>): Once<Function<int, [b: boolean]>> { return (b: boolean): int => { return f(7); }; } function main() { const g = capture((n: int): int => { return n; }); console.log(g(true)); }",
        "7\n",
        None,
    );
    assert!(check("function use(f: (g: () => int) => int) {} function main() {}").is_ok());
    execute(
        "function main() { const f = (n: int): int => { const g = (b: boolean): int => { return n; }; const alias = g; return n; }; }",
        "",
        None,
    );
    // Defaulted parameters and concise bodies are accepted arrow forms.
    execute(
        "function main() { const f = (n: int = 1): int => { return n; }; const g = (n: int): int => n; console.log(f(), g(2)); }",
        "1 2\n",
        None,
    );
    for (source, code) in [
        (
            "function invalid(f: Mutable<Function<int, [n: int]>>): (n: int) => int { return f; } function main() {}",
            "callback-escape",
        ),
        (
            "function use(f: (g: () =>) => int) {} function main() {}",
            "syntax",
        ),
        (
            "function main() { const f = (n: int,): int => { return n; }; }",
            "syntax",
        ),
        (
            "function main() { const f = (n): int => { return n; }; }",
            "parameter-inference",
        ),
        (
            "function use(f: (n: int => int) {} function main() {}",
            "syntax",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn argument_checks_and_rejections_need_no_backend() {
    let valid = include_str!("fixtures/callbacks/scalar_arguments.tsr");
    let invalid = "function main() { const f = (n: int): int => { return n; }; f(false); }";
    for (source, command, status) in [
        (valid, "check", 0),
        (valid, "emit", 0),
        (invalid, "check", 1),
        (invalid, "build", 1),
    ] {
        let dir = support::TempDirectory::new();
        let mut invocation = support::cli();
        invocation
            .arg(command)
            .arg(dir.source(source))
            .env("PATH", "");
        if command == "build" {
            invocation.arg("--output").arg(dir.0.join("program"));
        }
        let output = support::run(&mut invocation, 10);
        assert_eq!(output.status.code(), Some(status));
        if status == 1 {
            let error = String::from_utf8(output.stderr).unwrap();
            assert!(error.contains("type"), "{error}");
            assert!(!error.contains("rustup"));
        }
    }
}
