mod support;

use support::closure_corpus::{BackendExpectation, CASES, SourceExpectation};
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn complete_source_closure_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/closures");
    for case in CASES {
        if matches!(case.source, SourceExpectation::BackendOnly) {
            continue;
        }
        let source = std::fs::read_to_string(root.join(format!("{}.tsr", case.name))).unwrap();
        match case.source {
            SourceExpectation::Accept => {
                let BackendExpectation::Output(expected) = case.backend else {
                    panic!("accepted witness");
                };
                let rust = execute(&source, expected, None);
                if case.name != "returned_clone_independent" {
                    assert!(!rust.contains("Clone::clone("));
                }
                assert!(!rust.contains("Box<"));
                assert!(!rust.contains("dyn "));
            }
            SourceExpectation::Reject(code) => {
                // Fixture directives suppress editor/check output only. Strict analysis
                // and emission must keep rejecting the original, unchanged source.
                let error = check(&source).err().expect("strict source rejection");
                assert_eq!(error.code, code, "{}", case.name);
                assert!(!error.related.is_empty(), "{}", case.name);
                let output=support::run(support::cli().arg("emit").arg(root.join(format!("{}.tsr",case.name))),10);
                assert!(!output.status.success(), "{}",case.name);
                assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")), "{}",case.name);
            }
            SourceExpectation::BackendOnly => unreachable!(),
        }
    }
}

#[test]
fn capture_storage_and_invocation_are_independent_and_inspectable() {
    let source = r#"type User = { name: string; };
function make(user: User): () => string { return (): string => { return user.name.trim(); }; }
function main() { const read = make({ name: "Pat" }); console.log(read()); console.log(read()); }"#;
    execute(source, "Pat\nPat\n", Some(&[("make.user", Mode::Consume)]));
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.message));
    let closures = analysis.closures();
    assert_eq!(closures.len(), 1);
    let closure = &closures[0];
    assert!(closure.escapes);
    assert_eq!(closure.invocation, Mode::Read);
    assert_eq!(closure.captures[0].name, "user");
    assert_eq!(closure.captures[0].storage, Mode::Consume);
    assert_eq!(closure.captures[0].access, Mode::Read);
    let span = closure.captures[0].span;
    assert_eq!(&source[span.start..span.end], "user");
    assert!(
        analysis
            .to_json()
            .contains("\"storage\":\"consume\",\"access\":\"read\"")
    );
}

#[test]
fn local_capture_contracts_propagate_through_forward_calls() {
    execute(
        r#"type User = { name: string; };
function outer(user: User) { local(user); }
function local(user: User) {
    if (true) { const change = (): void => { rename(user); }; change(); change(); }
    console.log(user.name);
}
function rename(user: User) { user.name = "Sam"; }
function main() { const user: User = { name: "Pat" }; outer(user); console.log(user.name); }"#,
        "Sam\nSam\n",
        Some(&[
            ("outer.user", Mode::Mutate),
            ("local.user", Mode::Mutate),
            ("rename.user", Mode::Mutate),
        ]),
    );
}

#[test]
fn conditional_consumption_sets_once_contract_on_every_path() {
    let source = r#"type User = { name: string; };
function run(user: User, flag: boolean) {
    const action = (): void => {
        if (flag) { const saved = user; console.log(saved.name); return; }
        console.log(user.name);
    };
    action();
}
function main() { run({ name: "taken" }, true); run({ name: "read" }, false); }"#;
    execute(
        source,
        "taken\nread\n",
        Some(&[("run.user", Mode::Consume), ("run.flag", Mode::Copy)]),
    );
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.message));
    assert_eq!(analysis.closures()[0].invocation, Mode::Consume);
    reject(
        &source.replace("    action();", "    action(); action();"),
        "closure-consumed",
    );
}

#[test]
fn uncalled_captures_apply_effects_but_unused_visible_values_do_not() {
    let source = r#"type User = { name: string; };
function mutate(user: User) { const action = (): void => { user.name = "Sam"; }; }
function consume(user: User) { const action = (): User => { return user; }; }
function main() {
    const user: User = { name: "Pat" };
    const empty = (): int => { return 7; };
    user.name = "Sam";
    console.log(empty());
    console.log(user.name);
    mutate(user);
    console.log(user.name);
    consume(user);
}"#;
    execute(
        source,
        "7\nSam\nSam\n",
        Some(&[
            ("mutate.user", Mode::Mutate),
            ("consume.user", Mode::Consume),
        ]),
    );
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.message));
    assert!(
        analysis
            .closures()
            .iter()
            .find(|c| c.function == "main")
            .unwrap()
            .captures
            .is_empty()
    );
    reject(
        r#"type User = { name: string; };
function main() { const user: User = { name: "Pat" }; const unused = (): void => { user.name = "Sam"; }; console.log(user.name); }"#,
        "capture-conflict",
    );
}

