mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn owned_capture_moves_through_single_use_forwarding() {
    let rust = execute(
        include_str!("fixtures/callbacks/single_use.tsr"),
        "Pat\n",
        Some(&[
            ("take.action", Mode::Consume),
            ("forward.action", Mode::Consume),
        ]),
    );
    assert!(rust.contains("OnceCallback::invoke_once(value0, ())"));
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden), "{hidden}");
    }
}

#[test]
fn read_mutable_and_returned_environments_satisfy_single_use_contracts() {
    execute(
        r#"type State = { on: boolean; };
function take(action: Once<Function<string>>): string { return action(); }
function change(action: Once<Function<boolean>>): boolean { return action(); }
function make(text: string): () => string { return (): string => { return text; }; }
function main() {
    const text = "read"; const snapshot=clone(text); const read: Once<Function<string>> = (): string => { return snapshot.trim(); };
    console.log(take(read)); console.log(text);
    const state: State = { on: false };
    const snapshotState=clone(state);
    console.log(change((): boolean => { snapshotState.on = true; return snapshotState.on; }));
    console.log(state.on);
    console.log(take(make("temporary")));
    const returned = make("named"); console.log(take(returned));
}"#,
        "read\nread\ntrue\nfalse\ntemporary\nnamed\n",
        Some(&[
            ("take.action", Mode::Consume),
            ("change.action", Mode::Consume),
            ("make.text", Mode::Consume),
        ]),
    );
    execute(
        r#"type State = { on: boolean; };
function make(): () => boolean {
    const state: State = { on: false };
    return (): boolean => { state.on = true; return state.on; };
}
function use(action: Once<Function<boolean>>) { console.log(action()); }
function main() { use(make()); }"#,
        "true\n",
        None,
    );
}

#[test]
fn unused_parameters_still_transfer_ownership_without_invoking() {
    let source = r#"function ignore(action: Once<Function<string>>) {}
function forward(action: Once<Function<string>>) { ignore(action); }
function main() {
    const text = "unprinted";
    forward((): string => { console.log(text); return text; });
    console.log("done");
}"#;
    execute(
        source,
        "done\n",
        Some(&[
            ("ignore.action", Mode::Consume),
            ("forward.action", Mode::Consume),
        ]),
    );
    reject(
        &source.replace("console.log(\"done\");", "console.log(text);"),
        "transferred",
    );
    reject(
        "function ignore(f: Once<Function<void>>) {} function main() { const f = (): void => {}; ignore(f); f(); }",
        "closure-consumed",
    );
}

#[test]
fn repeated_calls_and_forwarding_report_transfer_locations() {
    for body in [
        "action(); action();",
        "forward(action); action();",
        "action(); forward(action);",
        "forward(action); forward(action);",
    ] {
        let source = format!(
            "function forward(f: Once<Function<void>>) {{ f(); }} function bad(action: Once<Function<void>>) {{ {body} }} function main() {{}}"
        );
        let error = reject(&source, "callback-consumed");
        assert_eq!(&source[error.span.start..error.span.end], "action");
        let previous = error.related[0].0;
        assert_eq!(&source[previous.start..previous.end], "action");
        assert!(previous.start < error.span.start);
    }
    for tail in ["f();", "take(f);", "reuse(f);"] {
        reject(
            &format!(
                "function take(f: Once<Function<void>>) {{ f(); }} function reuse(f: Mutable<Function<void>>) {{ f(); }} function main() {{ const f = (): void => {{}}; take(f); {tail} }}"
            ),
            "closure-consumed",
        );
    }
}

