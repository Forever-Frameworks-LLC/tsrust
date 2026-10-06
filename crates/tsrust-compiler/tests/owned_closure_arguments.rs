mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn direct_and_callback_owned_arguments_preserve_inferred_access() {
    let rust = execute(
        include_str!("fixtures/callbacks/owned_arguments.tsr"),
        "Sam\nSam\nJo\n",
        Some(&[("apply.action", Mode::Mutate), ("apply.user", Mode::Mutate)]),
    );
    assert!(rust.contains("for<'p0> Callback<(&'p0 mut Record0,), ()>"));
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
}

#[test]
fn consuming_fresh_arguments_does_not_consume_the_environment() {
    let source = r#"type User = { name: string; };
function apply(action: Mutable<Function<User, [user: Owned<User>]>>, user: User): User { return action(user); }
function main() {
    const text = "take";
    const take = (input: User): User => { console.log(text); input.name = "saved"; return input; };
    const first = apply(take, { name: "first" }); console.log(first.name);
    const second = apply(take, { name: "second" }); console.log(second.name);
    console.log(text);
}"#;
    execute(
        source,
        "take\nsaved\ntake\nsaved\ntake\n",
        Some(&[
            ("apply.action", Mode::Mutate),
            ("apply.user", Mode::Consume),
        ]),
    );
    let closures = check(source).unwrap().closures();
    assert_eq!(closures[0].invocation, Mode::Read);
    assert_eq!(closures[0].parameters[0].access, Mode::Consume);
    assert_eq!(closures[0].captures[0].access, Mode::Read);
}

#[test]
fn string_read_and_consume_contracts_work_with_temporary_inputs() {
    execute(
        r#"function borrow(action: Mutable<Function<string, [text: ReadOnly<string>]>>, text: string): string { return action(text); }
function take(action: Mutable<Function<string, [text: Owned<string>]>>, text: string): string { return action(text); }
function main() {
    const trim = (input: string): string => { return input.trim(); };
    const text = " Pat "; console.log(borrow(trim, text)); console.log(text);
    console.log(borrow(trim, " Sam "));
    const identity = (input: string): string => { return input; };
    console.log(take(identity, text)); console.log(take(identity, "Jo"));
}"#,
        "Pat\n Pat \nSam\n Pat \nJo\n",
        Some(&[
            ("borrow.action", Mode::Mutate),
            ("borrow.text", Mode::Read),
            ("take.action", Mode::Mutate),
            ("take.text", Mode::Consume),
        ]),
    );
}

#[test]
fn owned_factory_environments_do_not_retain_argument_borrows() {
    execute(
        r#"type User = { name: string; };
function make(prefix: string): (user: ReadOnly<User>) => string {
    return (input: User): string => { console.log(prefix); return input.name.trim(); };
}
function forward(prefix: string): (renamed: ReadOnly<User>) => string { return make(prefix); }
function use(action: Mutable<Function<string, [u: ReadOnly<User>]>>, user: User): string { return action(user); }
function main() {
    const read = forward("hello");
    if (true) { const temporary: User = { name: " Pat " }; console.log(use(read, temporary)); }
    const result = use(read, { name: " Sam " }); console.log(result);
    const user: User = { name: "Jo" }; console.log(read(user)); user.name = "new"; console.log(user.name);
}"#,
        "hello\nPat\nhello\nSam\nhello\nJo\nnew\n",
        None,
    );
}

#[test]
fn mixed_access_arguments_have_independent_lifetimes_and_captures() {
    let rust = execute(
        r#"type User = { name: string; };
function apply(action: Mutable<Function<string, [a: ReadOnly<User>, b: Mutable<User>, c: Owned<User>, flag: boolean]>>, a: User, b: User, c: User): string {
    return action(a, b, c, true);
}
function main() {
    const capture = "captured";
    const mix = (reader: User, changed: User, owned: User, enabled: boolean): string => {
        console.log(capture); console.log(reader.name); console.log(enabled);
        changed.name = owned.name.trim();
        const saved = owned; return saved.name.trim();
    };
    const a: User = { name: "A" }; const b: User = { name: "B" };
    console.log(apply(mix, a, b, { name: " C " })); console.log(a.name); console.log(b.name);
    console.log(apply(mix, { name: "D" }, b, { name: "E" })); console.log(b.name);
}"#,
        "captured\nA\ntrue\nC\nA\nC\ncaptured\nD\ntrue\nE\nE\n",
        None,
    );
    assert!(rust.contains(
        "for<'p0, 'p1> Callback<(&'p0 Record0, &'p1 mut Record0, Record0, bool,), String>"
    ));
    assert!(rust.contains("impl<'c0, 'p0, 'p1> Callback<"));
}