#[test]
fn string_and_mixed_captures_lower_without_hidden_cloning() {
    let rust = execute(
        r#"type User = { name: string; };
function make(text: string, id: int): () => string {
    return (): string => { console.log(id); return text.trim(); };
}
function main() {
    const reader: User = { name: "read" };
    const changed: User = { name: "old" };
    const owned: User = { name: "owned" };
    if (true) {
        const action = (): User => { console.log(reader.name); changed.name = "new"; return owned; };
        const saved = action(); console.log(saved.name);
    }
    console.log(changed.name);
    const text = " Pat ";
    const read = make(text, 7); console.log(read()); console.log(read());
}"#,
        "read\nowned\nnew\n7\nPat\n7\nPat\n",
        Some(&[("make.text", Mode::Consume), ("make.id", Mode::Copy)]),
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn named_escape_is_owned_at_creation_and_single_origin_returns_work() {
    execute(
        r#"type User = { name: string; };
function forward(user: User): () => string { return make(user, false); }
function make(user: User, early: boolean): () => string {
    const read = (): string => { return user.name.trim(); };
    if (early) { return read; }
    console.log(read());
    return read;
}
function main() {
    const first = make({ name: "early" }, true); console.log(first());
    const second = forward({ name: "late" }); console.log(second());
}"#,
        "early\nlate\nlate\n",
        Some(&[
            ("forward.user", Mode::Consume),
            ("make.user", Mode::Consume),
            ("make.early", Mode::Copy),
        ]),
    );
    let source = r#"type User = { name: string; };
function make(user: User): () => string {
    const read = (): string => { return user.name.trim(); };
    console.log(user.name);
    return read;
}
function main() {}"#;
    let error = reject(source, "transferred");
    assert!(source[error.related[0].0.start..error.related[0].0.end].starts_with("(): string =>"));
}

#[test]
fn escaping_mutable_environment_retains_state_across_calls() {
    execute(
        r#"type State = { on: boolean; };
function make(): () => boolean {
    const state: State = { on: false };
    const toggle = (): boolean => {
        if (state.on) { state.on = false; } else { state.on = true; }
        return state.on;
    };
    return toggle;
}
function main() { const toggle = make(); console.log(toggle()); console.log(toggle()); console.log(toggle()); }"#,
        "true\nfalse\ntrue\n",
        None,
    );
}

#[test]
fn once_closure_state_joins_and_returning_paths_match_owned_values() {
    let source = r#"type User = { name: string; };
function run(flag: boolean) {
    const user: User = { name: "Pat" };
    const get = (): User => { return user; };
    if (flag) { const saved = get(); console.log(saved.name); return; }
    const saved = get(); console.log(saved.name);
}
function main() { run(true); run(false); }"#;
    execute(source, "Pat\nPat\n", Some(&[("run.flag", Mode::Copy)]));
    reject(
        &source.replace(
            "console.log(saved.name); return;",
            "console.log(saved.name);",
        ),
        "closure-consumed",
    );
}

#[test]
fn block_and_early_return_cleanup_release_capture_loans() {
    execute(
        r#"type User = { name: string; };
function run(user: User, flag: boolean) {
    if (flag) { const change = (): void => { user.name = "early"; }; change(); return; }
    if (true) { const read = (): void => { console.log(user.name); }; read(); }
    user.name = "late";
}
function main() {
    const user: User = { name: "Pat" };
    run(user, false); console.log(user.name);
    run(user, true); console.log(user.name);
}"#,
        "Pat\nlate\nearly\n",
        Some(&[("run.user", Mode::Mutate), ("run.flag", Mode::Copy)]),
    );
}

#[test]
fn capture_conflicts_and_transfers_report_both_source_locations() {
    for source in [
        r#"type User = { name: string; }; function make(user: User): () => string { const inspect = (): void => { console.log(user.name); }; return (): string => { return user.name.trim(); }; } function main() {}"#,
        r#"type User = { name: string; }; function main() { const user: User = { name: "Pat" }; const change = (): void => { user.name = "Sam"; }; const read = (): void => { console.log(user.name); }; }"#,
        r#"type User = { name: string; }; function main() { const user: User = { name: "Pat" }; const read = (): void => { console.log(user.name); }; const get = (): User => { return user; }; }"#,
    ] {
        let error = reject(source, "capture-conflict");
        let related = error.related[0].0;
        assert!(source[related.start..related.end].starts_with("(): void =>"));
        assert!(related.start < error.span.start);
    }
    reject(
        r#"type User = { name: string; }; function main() { const user: User = { name: "Pat" }; const saved = user; const read = (): void => { console.log(user.name); }; }"#,
        "transferred",
    );
    reject(
        r#"type User = { name: string; }; function main() { const user: User = { name: "Pat" }; const bad = (): void => { const saved = user; console.log(user.name); }; }"#,
        "transferred",
    );
}

#[test]
fn unsupported_escape_routes_and_invalid_closures_are_source_errors() {
    execute(
        "type U = { f: () => void; }; function main() { const value: U = { f: (): void => { console.log(7); } }; value.f(); }",
        "7\n",
        None,
    );
    for source in [
        "function main() { const f = (): void => {}; const g = f; }",
        "function main() { const f = (): void => {}; const g = (): void => { f(); }; const alias = g; }",
        "function main() { const f = (): void => { const g = (): void => {}; const alias = g; }; }",
        "function make(): () => int { return (): int => { return 1; }; } function main() { const f = (): void => { const g = make(); const alias = g; }; }",
    ] {
        execute(source, "", None);
    }
    for (source, code) in [
        ("function main() { const f = (): int => {}; }", "type"),
        (
            "function main() { const f = (): int => { return true; }; }",
            "type",
        ),
        (
            "function main() { const f = (): int => { if (true) { return 1; } }; }",
            "type",
        ),
        ("function main() { const f = (): Missing => {}; }", "type"),
        (
            "function make(): () => int { return (): string => { return \"wrong\"; }; } function main() {}",
            "type",
        ),
        ("function make(): () => int {} function main() {}", "type"),
        (
            "function make(flag: boolean): () => int { if (flag) { return (): int => { return 1; }; } else { return (): int => { return 2; }; } } function main() {}",
            "unsupported",
        ),
        (
            "function main() { const f = (): void => { f(); }; }",
            "name",
        ),
        (
            "function main() { const f = (): void => {}; f(1); }",
            "arity",
        ),
        (
            "function main() { const f = (): void => {}; console.log(f); }",
            "unsupported",
        ),
        (
            "function main() { const f = (): void => {}; f = (): void => {}; }",
            "unsupported",
        ),
        ("function main() { (): void => {}; }", "unsupported"),
        (
            "function main() { if (true) { const f = (): void => {}; } f(); }",
            "name",
        ),
        (
            "function use(f: (x: string) => void) {} function main() {}",
            "callback-access",
        ),
        (
            "function main() { const f = (x: void): void => {}; }",
            "unsupported",
        ),
        (
            "type U = { name: string; }; function make(user: U): () => string { return (): string => { const name = user.name; return user.name; }; } function main() {}",
            "partial-transfer",
        ),
        (
            "function make(): () => int { return make(); } function main() {}",
            "unsupported",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn closure_checks_and_rejections_need_no_backend() {
    for (source, valid) in [
        (
            "function main() { const f = (): int => { return 7; }; console.log(f()); }",
            true,
        ),
        (
            "function main() { const f = (): int => { return false; }; }",
            false,
        ),
    ] {
        let directory = support::TempDirectory::new();
        let path = directory.source(source);
        let output = support::run(
            support::cli()
                .arg("check")
                .arg(path)
                .arg("--json")
                .env("PATH", ""),
            10,
        );
        assert_eq!(output.status.code(), Some(if valid { 0 } else { 1 }));
        assert!(output.stderr.is_empty());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains(if valid {
                    "\"invocation\":\"read\""
                } else {
                    "\"code\":\"type\""
                })
        );
    }
}

#[test]
fn closure_nesting_and_malformed_bodies_are_bounded() {
    let source = format!(
        "function main() {{ {} {} }}",
        "const f = (): void => {".repeat(200),
        "};".repeat(200)
    );
    support::accept_deep(&source);
    for source in [
        "function main() { const f = (): int => {",
        "function main() { const f = (): => {}; }",
        "function main() { const f = (): int => return 1; }",
    ] {
        reject(source, "syntax");
    }
}
