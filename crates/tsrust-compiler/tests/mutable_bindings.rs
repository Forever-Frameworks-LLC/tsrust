mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn mutable_scalars_support_loop_updates_and_branch_reassignment() {
    execute(
        r#"function main() { let count = 0; let enabled: boolean = true;
 while (enabled) { count = count + 1; if (count < 3) { continue; } enabled = false; }
 console.log(count); console.log(enabled); count = count; console.log(count);
}"#,
        "3\nfalse\n3\n",
        None,
    );
}

#[test]
fn strings_and_records_reinitialize_after_transfer() {
    execute(
        r#"type U = { name: string; };
function take(text: string): string { return text; }
function main() { let text = "Pat"; const first = take(text); text = "Sam"; console.log(first); console.log(text); text = take(text); console.log(text);
 let user: U = { name: "Pat" }; const original = user; user = { name: "Sam" }; console.log(original.name); console.log(user.name); user = clone(original); console.log(user.name); }
"#,
        "Pat\nSam\nSam\nPat\nSam\nPat\n",
        Some(&[("take.text", Mode::Consume)]),
    );
}

#[test]
fn replacement_evaluates_rhs_once_before_committing() {
    execute(
        r#"function change(text: string): string { console.log(text); return "Sam"; }
function main() { let text = "Pat"; text = change(text); console.log(text); text = text.trim(); console.log(text); }"#,
        "Pat\nSam\nSam\n",
        None,
    );
    reject(
        "function main() { let text = \"Pat\"; const moved = text; text = text.trim(); }",
        "transferred",
    );
    reject(
        "type U = { n: int; }; function main() { let u: U = { n: 1 }; const moved = u; u.n = 2; }",
        "transferred",
    );
}

#[test]
fn consume_and_replace_can_repeat_on_all_continuing_edges() {
    execute(
        include_str!("fixtures/mutable-bindings/reinitialize.tsr"),
        "Pat\nSam\nSam\nSam\n",
        None,
    );
    execute(
        r#"function take(text: string): string { return text; }
function main() { let n = 0; let text = "Pat"; while (n < 3) { console.log(take(text)); text = "Sam"; n = n + 1; if (n < 3) { continue; } } console.log(text); }"#,
        "Pat\nSam\nSam\nSam\n",
        None,
    );
    for body in [
        "text;",
        "text; continue;",
        "text; if (flag) { text = \"Sam\"; }",
    ] {
        reject(
            &format!(
                "function main() {{ let text = \"Pat\"; const flag = true; while (flag) {{ {body} }} }}"
            ),
            "loop-transfer",
        );
    }
}

#[test]
fn branch_reinitialization_requires_every_continuing_path() {
    let prefix = "function f(flag: boolean) { let text = \"Pat\"; const moved = text; ";
    execute(
        &format!(
            "{prefix} if (flag) {{ text = \"Sam\"; }} else {{ text = \"Alex\"; }} console.log(text); }} function main() {{ f(true); f(false); }}"
        ),
        "Sam\nAlex\n",
        None,
    );
    reject(
        &format!(
            "{prefix} if (flag) {{ text = \"Sam\"; }} console.log(text); }} function main() {{}}"
        ),
        "transferred",
    );
    execute(
        &format!(
            "{prefix} if (flag) {{ text = \"Sam\"; }} else {{ return; }} console.log(text); }} function main() {{ f(true); f(false); }}"
        ),
        "Sam\n",
        None,
    );
}

#[test]
fn loop_exits_preserve_zero_iteration_and_condition_effects() {
    reject(
        "function main() { let text = \"Pat\"; text; while (true) { text = \"Sam\"; break; } console.log(text); }",
        "transferred",
    );
    reject(
        "function take(text: string): boolean { text; return false; } function main() { let text = \"Pat\"; while (take(text)) { text = \"Sam\"; } console.log(text); }",
        "transferred",
    );
    execute(
        "function main() { let text = \"Pat\"; while (true) { text; text = \"Sam\"; break; } console.log(text); }",
        "Sam\n",
        None,
    );
    reject(
        "function main() { let text = \"Pat\"; while (true) { text; break; } console.log(text); }",
        "transferred",
    );
    execute(
        "function main() { let n = 0; let text = \"unused\"; text; while (n < 2) { text = \"Pat\"; console.log(text); text; n = n + 1; } text = \"Sam\"; console.log(text); }",
        "Pat\nPat\nSam\n",
        None,
    );
}

#[test]
fn replacement_respects_existing_capture_loans() {
    for closure in [
        "(): string => { return text.trim(); }",
        "(): void => { text.trim(); }",
    ] {
        let source = format!(
            "function main() {{ let text = \"Pat\"; const read = {closure}; read(); text = \"Sam\"; }}"
        );
        let error = reject(&source, "capture-conflict");
        assert_eq!(error.related.len(), 1);
    }
    execute(
        "function main() { let text = \"Pat\"; if (true) { const read = (): string => { return text.trim(); }; console.log(read()); } text = \"Sam\"; console.log(text); }",
        "Pat\nSam\n",
        None,
    );
    reject(
        "type U = { n: int; }; function main() { let u: U = { n: 1 }; const update = (): void => { u.n = 2; }; u = { n: 3 }; }",
        "capture-conflict",
    );
}

#[test]
fn owned_capture_and_scalar_capture_preserve_existing_storage_semantics() {
    execute(
        r#"function make(): () => string { let text = "Pat"; const f = (): string => { return text.trim(); }; text = "Sam"; console.log(text); return f; }
function main() { let n = 1; const read = (): int => { return n; }; n = 2; console.log(read()); console.log(n); const f = make(); console.log(f()); }
"#,
        "1\n2\nSam\nPat\n",
        None,
    );
}

#[test]
fn closures_can_rebind_their_own_fresh_locals() {
    let source = "function main() { const f = (limit: int): int => { let n = 0; while (n < limit) { n = n + 1; } return n; }; console.log(f(3)); console.log(f(5)); }";
    execute(source, "3\n5\n", None);
    assert_eq!(check(source).unwrap().closures()[0].invocation, Mode::Read);
    for source in [
        "function main() { const n = 0; const f = (): void => { n = 1; }; }",
        "function main() { const text = \"Pat\"; const f = (): void => { text = \"Sam\"; }; }",
    ] {
        reject(source, "unsupported");
    }
}

#[test]
fn declarations_and_replacements_keep_exact_types_and_const_rules() {
    for source in [
        "function main() { const n = 1; n = 2; }",
        "function f(n: int) { n = 2; } function main() {}",
        "function main() { const f = (n: int): void => { n = 2; }; }",
    ] {
        reject(source, "unsupported");
    }
    for source in [
        "function main() { let n = 1; n = true; }",
        "function main() { let text = \"Pat\"; text = 1; }",
        "type A = { n: int; }; type B = { n: int; }; function main() { let a: A = { n: 1 }; const b: B = { n: 2 }; a = b; }",
    ] {
        reject(source, "type");
    }
    // A declaration without an initializer is accepted; reading it first is not.
    reject("function main() { let n: int; console.log(n); }", "unassigned");
    reject("function main() { const n: int; }", "syntax");
    reject("function main() { let n = 1; let n = 2; }", "duplicate");
    reject("function main() { absent = 1; }", "name");
}

#[test]
fn mutable_binding_checks_remain_backend_independent() {
    let dir = support::TempDirectory::new();
    let path =
        dir.source("function main() { let n = 0; while (n < 2) { n = n + 1; } console.log(n); }");
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}