#[test]
fn once_environment_and_argument_ownership_are_separate_contracts() {
    execute(
        r#"type User = { name: string; };
function take(action: Once<Function<string, [u: ReadOnly<User>]>>, user: User): string { return action(user); }
function make(text: string): (u: ReadOnly<User>) => string {
    return (input: User): string => { console.log(input.name); return text; };
}
function main() {
    const user: User = { name: "Pat" };
    console.log(take(make("owned capture"), user)); console.log(user.name);
}"#,
        "Pat\nowned capture\nPat\n",
        Some(&[
            ("take.action", Mode::Consume),
            ("take.user", Mode::Read),
            ("make.text", Mode::Consume),
        ]),
    );
    execute(
        r#"type User = { name: string; };
function take(action: Once<Function<User, [u: Owned<User>]>>, user: User): User { return action(user); }
function main() { const identity = (input: User): User => { return input; }; const saved = take(identity, { name: "Pat" }); console.log(saved.name); }"#,
        "Pat\n",
        Some(&[("take.action", Mode::Consume), ("take.user", Mode::Consume)]),
    );
}

#[test]
fn forwarding_and_branch_argument_effects_reach_enclosing_contracts() {
    execute(
        r#"type User = { name: string; };
function outer(action: Mutable<Function<void, [u: Mutable<User>]>>, user: User, flag: boolean) { inner(action, user, flag); }
function inner(action: Mutable<Function<void, [u: Mutable<User>]>>, user: User, flag: boolean) { if (flag) { action(user); } else { console.log(user.name); } }
function main() {
    const change = (target: User): void => { target.name = "Sam"; };
    const user: User = { name: "Pat" }; outer(change, user, false); outer(change, user, true); console.log(user.name);
}"#,
        "Pat\nSam\n",
        Some(&[
            ("outer.action", Mode::Mutate),
            ("outer.user", Mode::Mutate),
            ("outer.flag", Mode::Copy),
            ("inner.action", Mode::Mutate),
            ("inner.user", Mode::Mutate),
            ("inner.flag", Mode::Copy),
        ]),
    );
    execute(
        r#"type User = { name: string; };
function run(action: Mutable<Function<User, [u: Owned<User>]>>, user: User, early: boolean): User {
    if (early) { return action(user); }
    return action(user);
}
function main() {
    const identity = (input: User): User => { return input; };
    const a = run(identity, { name: "A" }, true); console.log(a.name);
    const b = run(identity, { name: "B" }, false); console.log(b.name);
}"#,
        "A\nB\n",
        None,
    );
    reject(
        "type U = { name: string; }; function bad(f: (u: Owned<U>) => U, user: U, flag: boolean) { if (flag) { f(user); } console.log(user.name); } function main() {}",
        "transferred",
    );
}

#[test]
fn signature_access_matches_exactly_for_closures_factories_and_forwarding() {
    let bodies = [
        ("ReadOnly", "console.log(input.name);"),
        ("Mutable", "input.name = \"Sam\";"),
        ("Owned", "const saved = input;"),
    ];
    for (actual, body) in bodies {
        for expected in ["ReadOnly", "Mutable", "Owned"] {
            let source = format!(
                "type U = {{ name: string; }}; function use(f: (u: {expected}<U>) => void) {{}} function main() {{ use((input: U): void => {{ {body} }}); }}"
            );
            if actual == expected {
                assert!(check(&source).is_ok());
            } else {
                reject(&source, "callback-access");
            }
        }
    }
    for source in [
        "type U = { name: string; }; function use(f: (u: ReadOnly<U>) => void) {} function forward(f: (u: Mutable<U>) => void) { use(f); } function main() {}",
        "type U = { name: string; }; function make(): (u: Owned<U>) => void { return (input: U): void => { console.log(input.name); }; } function main() {}",
        "type U = { name: string; }; function make(): (u: ReadOnly<U>) => void { return (input: U): void => { input.name = \"Sam\"; }; } function main() {}",
        "function use(f: (s: ReadOnly<string>) => string) {} function main() { use((s: string): string => { return s; }); }",
    ] {
        reject(source, "callback-access");
    }
    reject(
        "type A = { name: string; }; type B = { name: string; }; function use(f: (u: ReadOnly<A>) => void) {} function main() { use((u: B): void => {}); }",
        "type",
    );
}

