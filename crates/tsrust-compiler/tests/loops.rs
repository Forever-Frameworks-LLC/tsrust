mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn counted_loops_mutate_records_and_can_run_zero_times() {
    execute(include_str!("fixtures/loops/count.tsr"), "5\n9\n", None);
    execute(
        r#"type State = { n: int; };
function count(state: State, limit: int) { while (state.n < limit) { console.log(state.n); state.n = state.n + 1; } }
function main() { const state: State = { n: 0 }; count(state, 3); count(state, 2); console.log(state.n); }"#,
        "0\n1\n2\n3\n",
        Some(&[("count.state", Mode::Mutate), ("count.limit", Mode::Copy)]),
    );
}

#[test]
fn conditions_run_before_each_iteration_and_after_continue() {
    execute(
        r#"type S = { n: int; };
function next(s: S): boolean { s.n = s.n + 1; console.log(s.n); return s.n < 4; }
function main() { const s: S = { n: 0 }; while (next(s)) { if (s.n === 2) { continue; } console.log("body"); } console.log(s.n); }"#,
        "1\nbody\n2\n3\nbody\n4\n4\n",
        None,
    );
}

#[test]
fn nested_break_and_continue_target_the_nearest_loop() {
    execute(
        r#"type S = { n: int; }; function main() { const outer: S = { n: 0 };
 while (outer.n < 2) { const inner: S = { n: 0 }; while (true) { inner.n = inner.n + 1; if (inner.n === 1) { continue; } break; } console.log(inner.n); outer.n = outer.n + 1; }
 console.log(outer.n); }"#,
        "2\n2\n2\n",
        None,
    );
}

#[test]
fn inherited_transfers_on_any_back_edge_are_rejected() {
    for body in [
        "user;",
        "user; continue;",
        "if (flag) { user; continue; }",
        "if (flag) { user; }",
        "while (flag) { user; break; }",
    ] {
        let source = format!(
            "type U = {{ n: int; }}; function f(user: U, flag: boolean) {{ while (flag) {{ {body} }} }} function main() {{}}"
        );
        let error = reject(&source, "loop-transfer");
        assert_eq!(error.related.len(), 1);
    }
    reject(
        "type U = { n: int; }; function f(user: U) { while (false) { user; } } function main() {}",
        "loop-transfer",
    );
    reject(
        "type U = { n: int; }; function take(user: U): boolean { user; return false; } function f(user: U) { while (take(user)) {} } function main() {}",
        "loop-transfer",
    );
}

#[test]
fn transfers_can_exit_through_break_or_return() {
    execute(
        r#"type U = { n: int; };
function done(user: U, flag: boolean) { while (flag) { console.log(user.n); user; break; } }
function take(user: U, flag: boolean): int { while (flag) { const saved = user; return saved.n; } return user.n; }
function main() { done({ n: 7 }, true); done({ n: 8 }, false); console.log(take({ n: 9 }, true)); console.log(take({ n: 10 }, false)); }"#,
        "7\n9\n10\n",
        Some(&[
            ("done.user", Mode::Consume),
            ("done.flag", Mode::Copy),
            ("take.user", Mode::Consume),
            ("take.flag", Mode::Copy),
        ]),
    );
    execute(
        "type U = { n: int; }; function take(user: U): boolean { user; return true; } function main() { const user: U = { n: 1 }; while (take(user)) { break; } console.log(7); }",
        "7\n",
        None,
    );
}

#[test]
fn break_states_join_but_function_returns_do_not_reach_loop_exit() {
    for body in [
        "user; break;",
        "if (other) { user; break; } else { break; }",
        "if (other) { user; break; } else { return; }",
    ] {
        reject(
            &format!(
                "type U = {{ n: int; }}; function f(user: U, flag: boolean, other: boolean) {{ while (flag) {{ {body} }} console.log(user.n); }} function main() {{}}"
            ),
            "transferred",
        );
    }
    execute(
        "type U = { n: int; }; function f(user: U, flag: boolean) { while (flag) { user; return; } console.log(user.n); } function main() { f({ n: 7 }, false); f({ n: 8 }, true); }",
        "7\n",
        None,
    );
    // The condition executes even on the zero-iteration path.
    reject(
        "type U = { n: int; }; function take(user: U): boolean { user; return false; } function f(user: U) { while (take(user)) { return; } console.log(user.n); } function main() {}",
        "transferred",
    );
}

#[test]
fn iteration_locals_are_fresh_and_lexical_capture_loans_end_on_exit() {
    execute(
        r#"type S = { n: int; }; function take(text: string): string { return text; }
function main() { const s: S = { n: 0 }; while (s.n < 3) { const text = "Pat"; console.log(take(text)); const advance = (): void => { s.n = s.n + 1; }; advance(); if (true) { continue; } } console.log(s.n); }"#,
        "Pat\nPat\nPat\n3\n",
        None,
    );
    reject(
        "function main() { while (false) { const local = 7; break; } console.log(local); }",
        "name",
    );
}

