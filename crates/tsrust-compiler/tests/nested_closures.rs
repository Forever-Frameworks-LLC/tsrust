mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn local_nested_mutation_propagates_to_outer_argument_contracts() {
    let source = include_str!("fixtures/callbacks/nested_local.tsr");
    let rust = execute(source, "Sam\nSam\nSam\n", None);
    let closures = check(source).unwrap().closures();
    let outer = closures.iter().find(|c| c.depth == 1).unwrap();
    let inner = closures.iter().find(|c| c.depth == 2).unwrap();
    assert_eq!(outer.invocation, Mode::Read);
    assert_eq!(outer.parameters[0].access, Mode::Mutate);
    assert_eq!(inner.invocation, Mode::Mutate);
    assert_eq!(inner.captures[0].name, "user");
    assert_eq!(inner.captures[0].storage, Mode::Mutate);
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
    execute(
        r#"type U = { name: string; };
function apply(action: Mutable<Function<string, [user: ReadOnly<U>, count: int]>>): string { return action({ name: "Pat" }, 7); }
function main() {
    const outer = (prefix: string): string => {
        const inner = (input: U, n: int): string => { console.log(prefix); console.log(n); return input.name.trim(); };
        return apply(inner);
    };
    console.log(outer("first")); console.log(outer("second"));
}"#,
        "first\n7\nPat\nsecond\n7\nPat\n",
        Some(&[("apply.action", Mode::Mutate)]),
    );
}

#[test]
fn ancestor_captures_propagate_through_multiple_levels() {
    let source = r#"type State = { value: int; };
function use(state: State, copied: int) {
    if (true) {
        const outer = (): void => {
            const middle = (): void => {
                const inner = (): void => { state.value = copied; };
                inner();
            };
            middle();
        };
        outer(); outer();
    }
    console.log(state.value);
}
function main() { const state: State = { value: 0 }; use(state, 7); console.log(state.value); }"#;
    execute(
        source,
        "7\n7\n",
        Some(&[("use.state", Mode::Mutate), ("use.copied", Mode::Copy)]),
    );
    let closures = check(source).unwrap().closures();
    assert_eq!(closures.len(), 3);
    for closure in closures {
        assert_eq!(closure.invocation, Mode::Mutate);
        assert_eq!(
            closure
                .captures
                .iter()
                .find(|c| c.name == "state")
                .unwrap()
                .access,
            Mode::Mutate
        );
        assert_eq!(
            closure
                .captures
                .iter()
                .find(|c| c.name == "copied")
                .unwrap()
                .storage,
            Mode::Copy
        );
    }
}

#[test]
fn returned_outer_environments_reborrow_captures_on_each_invocation() {
    let source = r#"type State = { on: boolean; };
function make(): () => boolean {
    const state: State = { on: false };
    return (): boolean => {
        const toggle = (): boolean => {
            if (state.on) { state.on = false; } else { state.on = true; }
            return state.on;
        };
        return toggle();
    };
}
function main() { const outer = make(); console.log(outer()); console.log(outer()); console.log(outer()); }"#;
    execute(source, "true\nfalse\ntrue\n", None);
    let closures = check(source).unwrap().closures();
    let outer = closures.iter().find(|c| c.depth == 1).unwrap();
    let inner = closures.iter().find(|c| c.depth == 2).unwrap();
    assert!(outer.escapes);
    assert!(!inner.escapes);
    assert_eq!(outer.captures[0].storage, Mode::Consume);
    assert_eq!(inner.captures[0].storage, Mode::Mutate);
}

#[test]
fn consuming_ancestor_capture_makes_outer_invocation_single_use() {
    let source = r#"function make(text: string): () => string {
    return (): string => {
        const middle = (): string => {
            const inner = (): string => { return text; };
            return inner();
        };
        return middle();
    };
}
function main() { const outer = make("Pat"); console.log(outer()); }"#;
    execute(source, "Pat\n", Some(&[("make.text", Mode::Consume)]));
    assert!(
        check(source)
            .unwrap()
            .closures()
            .iter()
            .all(|c| c.invocation == Mode::Consume)
    );
    reject(
        &source.replace(
            "console.log(outer());",
            "console.log(outer()); console.log(outer());",
        ),
        "closure-consumed",
    );
    reject(
        &source.replace("return inner();", "const saved = inner(); return inner();"),
        "closure-consumed",
    );
}

