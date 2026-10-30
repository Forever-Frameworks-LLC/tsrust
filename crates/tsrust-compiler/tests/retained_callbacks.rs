mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn retained_reusable_callbacks_own_state_and_remain_independent() {
    let source = r#"type State = { on: boolean; };
function retain(action: Function<boolean>): () => boolean { return (): boolean => { return action(); }; }
function make(state: State): () => boolean { return retain((): boolean => { if (state.on) { state.on = false; } else { state.on = true; } return state.on; }); }
function main() { const first = make({ on: false }); const second = make({ on: true }); console.log(first()); console.log(first()); console.log(second()); }"#;
    execute(source, "true\nfalse\nfalse\n", None);
    let analysis = check(source).unwrap();
    assert_eq!(analysis.contracts()["retain.action"], Mode::Consume);
    assert_eq!(analysis.contracts()["make.state"], Mode::Consume);
    assert!(analysis.closures().iter().any(|c| c.template));
    assert!(analysis.to_json().contains("\"template\":true"));
    let rust = analysis.emit_rust();
    assert!(!rust.contains("Box<"));
    assert!(!rust.contains("dyn "));
}

#[test]
fn retained_once_callbacks_transfer_their_capture_and_allow_one_invocation() {
    let source = r#"function retain(action: Once<Function<string>>): () => string { return (): string => { return action(); }; }
function main() { const text = "Pat"; const callback = retain((): string => { return text; }); console.log(callback()); }"#;
    execute(source, "Pat\n", None);
    reject(
        &source.replace(
            "console.log(callback());",
            "console.log(callback()); console.log(callback());",
        ),
        "closure-consumed",
    );
    reject(
        &source.replace("console.log(callback());", "console.log(text);"),
        "transferred",
    );
}

#[test]
fn owned_callbacks_forward_through_factories_and_support_owned_arguments() {
    execute(
        r#"type User = { name: string; };
function retain(action: Function<string, [user: Mutable<User>]>): (user: Mutable<User>) => string { return (user: User): string => { return action(user); }; }
function forward(action: Function<string, [user: Mutable<User>]>): (user: Mutable<User>) => string { return retain(action); }
function main() { const prefix = "Pat"; const callback = forward((user: User): string => { user.name = prefix.trim(); return user.name.trim(); }); const user: User = { name: "Sam" }; console.log(callback(user)); console.log(user.name); }"#,
        "Pat\nPat\n",
        None,
    );
    execute(
        r#"function retain(action: Once<Function<string, [text: Owned<string>]>>): (text: Owned<string>) => string { return (text: string): string => { return action(text); }; }
function main() { const callback = retain((text: string): string => { return text; }); console.log(callback("Pat")); }"#,
        "Pat\n",
        None,
    );
}

#[test]
fn owned_callbacks_reject_borrows_mismatches_and_reuse_after_transfer() {
    let source = r#"function retain(action: Function<string>): () => string { return (): string => { return action(); }; }
function main() { const text = "Pat"; const action = (): string => { return text.trim(); }; const callback = retain(action); console.log(callback()); }"#;
    reject(source, "callback-escape");
    reject(
        &source
            .replace(
                "const action = (): string => { return text.trim(); };",
                "const action = (): string => { return \"Pat\"; };",
            )
            .replace("console.log(callback());", "console.log(action());"),
        "closure-consumed",
    );
    reject(
        &source
            .replace("const action = (): string => { return text.trim(); };", "")
            .replace(
                "const callback = retain(action);",
                "const callback = retain((): string => { return text; });",
            ),
        "callback-once",
    );
    reject(
        "function bad(action: Mutable<Function<string>>): () => string { return (): string => { return action(); }; } function main() {}",
        "callback-escape",
    );
    reject(
        "function bad(action: Mutable<Function<string>>): () => string { return (): string => { return action(); }; } function main() {}",
        "callback-escape",
    );
    reject(
        "function retain(action: Function<int>): () => int { return (): int => { return action(); }; } function main() { retain((): boolean => { return true; }); }",
        "type",
    );
    reject(
        "function retain(action: Function<int>) {} function bad(action: Mutable<Function<int>>) { retain(action); } function main() {}",
        "callback-contract",
    );
    execute(
        "function use(action: Mutable<Function<int>>):int{return action();} function forward(action: () => int):int{const first=use(action);return first+action();}function main(){let value=0;console.log(forward(():int=>{value=value+1;return value;}));}",
        "3\n",
        None,
    );
}