#[test]
fn branch_exits_allow_at_most_one_call_on_each_runtime_path() {
    execute(
        r#"function choose(action: Once<Function<string>>, early: boolean): string {
    if (early) { return action(); }
    return action();
}
function either(action: Once<Function<string>>, left: boolean): string {
    if (left) { return action(); } else { return action(); }
}
function skip(action: Once<Function<string>>, skip: boolean): string {
    if (skip) { return "skipped"; }
    return action();
}
function main() {
    console.log(choose((): string => { return "early"; }, true));
    console.log(choose((): string => { return "late"; }, false));
    console.log(either((): string => { return "left"; }, true));
    console.log(either((): string => { return "right"; }, false));
    console.log(skip((): string => { return "unused"; }, true));
    console.log(skip((): string => { return "called"; }, false));
}"#,
        "early\nlate\nleft\nright\nskipped\ncalled\n",
        Some(&[
            ("choose.action", Mode::Consume),
            ("choose.early", Mode::Copy),
            ("either.action", Mode::Consume),
            ("either.left", Mode::Copy),
            ("skip.action", Mode::Consume),
            ("skip.skip", Mode::Copy),
        ]),
    );
    for branch in [
        "if (flag) { action(); }",
        "if (flag) {} else { action(); }",
        "if (flag) { action(); } else { action(); }",
        "if (flag) { forward(action); }",
    ] {
        reject(
            &format!(
                "function forward(f: Once<Function<void>>) {{ f(); }} function bad(action: Once<Function<void>>, flag: boolean) {{ {branch} action(); }} function main() {{}}"
            ),
            "callback-consumed",
        );
    }
}

#[test]
fn borrowed_and_owned_parameter_contracts_cannot_be_interchanged() {
    for (source_kind, target_kind) in [("Function<void>", "Once<Function<void>>"), ("Once<Function<void>>", "Function<void>")] {
        for body in ["f();", ""] {
            reject(
                &format!(
                    "function target(f: {target_kind}) {{ {body} }} function forward(f: {source_kind}) {{ target(f); }} function main() {{}}"
                ),
                "callback-contract",
            );
        }
    }
    // A concrete reusable closure can first be borrowed, then transferred.
    execute(
        "function borrow(f: Mutable<Function<int>>): int { return f(); } function take(f: Once<Function<int>>): int { return f(); } function main() { const f = (): int => { return 7; }; console.log(borrow(f)); console.log(take(f)); }",
        "7\n7\n",
        Some(&[("borrow.f", Mode::Mutate), ("take.f", Mode::Consume)]),
    );
}

#[test]
fn single_use_parameters_own_escaping_environments_but_borrowed_parameters_do_not() {
    execute("function factory(action:Once<Function<int>>):()=>int{return action;}function wrap(action:Once<Function<int>>):()=>int{return ():int=>{return action();};}function main(){const first=factory(():int=>{return 7;});const second=wrap(():int=>{return 8;});console.log(first(),second());}", "7 8\n", None);
    for modifier in ["ReadOnly<Function<int>>", "Mutable<Function<int>>"] {
        reject(&format!("function factory(action:{modifier}):()=>int{{return action;}}function main(){{}}"), "callback-escape");
        reject(&format!("function factory(action:{modifier}):()=>int{{return ():int=>{{return action();}};}}function main(){{}}"), "callback-escape");
    }
    reject("function invalid(action:Once<Function<int>>){console.log(action);}function main(){}", "callback-escape");
}

