mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn boolean_values_copy_through_fields_calls_and_returns() {
    execute(
        r#"
type Flag = { value: boolean; };
function identity(value: boolean): boolean { const saved = value; value; return saved; }
function read(flag: Flag): boolean { return flag.value; }
function pair(first: boolean, second: boolean) { console.log(first); console.log(second); }
function update(old: boolean, flag: Flag) { flag.value = false; console.log(old); }
function unused(value: boolean) {}
function main() {
    const value: boolean = true;
    const copy = identity(value);
    pair(value, value);
    console.log(copy);
    const flag: Flag = { value: value };
    const backup = clone(flag);
    update(flag.value, flag);
    console.log(read(flag));
    console.log(backup.value);
    unused(false);
    if (flag.value) { console.log("wrong"); } else { console.log("false arm"); }
}
"#,
        "true\ntrue\ntrue\ntrue\nfalse\ntrue\nfalse arm\n",
        Some(&[
            ("identity.value", Mode::Copy),
            ("read.flag", Mode::Read),
            ("pair.first", Mode::Copy),
            ("pair.second", Mode::Copy),
            ("update.old", Mode::Copy),
            ("update.flag", Mode::Mutate),
            ("unused.value", Mode::Copy),
        ]),
    );
}

#[test]
fn both_runtime_arms_preserve_read_and_mutate_contracts() {
    let rust = execute(
        include_str!("fixtures/control-flow/branch_reuse.tsr"),
        "Pat\nPat\nSam\nSam\n",
        Some(&[("show.user", Mode::Mutate), ("show.change", Mode::Copy)]),
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn mutually_exclusive_moves_and_all_path_returns() {
    execute(
        r#"
type User = { name: string; };
function outer(user: User, first: boolean): User { return choose(user, first); }
function choose(user: User, first: boolean): User {
    if (first) { return user; } else { user.name = "Sam"; return user; }
}
function number(first: boolean): int { if (first) { return 1; } else { return 2; } }
function main() {
    const first = outer({ name: "Pat" }, true);
    const second = outer({ name: "Pat" }, false);
    console.log(first.name);
    console.log(second.name);
    console.log(number(true));
    console.log(number(false));
}
"#,
        "Pat\nSam\n1\n2\n",
        Some(&[
            ("outer.user", Mode::Consume),
            ("outer.first", Mode::Copy),
            ("choose.user", Mode::Consume),
            ("choose.first", Mode::Copy),
            ("number.first", Mode::Copy),
        ]),
    );
}

#[test]
fn returning_paths_do_not_poison_the_continuation() {
    execute(
        r#"
type User = { name: string; };
function thenExit(user: User, stop: boolean) {
    if (stop) { const saved = user; console.log(saved.name); return; }
    console.log(user.name);
}
function elseExit(user: User, keep: boolean) {
    if (keep) { console.log("keep"); } else { user; return; }
    console.log(user.name);
}
function nested(user: User, first: boolean, second: boolean) {
    if (first) { if (second) { user; return; } }
    console.log(user.name);
}
function main() {
    thenExit({ name: "then" }, true);
    thenExit({ name: "continue" }, false);
    elseExit({ name: "else" }, false);
    elseExit({ name: "continue" }, true);
    nested({ name: "a" }, true, true);
    nested({ name: "b" }, true, false);
    nested({ name: "c" }, false, true);
}
"#,
        "then\ncontinue\nkeep\ncontinue\nb\nc\n",
        Some(&[
            ("thenExit.user", Mode::Consume),
            ("thenExit.stop", Mode::Copy),
            ("elseExit.user", Mode::Consume),
            ("elseExit.keep", Mode::Copy),
            ("nested.user", Mode::Consume),
            ("nested.first", Mode::Copy),
            ("nested.second", Mode::Copy),
        ]),
    );
}

#[test]
fn branch_locals_have_distinct_bindings_and_lexical_scopes() {
    execute(
        r#"
type User = { name: string; };
function scopes(first: boolean) {
    if (first) {
        const item: User = { name: "Pat" };
        item.name = "Sam";
        if (first) { const nested = item; console.log(nested.name); }
    } else {
        const item = 7;
        console.log(item);
    }
    const item = false;
    console.log(item);
}
function main() { scopes(true); scopes(false); }
"#,
        "Sam\nfalse\n7\nfalse\n",
        Some(&[("scopes.first", Mode::Copy)]),
    );
}

#[test]
fn condition_calls_run_once_and_apply_effects_before_both_arms() {
    execute(
        r#"
type User = { name: string; };
function condition(user: User, value: boolean): boolean {
    console.log("condition"); user.name = "changed"; return value;
}
function choose(user: User, value: boolean) {
    if (condition(user, value)) { console.log(user.name); } else { console.log(user.name); }
}
function main() { choose({ name: "a" }, true); choose({ name: "b" }, false); }
"#,
        "condition\nchanged\ncondition\nchanged\n",
        Some(&[
            ("condition.user", Mode::Mutate),
            ("condition.value", Mode::Copy),
            ("choose.user", Mode::Mutate),
            ("choose.value", Mode::Copy),
        ]),
    );
    for arm in [
        "if (take(user)) { console.log(user.name); }",
        "if (take(user)) {} else { console.log(user.name); }",
    ] {
        reject(
            &format!(
                r#"type User = {{ name: string; }};
function take(user: User): boolean {{ user; return true; }}
function main() {{ const user: User = {{ name: "Pat" }}; {arm} }}"#
            ),
            "transferred",
        );
    }
}

#[test]
fn possible_moves_reject_later_uses_with_transfer_locations() {
    for branch in [
        "if (flag) { const saved = user; }",
        "if (flag) {} else { const saved = user; }",
        "if (flag) { const saved = user; } else { user; }",
        "if (flag) { if (flag) { const saved = user; } }",
        "if (false) { const saved = user; }",
    ] {
        let source = format!(
            r#"type User = {{ name: string; }};
function f(user: User, flag: boolean) {{ {branch} console.log(user.name); }}
function main() {{}}"#
        );
        let error = reject(&source, "transferred");
        assert_eq!(&source[error.span.start..error.span.end], "user");
        assert_eq!(error.related.len(), 1);
        let transfer = error.related[0].0;
        assert_eq!(&source[transfer.start..transfer.end], "user");
        assert!(transfer.start < error.span.start);
        assert!(source[..transfer.start].ends_with("const saved = "));
    }
    // Contracts apply to every call, even when the supplied flag is false.
    reject(
        r#"type User = { name: string; };
function maybe(user: User, flag: boolean) { if (flag) { user; return; } }
function main() { const user: User = { name: "Pat" }; maybe(user, false); console.log(user.name); }"#,
        "transferred",
    );
}

#[test]
fn invalid_conditions_scopes_returns_and_syntax_are_rejected() {
    execute(
        r#"function main() { const local = 1; if (true) { const local = 2; console.log(local); } console.log(local); }"#,
        "2\n1\n",
        None,
    );
    for (source, code) in [
        (r#"function main() { if (console.log(true)) {} }"#, "type"),
        (r#"function main() { const x: boolean = 1; }"#, "type"),
        (r#"function main() { const x: int = true; }"#, "type"),
        (r#"function main() { const x = true; x.trim(); }"#, "type"),
        (
            r#"function f(): boolean { return 1; } function main() {}"#,
            "type",
        ),
        (
            r#"type Flag = { value: boolean; }; function main() { const x: Flag = { value: 1 }; }"#,
            "type",
        ),
        (
            r#"function main() { if (true) { const local = 1; } console.log(local); }"#,
            "name",
        ),
        (
            r#"function main() { if (true) { const local = 1; } else { console.log(local); } }"#,
            "name",
        ),
        (
            r#"function f(flag: boolean): int { if (flag) { return 1; } } function main() {}"#,
            "type",
        ),
        (
            r#"function f(): int { if (true) { return 1; } } function main() {}"#,
            "type",
        ),
        (
            r#"function f(flag: boolean): int { if (flag) { return 1; } else { return "bad"; } } function main() {}"#,
            "type",
        ),
        (
            r#"function main() { if (true) { return; } else { return; } console.log(1); }"#,
            "unsupported",
        ),
        (
            r#"function main() { if (false) { console.log(missing); } }"#,
            "name",
        ),
        (r#"function main() { if true {} }"#, "syntax"),
        (r#"function main() { if (true) const a = 1; }"#, "syntax"),
        (r#"function main() { if (true) {}; }"#, "syntax"),
        (r#"function main() { if (true) {"#, "syntax"),
        (r#"function main() { const true = false; }"#, "syntax"),
        (r#"function false() {} function main() {}"#, "syntax"),
        (r#"type boolean = {}; function main() {}"#, "duplicate"),
        (
            r#"function main() { const flag = true; flag = false; }"#,
            "unsupported",
        ),
        (
            r#"function main() { if (true == false) {} }"#,
            "unsupported",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn branch_calls_keep_existing_argument_conflict_rules() {
    for source in [
        r#"type User = { name: string; };
function both(a: User, b: User) { a.name = "Sam"; console.log(b.name); }
function main() { const user: User = { name: "Pat" }; if (true) { both(user, user); } }"#,
        r#"type Flag = { value: boolean; };
function both(flag: Flag, value: boolean) { flag.value = value; }
function main() { const flag: Flag = { value: true }; if (true) { both(flag, flag.value); } }"#,
    ] {
        let error = reject(source, "conflicting-access");
        assert_eq!(error.related.len(), 1);
    }
}

#[test]
fn branch_depth_stays_bounded_while_function_dependencies_use_worklists() {
    let deep = format!(
        "function main() {{ {} console.log(true); {} }}",
        "if (true) {".repeat(200),
        "}".repeat(200)
    );
    reject(&deep, "unsupported");
    let mut source = String::from("function main() {}\n");
    for i in 0..50 {
        source.push_str(&format!(
            "function f{i}() {{ {} f{}(); {} }}\n",
            "if (true) {".repeat(4),
            i + 1,
            "}".repeat(4)
        ));
    }
    source.push_str("function f50() {}\n");
    check(&support::format_round_trip(&source)).unwrap();
}

#[test]
fn join_matrix_distinguishes_transfers_from_exiting_paths() {
    // Each cell gives the independent expected result for the two arm actions.
    let actions = [
        "console.log(user.name);",
        "user;",
        "return;",
        "user; return;",
    ];
    let expected = [
        [None, Some("transferred"), None, None],
        [
            Some("transferred"),
            Some("transferred"),
            Some("transferred"),
            Some("transferred"),
        ],
        [
            None,
            Some("transferred"),
            Some("unsupported"),
            Some("unsupported"),
        ],
        [
            None,
            Some("transferred"),
            Some("unsupported"),
            Some("unsupported"),
        ],
    ];
    for (i, then_action) in actions.iter().enumerate() {
        for (j, else_action) in actions.iter().enumerate() {
            let source = format!(
                r#"type User = {{ name: string; }};
function f(user: User, flag: boolean) {{ if (flag) {{ {then_action} }} else {{ {else_action} }} console.log(user.name); }}
function main() {{}}"#
            );
            match (check(&source), expected[i][j]) {
                (Ok(analysis), None) => {
                    let mode = if i == 3 || j == 3 {
                        Mode::Consume
                    } else {
                        Mode::Read
                    };
                    assert_eq!(analysis.contracts()["f.user"], mode);
                }
                (Err(error), Some(code)) => assert_eq!(error.code, code, "{source}"),
                (Ok(_), Some(code)) => panic!("Expected {code}: {source}"),
                (Err(error), None) => panic!("Unexpected {}: {source}", error.message),
            }
        }
    }
}