#[test]
fn nested_consumption_of_fresh_arguments_keeps_outer_reusable() {
    execute(
        r#"type U = { name: string; };
function apply(action: Mutable<Function<U, [user: Owned<U>]>>, user: U): U { return action(user); }
function main() {
    const outer = (input: U): U => {
        const take = (): U => { return input; };
        return take();
    };
    const first = apply(outer, { name: "Pat" }); console.log(first.name);
    const second = apply(outer, { name: "Sam" }); console.log(second.name);
}"#,
        "Pat\nSam\n",
        Some(&[
            ("apply.action", Mode::Mutate),
            ("apply.user", Mode::Consume),
        ]),
    );
    reject(
        "function main() { const outer = (text: string): void => { const take = (): string => { return text; }; console.log(text); }; }",
        "transferred",
    );
}

#[test]
fn nested_local_callbacks_support_reuse_once_and_inline_loans() {
    execute(
        r#"type U = { name: string; };
function twice(action: Mutable<Function<void>>) { action(); action(); }
function take(action: Once<Function<string>>): string { return action(); }
function main() {
    const outer = (user: U, text: string): string => {
        twice((): void => { user.name = "Sam"; });
        console.log(user.name);
        const show = (): void => { console.log(user.name); };
        twice(show);
        const get = (): string => { return text; };
        return take(get);
    };
    const user: U = { name: "Pat" };
    console.log(outer(user, "first")); console.log(outer(user, "second"));
}"#,
        "Sam\nSam\nSam\nfirst\nSam\nSam\nSam\nsecond\n",
        None,
    );
    reject(
        "function twice(f: Mutable<Function<void>>) { f(); f(); } function main() { const outer = (text: string): void => { const inner = (): void => { const saved = text; }; twice(inner); }; }",
        "callback-once",
    );
}

#[test]
fn factories_inside_closures_apply_capture_or_fresh_argument_effects() {
    execute(
        r#"function make(text: string): () => string { return (): string => { return text; }; }
function read(action: Once<Function<string>>): string { return action(); }
function main() {
    const outer = (input: string): string => { const local = make(input); return read(local); };
    console.log(outer("Pat")); console.log(outer("Sam"));
    const text = "owned";
    const once = (): string => { return read(make(text)); };
    console.log(once());
}"#,
        "Pat\nSam\nowned\n",
        None,
    );
    execute(
        r#"type U = { name: string; };
function make(user: U): () => string { const copy = clone(user); return (): string => { return copy.name.trim(); }; }
function main() {
    const outer = (input: U): string => { const local = make(input); console.log(local()); return local(); };
    const user: U = { name: "Pat" }; console.log(outer(user)); user.name = "Sam"; console.log(outer(user));
}"#,
        "Pat\nPat\nSam\nSam\n",
        Some(&[("make.user", Mode::Read)]),
    );
    reject(
        "function make(text: string): () => string { return (): string => { return text; }; } function main() { const text = \"Pat\"; const outer = (): string => { const local = make(text); return local(); }; outer(); outer(); }",
        "closure-consumed",
    );
}

#[test]
fn named_inner_loans_keep_block_lifetime_and_conflict_spans() {
    let source = r#"type U = { name: string; }; function main() {
    const outer = (user: U): void => {
        if (true) { const inner = (): void => { user.name = "Sam"; }; inner(); }
        console.log(user.name);
    };
    outer({ name: "Pat" });
}"#;
    execute(source, "Sam\n", None);
    let bad = source.replace("inner(); }", "inner(); console.log(user.name); }");
    let error = reject(&bad, "capture-conflict");
    let creation = error.related[0].0;
    assert!(bad[creation.start..creation.end].starts_with("(): void =>"));
    for source in [
        "type U = { name: string; }; function main() { const outer = (user: U): void => { const inner = (): void => { user.name = \"Sam\"; }; console.log(user.name); }; }",
        "type U = { name: string; }; function main() { const outer = (user: U): void => { const inner = (): void => { console.log(user.name); }; user.name = \"Sam\"; }; }",
        "type U = { name: string; }; function main() { const outer = (user: U): void => { const first = (): void => { user.name = \"Sam\"; }; const second = (): void => { console.log(user.name); }; }; }",
    ] {
        reject(source, "capture-conflict");
    }
}