#[test]
fn owned_arguments_obey_whole_binding_loans_and_left_to_right_evaluation() {
    let prefix =
        "type U = { name: string; }; function main() { const user: U = { name: \"Pat\" }; ";
    for (body, code) in [
        (
            "const f = (a: U, b: U): void => { a.name = \"Sam\"; console.log(b.name); }; f(user, user);",
            "conflicting-access",
        ),
        (
            "const f = (a: U, b: U): void => { console.log(a.name); b.name = \"Sam\"; }; f(user, user);",
            "conflicting-access",
        ),
        (
            "const f = (a: U, b: U): void => { const saved = a; console.log(b.name); }; f(user, user);",
            "transferred",
        ),
        (
            "const f = (a: U, b: U): void => { console.log(a.name); const saved = b; }; f(user, user);",
            "conflicting-access",
        ),
        (
            "const f = (a: string, b: U): void => { console.log(a); b.name = \"Sam\"; }; f(user.name, user);",
            "conflicting-access",
        ),
    ] {
        let error = reject(&format!("{prefix}{body}}}"), code);
        assert!(!error.related.is_empty());
    }
    execute(
        &format!(
            "{prefix}const f = (a: U, b: U): void => {{ console.log(a.name); console.log(b.name); }}; f(user, user); user.name = \"Sam\"; console.log(user.name);}}"
        ),
        "Pat\nPat\nSam\n",
        None,
    );
    reject(
        "type U = { name: string; }; function bad(f: (a: ReadOnly<U>, b: Mutable<U>) => void, user: U) { f(user, user); } function main() {}",
        "conflicting-access",
    );
}

#[test]
fn computed_owned_results_release_inputs_but_live_arguments_do_not() {
    execute(
        r#"type U = { name: string; };
function mutate(user: U): string { user.name = "Sam"; return user.name.trim(); }
function main() {
    const user: U = { name: " Pat " };
    const show = (first: string, second: string): void => { console.log(first); console.log(second); };
    show(user.name.trim(), mutate(user)); console.log(user.name);
}"#,
        "Pat\nSam\nSam\n",
        None,
    );
    reject(
        r#"type U = { name: string; }; function mutate(user: U): string { user.name = "Sam"; return "changed"; } function main() { const user: U = { name: "Pat" }; const show = (first: string, second: string): void => { console.log(first); console.log(second); }; show(user.name, mutate(user)); }"#,
        "conflicting-access",
    );
    reject(
        "type U = { name: string; }; function main() { const user: U = { name: \"Pat\" }; const take = (input: U): U => { return input; }; const saved = take(user); console.log(user.name); }",
        "transferred",
    );
}

#[test]
fn capture_loans_and_receiver_access_still_constrain_owned_arguments() {
    for source in [
        "type U = { name: string; }; function main() { const user: U = { name: \"Pat\" }; const f = (input: U): void => { console.log(user.name); input.name = \"Sam\"; }; f(user); }",
        "type U = { name: string; }; function main() { const user: U = { name: \"Pat\" }; const f = (input: U): void => { user.name = \"Sam\"; console.log(input.name); }; f(user); }",
    ] {
        reject(source, "capture-conflict");
    }
    reject(
        "type U = { name: string; }; function bad(f: (u: ReadOnly<U>) => U, user: U): U { return f(f(user)); } function main() {}",
        "conflicting-access",
    );
    execute(
        r#"type U = { name: string; }; function main() {
    const read = (input: U): U => { return clone(input); };
    const user: U = { name: "Pat" }; const saved = read(read(user)); console.log(saved.name); console.log(user.name);
}"#,
        "Pat\nPat\n",
        None,
    );
}

