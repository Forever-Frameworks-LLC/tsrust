mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn read_and_mutable_callable_captures_preserve_access() {
    execute(
        r#"type U = { name: string; };
function twice(f: Mutable<Function<void>>) { f(); f(); }
function main() {
 const text = "Pat"; const read = (): void => { console.log(text); };
 const wrapper = (): void => { twice(read); }; wrapper(); wrapper();
 const user: U = { name: "old" };
 if (true) { const change = (): void => { user.name = "Sam"; };
   const wrapped = (): void => { change(); }; wrapped(); wrapped(); }
 console.log(user.name);
}"#,
        "Pat\nPat\nPat\nPat\nSam\n",
        None,
    );
}

#[test]
fn consuming_callable_moves_through_wrapper_and_remains_single_use() {
    let source = r#"function main() { const text = "Pat";
 const take = (): string => { return text; };
 const wrapper = (): string => { return take(); }; console.log(wrapper()); }"#;
    execute(source, "Pat\n", None);
    assert!(
        check(source)
            .unwrap()
            .closures()
            .iter()
            .all(|c| c.invocation == Mode::Consume)
    );
    reject(
        &source.replace(
            "console.log(wrapper());",
            "console.log(wrapper()); wrapper();",
        ),
        "closure-consumed",
    );
    reject(
        &source.replace("console.log(wrapper());", "take();"),
        "closure-consumed",
    );
}

#[test]
fn owned_wrappers_escape_but_borrowed_environments_do_not() {
    execute(
        r#"function make(text: string): () => string { return (): string => { return text.trim(); }; }
function wrap(text: string): () => string { const inner = make(text); return (): string => { return inner(); }; }
function main() { const f = wrap(" Pat "); console.log(f()); console.log(f()); }"#,
        "Pat\nPat\n",
        None,
    );
    reject(
        r#"function bad(text: string): () => string { const inner = (): string => { return text.trim(); }; return (): string => { return inner(); }; } function main() {}"#,
        "closure-escape",
    );
}

#[test]
fn callable_capture_loans_conflict_until_scope_exit() {
    let prefix = "type U = { name: string; }; function main() { const user: U = { name: \"Pat\" }; const change = (): void => { user.name = \"Sam\"; }; ";
    reject(
        &format!("{prefix}const wrapper = (): void => {{ change(); }}; change(); }}"),
        "capture-conflict",
    );
    execute(
        &format!(
            "{prefix}if (true) {{ const wrapper = (): void => {{ change(); }}; wrapper(); }} change(); }}"
        ),
        "",
        None,
    );
    reject(
        "function take(f: Once<Function<void>>) { f(); } function main() { const f = (): void => {}; const wrapper = (): void => { f(); }; take(f); }",
        "capture-conflict",
    );
}

#[test]
fn nested_owned_borrowed_callable_fields_lower_correctly() {
    execute(
        r#"function take(f: Mutable<Function<void>>) { f(); }
function main() { const text = "Pat"; const read = (): void => { console.log(text); };
 const consume = (): void => { take(read); }; const wrap = (): void => { consume(); }; wrap(); }"#,
        "Pat\n",
        None,
    );
    reject(
        r#"function take(f: Mutable<Function<void>>) { f(); }
function bad(text: string): () => void { const read = (): void => { console.log(text); }; const consume = (): void => { take(read); }; return (): void => { consume(); }; } function main() {}"#,
        "closure-escape",
    );
}

#[test]
fn argument_contracts_survive_callable_capture() {
    execute(
        r#"type U = { name: string; };
function main() { const change = (user: U): string => { user.name = "Sam"; return user.name.trim(); };
 const wrap = (input: U): string => { return change(input); }; const user: U = { name: "Pat" };
 console.log(wrap(user)); console.log(user.name); }"#,
        "Sam\nSam\n",
        None,
    );
}