#[test]
fn argument_evaluation_prevents_duplicate_transfers_and_overlapping_access() {
    for source in [
        "function pair(a: Once<Function<void>>, b: Once<Function<void>>) {} function main() { const f = (): void => {}; pair(f, f); }",
        "function later(f: Once<Function<int>>, n: int) {} function main() { const f = (): int => { return 1; }; later(f, f()); }",
    ] {
        reject(source, "closure-consumed");
    }
    reject(
        "function pair(a: Once<Function<void>>, b: Once<Function<void>>) {} function forward(f: Once<Function<void>>) { pair(f, f); } function main() {}",
        "callback-consumed",
    );
    reject(
        "function pair(a: Mutable<Function<void>>, b: Once<Function<void>>) {} function main() { const f = (): void => {}; pair(f, f); }",
        "conflicting-access",
    );
    let prefix = r#"type User = { name: string; };
function first(action: Once<Function<void>>, user: User) { action(); console.log(user.name); }
function last(user: User, action: Once<Function<void>>) { console.log(user.name); action(); }
function main() { const user: User = { name: "Pat" }; "#;
    for (body, code) in [
        (
            "first((): void => { user.name = \"Sam\"; }, user);",
            "transferred",
        ),
        (
            "last(user, (): void => { user.name = \"Sam\"; });",
            "conflicting-access",
        ),
        (
            "first((): void => { const saved = user; }, user);",
            "transferred",
        ),
        (
            "last(user, (): void => { const saved = user; });",
            "conflicting-access",
        ),
    ] {
        let error = reject(&format!("{prefix}{body}}}"), code);
        assert!(!error.related.is_empty());
    }
}

#[test]
fn borrowed_named_environments_keep_block_capture_loans_after_calls() {
    let source = r#"type User = { name: string; };
function take(action: Mutable<Function<void>>) { action(); }
function main() {
    const user: User = { name: "Pat" };
    if (true) { const f = (): void => { user.name = "Sam"; }; take(f); }
    console.log(user.name);
}"#;
    execute(source, "Sam\n", None);
    reject(
        &source.replace("take(f); }", "take(f); console.log(user.name); }"),
        "capture-conflict",
    );
    reject(
        r#"type User = { name: string; }; function ignore(f: Mutable<Function<void>>) {} function main() { const user: User = { name: "Pat" }; const f = (): void => { console.log(user.name); }; ignore(f); user.name = "Sam"; }"#,
        "capture-conflict",
    );
}

#[test]
fn contextual_once_preserves_existing_identifiers_and_checks_result_types() {
    execute(
        r#"type once = { name: string; };
function once(once: once) { console.log(once.name); }
function take(once: Once<Function<once>>): once { return once(); }
function main() {
    const once: once = { name: "Pat" }; console.log(once.name);
    const saved = take((): once => { return once; }); console.log(saved.name);
}"#,
        "Pat\nPat\n",
        Some(&[("once.once", Mode::Read), ("take.once", Mode::Consume)]),
    );
    for (source, code) in [
        (
            "function use(f: Once<Function<int>>) {} function main() { use((): boolean => { return true; }); }",
            "type",
        ),
        (
            "function use(f: Once<Function<int>>) {} function forward(f: Once<Function<string>>) { use(f); } function main() {}",
            "type",
        ),
        (
            "function use(f: Once<Function<void>>) {} function main() { use(7); }",
            "type",
        ),
        (
            "function use(f: Once<Function<Missing>>) {} function main() {}",
            "type",
        ),
        (
            "function use(f: Once<Function<void>>) { f(1); } function main() {}",
            "arity",
        ),
        (
            "function use(f: Once<Function<void, [x: string]>>) {} function main() {}",
            "callback-access",
        ),
        (
            "function use(f: Once<Function<() =>>>) {} function main() {}",
            "syntax",
        ),
        (
            "function use(f: Once<Function<void>) {} function main() {}",
            "syntax",
        ),
        (
            "function use(f: Once<Function<) {} function main() {}",
            "syntax",
        ),
        (
            "function use(f: Once<Function<void>>): () => int { return f; } function main() {}",
            "type",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn single_use_checks_and_errors_need_no_backend() {
    let valid = include_str!("fixtures/callbacks/single_use.tsr");
    let invalid = "function bad(f: Once<Function<void>>) { f(); f(); } function main() {}";
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
            assert!(error.contains("callback-consumed"), "{error}");
            assert!(!error.contains("rustup"));
        }
    }
    assert!(
        check(valid)
            .unwrap()
            .to_json()
            .contains("\"take.action\":\"consume\"")
    );
}