#[test]
fn access_words_remain_identifiers_and_invalid_annotations_are_diagnosed() {
    execute(
        r#"type read = { name: string; }; type mutate = { name: string; }; type consume = { name: string; };
function read(read: read) { console.log(read.name); }
function apply(action: (read: ReadOnly<read>) => void, value: read) { action(value); }
function main() { const read: read = { name: "Pat" }; const mutate = (consume: read): void => { console.log(consume.name); }; apply(mutate, read); }"#,
        "Pat\n",
        None,
    );
    for source in [
        "function use(f: (s: string) => void) {} function main() {}",
        "type U = { name: string; }; function make(): (u: U) => void { return (input: U): void => {}; } function main() {}",
        "function use(f: (n: ReadOnly<int>) => void) {} function main() {}",
        "function use(f: (b: Owned<boolean>) => void) {} function main() {}",
        "function main() { const f = (n: Mutable<int>): void => {}; }",
    ] {
        reject(source, "callback-access");
    }
    // Arrow parameters may state access for owned data, in either spelling.
    for source in [
        "function main() { const f = (s: ReadOnly<string>): void => {}; }",
        "function main() { const f = (s: ReadOnly<string>): void => {}; }",
        "function main() { const f = (values: Mutable<int[]>): void => { values.push(1); }; }",
    ] {
        assert!(check(source).is_ok(), "{source}");
    }
    reject(
        "function use(f: (s: ReadOnly<void>) => void) {} function main() {}",
        "unsupported",
    );
    reject(
        "function use(f: (s: ReadOnly<Missing>) => void) {} function main() {}",
        "type",
    );
    reject(
        "function use(f: (s: ReadOnly<Mutable<string>>) => void) {} function main() {}",
        "callback-access",
    );
}

#[test]
fn inferred_argument_access_is_available_in_library_json_and_text_checks() {
    let source = r#"type U = { name: string; }; function main() {
    const f = (a: U, b: U, c: U, n: int): void => { console.log(a.name); b.name = "Sam"; const saved = c; console.log(n); };
}"#;
    let analysis = check(source).unwrap();
    let closures = analysis.closures();
    assert_eq!(
        closures[0]
            .parameters
            .iter()
            .map(|p| (p.name.as_str(), p.access))
            .collect::<Vec<_>>(),
        vec![
            ("a", Mode::Read),
            ("b", Mode::Mutate),
            ("c", Mode::Consume),
            ("n", Mode::Copy)
        ]
    );
    assert!(closures[0].captures.is_empty());
    assert_eq!(closures[0].invocation, Mode::Read);
    assert!(
        analysis
            .to_json()
            .contains("\"parameters\":[{\"name\":\"a\",\"access\":\"read\"}")
    );
    let dir = support::TempDirectory::new();
    let output = support::run(support::cli().arg("check").arg(dir.source(source)), 10);
    support::success(&output);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("parameter c: consume")
    );
}

#[test]
fn owned_argument_checks_need_no_backend_and_unsupported_escape_stays_rejected() {
    let valid = include_str!("fixtures/callbacks/owned_arguments.tsr");
    let invalid = "function use(f: (s: ReadOnly<string>) => void) {} function main() { use((s: string): void => { const saved = s; }); }";
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
            assert!(error.contains("callback-access"), "{error}");
            assert!(!error.contains("rustup"));
        }
    }
    reject(
        "function invalid(f: Mutable<Function<string, [s: ReadOnly<string>]>>): (s: ReadOnly<string>) => string { return f; } function main() {}",
        "callback-escape",
    );
    execute(
        "type U = { name: string; }; function main() { const f = (input: U): string => { return input.name; }; const u: U = { name: \"Pat\" }; console.log(f(u)); }",
        "Pat\n",
        None,
    );
    execute(
        "function main() { const f = (input: string): void => { const nested = (): void => { console.log(input); }; const alias = nested; }; }",
        "",
        None,
    );
}