#[test]
fn loops_support_reusable_callbacks_but_reject_repeated_once_calls() {
    execute(
        r#"type S = { n: int; };
function repeat(action: (n: int) => int, state: S) { while (state.n < 3) { console.log(action(state.n)); state.n = state.n + 1; } }
function main() { const state: S = { n: 0 }; repeat((n: int): int => { return n * 2; }, state); console.log(state.n); }"#,
        "0\n2\n4\n3\n",
        None,
    );
    reject(
        "function bad(action: Once<Function<boolean>>) { while (action()) {} } function main() {}",
        "loop-transfer",
    );
    reject(
        "function bad(action: Once<Function<int>>) { while (true) { action(); continue; } } function main() {}",
        "loop-transfer",
    );
    execute(
        "function once(action: Once<Function<int>>) { while (true) { console.log(action()); break; } } function main() { once((): int => { return 7; }); }",
        "7\n",
        None,
    );
}

#[test]
fn loop_capture_effects_propagate_to_closure_and_retained_contracts() {
    execute(
        r#"type S = { n: int; };
function retain(action: Function<int, [limit: int]>): (limit: int) => int { return (limit: int): int => { return action(limit); }; }
function main() { const state: S = { n: 0 }; const advance = retain((limit: int): int => { while (state.n < limit) { state.n = state.n + 1; } return state.n; }); console.log(advance(3)); console.log(advance(5)); }"#,
        "3\n5\n",
        None,
    );
    let source = "function main() { const text = \"Pat\"; const action = (flag: boolean): string => { while (flag) { return text; } return \"Sam\"; }; console.log(action(true)); }";
    execute(source, "Pat\n", None);
    assert_eq!(
        check(source).unwrap().closures()[0].invocation,
        Mode::Consume
    );
}

#[test]
fn callable_returns_inside_loops_classify_owned_captures_before_creation() {
    execute(
        r#"function make(text: string, flag: boolean): () => string { const f = (): string => { return text.trim(); }; while (flag) { return f; } return f; }
function main() { const first = make(" Pat ", true); const second = make(" Sam ", false); console.log(first()); console.log(second()); }"#,
        "Pat\nSam\n",
        None,
    );
    reject(
        "function make(text: string, flag: boolean): () => string { while (flag) { return (): string => { return text; }; } return (): string => { return \"Sam\"; }; } function main() {}",
        "unsupported",
    );
}

#[test]
fn invalid_control_flow_and_return_coverage_fail_in_frontend() {
    for source in [
        "function main() { break; }",
        "function main() { continue; }",
        "function main() { while (true) { const f = (): void => { break; }; break; } }",
        "function helper() { continue; } function main() { while (false) { helper(); } }",
    ] {
        reject(source, "loop-control");
    }
    for source in [
        "function main() { while (true) { break; console.log(1); } }",
        "function main() { while (true) { if (true) { break; } else { continue; } console.log(1); } }",
    ] {
        reject(source, "unsupported");
    }
    reject(
        "function f(): int { while (true) { return 7; } } function main() {}",
        "type",
    );
    // A number condition selects by truthiness: zero ends the loop.
    support::execute("function main() { let turns = 2; while (turns) { turns -= 1; } console.log(turns); }", "0\n", None);
    reject("function main() { while (true) let a = 1; }", "syntax");
    reject(
        "function main() { while (true) { break outer; } }",
        "loop-control",
    );
}

#[test]
fn loop_nesting_and_combined_analysis_remain_bounded() {
    let source = format!(
        "function main() {{ {} {} }}",
        "while (false) {".repeat(200),
        "}".repeat(200)
    );
    support::accept_deep(&source);
    let source = format!(
        "function main() {{ {} {} }}",
        "while (false) {".repeat(40),
        "}".repeat(40)
    );
    check(&source).unwrap().emit_rust();
}

#[test]
fn loop_checks_are_backend_independent() {
    let dir = support::TempDirectory::new();
    let path=dir.source("type S = { n: int; }; function main() { const s: S = { n: 0 }; while (s.n < 2) { s.n = s.n + 1; } }");
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}

#[test]
fn branch_exit_matrix_distinguishes_back_edges_breaks_and_returns() {
    let actions = [
        "console.log(user.n);",
        "continue;",
        "break;",
        "user; break;",
        "user; return;",
        "user;",
    ];
    let mut accepted = String::from("type U = { n: int; };\n");
    let mut calls = String::new();
    let mut output = String::new();
    for (i, left) in actions.iter().enumerate() {
        for (j, right) in actions.iter().enumerate() {
            let function = format!(
                "function f{i}_{j}(user: U, flag: boolean, branch: boolean) {{ while (flag) {{ if (branch) {{ {left} }} else {{ {right} }} }} console.log(user.n); }}"
            );
            let source = format!("type U = {{ n: int; }}; {function} function main() {{}}");
            if i == 5 || j == 5 {
                reject(&source, "loop-transfer");
            } else if i == 3 || j == 3 {
                reject(&source, "transferred");
            } else {
                check(&source).unwrap();
                accepted.push_str(&function);
                calls.push_str(&format!("f{i}_{j}({{ n: 7 }}, false, true);"));
                output.push_str("7\n");
            }
        }
    }
    accepted.push_str(&format!("function main() {{ {calls} }}"));
    execute(&accepted, &output, None);
}