#[test]
fn branch_joins_and_uncalled_nested_creation_preserve_transfer_rules() {
    let source = r#"function main() {
    const outer = (text: string, early: boolean): string => {
        if (early) { const take = (): string => { return text; }; return take(); }
        return text;
    };
    console.log(outer("early", true)); console.log(outer("late", false));
}"#;
    execute(source, "early\nlate\n", None);
    reject(&source.replace("return take();", "take();"), "transferred");
    let uncalled = "function main() { const text = \"Pat\"; const outer = (): void => { const inner = (): void => { const saved = text; }; }; outer(); outer(); }";
    reject(uncalled, "closure-consumed");
    reject(
        "function main() { const text = \"Pat\"; const outer = (): void => { const inner = (): void => { const saved = text; }; }; console.log(text); }",
        "transferred",
    );
}

#[test]
fn declaration_moves_preserve_callback_and_escape_boundaries() {
    execute(
        "function call(f: Once<Function<void>>) { f(); } function use(f: Once<Function<void>>) { const outer = (): void => { call(f); }; outer(); } function main() { use((): void => { console.log(7); }); }",
        "7\n",
        None,
    );
    for source in [
        "function main() { const first = (): void => {}; const outer = (): void => { first(); }; const alias = outer; }",
        "function main() { const outer = (): void => { const first = (): void => {}; const inner = (): void => { first(); }; const alias = inner; }; }",
        "function use(f: Mutable<Function<void>>) { f(); } function main() { const first = (): void => {}; const outer = (): void => { use(first); }; const alias = outer; }",
        "function main() { const outer = (): void => { const local = (): void => {}; const alias = local; }; }",
    ] {
        execute(source, "", None);
    }
    for source in [
        "function use(f: Mutable<Function<void>>) { const outer = (): void => { const inner = (): void => { f(); }; }; } function main() {}",
    ] {
        reject(source, "callback-escape");
    }
    reject(
        "function main() { const outer = (): () int => { return (): int => { return 7; }; }; }",
        "syntax",
    );
    reject(
        "function main() { const outer = (): void => { return (): void => {}; }; }",
        "type",
    );
    reject(
        "function main() { const outer = (): void => { if (true) { const local = (): void => {}; } local(); }; }",
        "name",
    );
}

#[test]
fn outer_argument_loans_remain_active_during_nested_capture_creation() {
    reject(
        "type U = { name: string; }; function use(user: U, f: Mutable<Function<void>>) { console.log(user.name); f(); } function main() { const outer = (input: U): void => { use(input, (): void => { const inner = (): void => { input.name = \"Sam\"; }; inner(); }); }; }",
        "conflicting-access",
    );
    reject(
        "type U = { name: string; }; function use(f: Mutable<Function<void>>, user: U) { f(); console.log(user.name); } function main() { const outer = (input: U): void => { use((): void => { const inner = (): void => { input.name = \"Sam\"; }; inner(); }, input); }; }",
        "capture-conflict",
    );
    execute(
        "function use(f: Mutable<Function<int>>): int { return f(); } function main() { const outer = (n: int): int => { return use((): int => { const inner = (): int => { return n; }; return inner(); }); }; console.log(outer(7)); }",
        "7\n",
        None,
    );
}

#[test]
fn lexical_depth_metadata_and_nesting_limits_are_preserved() {
    let mut body = "console.log(text);".to_string();
    for level in (0..12).rev() {
        body = format!("const f{level} = (): void => {{ {body} }}; f{level}();");
    }
    let source = format!("function main() {{ const text = \"Pat\"; {body} }}");
    execute(&source, "Pat\n", None);
    let analysis = check(&source).unwrap();
    let mut depths = analysis
        .closures()
        .iter()
        .map(|c| c.depth)
        .collect::<Vec<_>>();
    depths.sort_unstable();
    assert_eq!(depths, (1..=12).collect::<Vec<_>>());
    assert!(analysis.to_json().contains("\"depth\":12"));
    let dir = support::TempDirectory::new();
    let output = support::run(support::cli().arg("check").arg(dir.source(&source)), 10);
    support::success(&output);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("(depth 12)")
    );
    let deep = format!(
        "function main() {{ {} {} }}",
        "const f = (): void => {".repeat(200),
        "};".repeat(200)
    );
    reject(&deep, "unsupported");
}

#[test]
fn nested_checks_and_errors_need_no_backend() {
    let valid = include_str!("fixtures/callbacks/nested_local.tsr");
    let invalid = "function main() { const outer = (): void => { const inner = (): int => { return false; }; inner(); }; }";
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