#[test]
fn template_bodies_are_validated_even_when_unused() {
    reject(
        "function bad(action: Once<Function<string>>): () => string { return (): string => { action(); return action(); }; } function main() {}",
        "callback-consumed",
    );
    execute(
        "function move(action: Function<int>) { const next = action; console.log(next()); } function main() { move((): int => { return 7; }); }",
        "7\n",
        None,
    );
    execute(
        "function identity(action: Function<int>): () => int { return action; } function main() { const f = identity((): int => { return 7; }); console.log(f()); }",
        "7\n",
        None,
    );
    execute(
        "function unused(action: Function<int>): () => int { return (): int => { return action(); }; } function main() { console.log(7); }",
        "7\n",
        None,
    );
}

#[test]
fn specialization_cache_is_deterministic_without_a_count_limit() {
    let source = "function retain(action: Function<int>): () => int { return (): int => { return action(); }; } function make(): () => int { return (): int => { return 7; }; } function main() { const first = retain(make()); const second = retain(make()); console.log(first()); console.log(second()); }";
    execute(source, "7\n7\n", None);
    let analysis = check(source).unwrap();
    assert_eq!(
        analysis
            .closures()
            .iter()
            .filter(|c| c.function == "retain" && !c.template)
            .count(),
        1
    );
    assert_eq!(analysis.emit_rust(), check(source).unwrap().emit_rust());
    let calls = (0..129)
        .map(|_| "retain((): int => { return 7; });")
        .collect::<String>();
    // Like rustc, distinct closure origins instantiate without a count limit.
    check(&format!("function retain(action: Function<int>) {{}} function main() {{ {calls} }}"))
        .unwrap();
}

#[test]
fn owned_contracts_transfer_unused_arguments_and_preserve_contextual_names() {
    reject(
        "function drop(action: Function<int>) {} function main() { const action = (): int => { return 7; }; drop(action); action(); }",
        "closure-consumed",
    );
    execute(
        "type owned = { on: boolean; }; function read(value: owned): boolean { return value.on; } function main() { console.log(read({ on: true })); }",
        "true\n",
        None,
    );
    let source = "function retain(action: Function<int>): () => int { return (): int => { return action(); }; } function main() { const f = retain((): int => { return 7; }); console.log(f()); }";
    let dir = support::TempDirectory::new();
    let path = dir.source(source);
    for command in ["check", "emit"] {
        let output = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&output);
    }
}

#[test]
fn retained_once_contract_stays_single_use_for_reusable_actuals() {
    let source = "function retain(action: Once<Function<int>>): () => int { return (): int => { return action(); }; } function main() { const f = retain((): int => { return 7; }); console.log(f()); }";
    execute(source, "7\n", None);
    reject(
        &source.replace("console.log(f());", "console.log(f()); f();"),
        "closure-consumed",
    );
}

#[test]
fn retained_callbacks_compose_multiple_environments_and_local_loans() {
    execute(
        r#"function combine(left: Function<int>, right: Once<Function<string>>): () => string {
 if (true) { const local = (): void => { console.log(left()); }; local(); }
 return (): string => { console.log(left()); return right(); };
}
function main() { const f = combine((): int => { return 7; }, (): string => { return "Pat"; }); console.log(f()); }"#,
        "7\n7\nPat\n",
        None,
    );
    execute(
        r#"function retain(action: Function<int>, early: boolean): () => int {
 const wrapped = (): int => { return action(); }; if (early) { return wrapped; } return wrapped;
}
function main() { const first = retain((): int => { return 7; }, true); const second = retain((): int => { return 8; }, false); console.log(first()); console.log(second()); }"#,
        "7\n8\n",
        None,
    );
}

#[test]
fn retained_capture_transfers_obey_argument_order_and_branch_joins() {
    reject(
        "function drop(action: Function<string>, text: string) { console.log(text); } function main() { const text = \"Pat\"; drop((): string => { return text.trim(); }, text); }",
        "transferred",
    );
    reject(
        "function drop(text: string, action: Function<string>) { console.log(text); } function main() { const text = \"Pat\"; drop(text, (): string => { return text.trim(); }); }",
        "conflicting-access",
    );
    reject(
        "function bad(action: Function<int>, early: boolean): () => int { if (early) { const f = (): int => { return action(); }; return f; } const g = (): int => { return action(); }; return g; } function main() {}",
        "unsupported",
    );
    reject(
        "function drop(action: Function<int>) {} function bad(action: Function<int>, b: boolean) { if (b) { drop(action); } action(); } function main() {}",
        "callback-consumed",
    );
}
