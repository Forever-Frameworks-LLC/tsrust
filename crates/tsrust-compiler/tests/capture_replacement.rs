mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn local_scalar_writes_borrow_exclusively_and_persist() {
    let source = r#"function main() { let n = 0; let on = false;
 if (true) { const tick = (): int => { n = n + 1; on = !on; console.log(on); return n; }; console.log(tick()); console.log(tick()); }
 console.log(n); console.log(on); }"#;
    execute(source, "true\n1\nfalse\n2\n2\nfalse\n", None);
    let c = &check(source).unwrap().closures()[0];
    assert_eq!(c.invocation, Mode::Mutate);
    assert!(
        c.captures
            .iter()
            .all(|c| c.storage == Mode::Mutate && c.access == Mode::Mutate)
    );
    execute(
        "function twice(action: Mutable<Function<int>>) { console.log(action()); console.log(action()); } function main() { let n = 0; twice((): int => { n = n + 1; return n; }); console.log(n); n = 7; console.log(n); }",
        "1\n2\n2\n7\n",
        None,
    );
}

#[test]
fn captured_owned_values_can_be_replaced_through_mutable_borrows() {
    execute(
        r#"type U = { name: string; };
function main() { let text = " Pat "; let user: U = { name: "Pat" };
 if (true) { const replace = (): void => { text = text.trim(); user = { name: text.toLowerCase() }; }; replace(); replace(); }
 console.log(text); console.log(user.name); }"#,
        "Pat\npat\n",
        None,
    );
}

#[test]
fn write_only_captures_are_not_lost_and_types_remain_exact() {
    let source = "function main() { let n = 1; if (true) { const set = (): void => { n = 7; }; set(); } console.log(n); }";
    execute(source, "7\n", None);
    assert_eq!(check(source).unwrap().closures()[0].captures[0].name, "n");
    reject(
        "function main() { let n = 1; const set = (): void => { n = true; }; }",
        "type",
    );
    reject(
        "function main() { const n = 1; const set = (): void => { n = 7; }; }",
        "unsupported",
    );
    reject(
        "function f(n: int) { const set = (): void => { n = 7; }; } function main() {}",
        "unsupported",
    );
}

#[test]
fn escaping_scalar_writes_update_a_private_copied_environment() {
    execute(
        include_str!("fixtures/capture-replacement/counter.tsr"),
        "2\n5\n1\n",
        None,
    );
    let source = r#"function factory(): (step: int) => int { let n = 0; return (step: int): int => { n = n + step; return n; }; }
function main() { const first = factory(); const second = factory(); console.log(first(2)); console.log(first(3)); console.log(second(1)); }"#;
    execute(source, "2\n5\n1\n", None);
    let c = &check(source).unwrap().closures()[0];
    assert_eq!(c.invocation, Mode::Mutate);
    assert_eq!(c.captures[0].storage, Mode::Copy);
    execute(
        "function main() { let n = 1; if (true) { const read = (): int => { return n; }; n = 2; console.log(read()); } console.log(n); }",
        "1\n2\n",
        None,
    );
}

#[test]
fn retained_scalar_and_owned_replacements_preserve_state() {
    execute(
        r#"function retain(action: Function<string>): () => string { return (): string => { return action(); }; }
function main() { let n = 0; let text = " Pat "; const f = retain((): string => { n = n + 1; text = text.trim(); console.log(n); return text.toLowerCase(); }); n = 8; text = "Sam"; console.log(f()); console.log(f()); console.log(n); console.log(text); }"#,
        "1\npat\n2\npat\n8\nSam\n",
        None,
    );
}

#[test]
fn nested_writes_propagate_and_scalar_copies_dereference_correctly() {
    execute(
        r#"function main() { let n = 0; if (true) { const outer = (): void => {
 const inner = (): void => { n = n + 1; }; inner(); }; outer(); outer(); } console.log(n); }
"#,
        "2\n",
        None,
    );
    execute(
        r#"function factory(): () => int { let n = 0; return (): int => { if (true) { const inner = (): void => { n = n + 1; }; inner(); } const read = (): int => { return n; }; return read(); }; }
function main() { const tick = factory(); console.log(tick()); console.log(tick()); }"#,
        "1\n2\n",
        None,
    );
}

#[test]
fn replacing_borrowed_captures_conflicts_with_outer_reads_writes_and_loans() {
    for use_after in [
        "console.log(n);",
        "n = 2;",
        "const read = (): int => { return n; };",
        "const other = (): void => { n = 3; };",
    ] {
        let error = reject(
            &format!(
                "function main() {{ let n = 1; const set = (): void => {{ n = 7; }}; {use_after} }}"
            ),
            "capture-conflict",
        );
        assert_eq!(error.related.len(), 1);
    }
    reject(
        "function main() { let text = \"Pat\"; const f = (): void => { const read = (): string => { return text.trim(); }; text = \"Sam\"; }; }",
        "capture-conflict",
    );
    reject(
        "function main() { let text = \"Pat\"; text; const set = (): void => { text = \"Sam\"; }; }",
        "transferred",
    );
}

#[test]
fn consuming_rhs_keeps_environment_single_use_even_when_restored() {
    let source = r#"function take(text: string): string { return text; }
function main() { let text = "Pat"; const f = (): string => { text = take(text); return text.trim(); }; console.log(f()); }"#;
    execute(source, "Pat\n", None);
    assert_eq!(
        check(source).unwrap().closures()[0].invocation,
        Mode::Consume
    );
    reject(
        &source.replace("console.log(f());", "console.log(f()); f();"),
        "closure-consumed",
    );
    execute(
        "function main() { let n = 0; if (true) { const text = \"Pat\"; const f = (): string => { n = n + 1; console.log(n); return text; }; console.log(f()); } console.log(n); }",
        "1\nPat\n1\n",
        None,
    );
    execute(
        r#"function make(): () => string { let n = 0; const text = "Pat"; return (): string => { n = n + 1; console.log(n); return text; }; } function main() { const f = make(); console.log(f()); }"#,
        "1\nPat\n",
        None,
    );
}

#[test]
fn replacement_and_loop_back_edges_compose_inside_closures() {
    execute(
        "function main() { let n = 0; if (true) { const f = (): void => { while (n < 3) { n = n + 1; } }; f(); f(); } console.log(n); }",
        "3\n",
        None,
    );
    execute(
        "function main() { let n = 0; while (n < 3) { const f = (): void => { n = n + 1; }; f(); continue; } console.log(n); }",
        "3\n",
        None,
    );
    reject(
        "function take(text: string): string { return text; } function main() { let text = \"Pat\"; const f = (): void => { while (true) { text = take(text); break; } }; while (true) { f(); } }",
        "loop-transfer",
    );
}

#[test]
fn capture_replacement_checking_is_backend_independent() {
    let dir = support::TempDirectory::new();
    let path=dir.source("function main() { let n = 0; const f = (): int => { n = n + 1; return n; }; console.log(f()); }");
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}
