mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn arrows_return_owned_environments_from_fresh_parameters() {
    let source = r#"function main() {
 const make = (text: string): () => string => { return (): string => { return text.trim(); }; };
 const first = make(" Pat "); const second = make(" Sam "); console.log(first()); console.log(second()); console.log(first()); }"#;
    execute(source, "Pat\nSam\nPat\n", None);
    let c = check(source).unwrap().closures();
    assert_eq!(
        c.iter().find(|c| c.depth == 1).unwrap().parameters[0].access,
        Mode::Consume
    );
}

#[test]
fn callable_results_can_be_nested_and_forwarded() {
    execute(
        r#"function factory(): (text: Owned<string>) => () => string {
 return (input: string): () => string => { return (): string => { return input; }; };
}
function forward(): (value: Owned<string>) => () => string { return factory(); }
function main() { const make = forward(); const first = make("Pat"); console.log(first()); const second = make("Sam"); console.log(second()); }"#,
        "Pat\nSam\n",
        None,
    );
    execute(
        r#"function main() { const first = (): () => () => int => { return (): () => int => { return (): int => { return 7; }; }; }; const second = first(); const third = second(); console.log(third()); console.log(third()); }"#,
        "7\n7\n",
        None,
    );
}

#[test]
fn named_returns_classify_moves_before_creation() {
    let source = r#"function main() { const make = (text: string, early: boolean): () => string => {
 const result = (): string => { return text.trim(); }; if (early) { return result; } return result;
 }; const a = make("Pat", true); const b = make("Sam", false); console.log(a()); console.log(b()); }"#;
    execute(source, "Pat\nSam\n", None);
    reject(
        &source.replace("if (early)", "console.log(text); if (early)"),
        "transferred",
    );
}

#[test]
fn returning_capture_transfers_outer_invocation_and_preserves_state() {
    let source = r#"type S = { on: boolean; }; function main() { const state: S = { on: false };
 const make = (): () => boolean => { return (): boolean => { if (state.on) { state.on = false; } else { state.on = true; } return state.on; }; };
 const toggle = make(); console.log(toggle()); console.log(toggle()); }"#;
    execute(source, "true\nfalse\n", None);
    reject(
        &source.replace("const toggle = make();", "const toggle = make(); make();"),
        "closure-consumed",
    );
}

#[test]
fn recursive_signatures_and_mixed_origins_are_checked() {
    for (source, code) in [
        (
            "function main() { const f = (): () => int => { return (): boolean => { return true; }; }; }",
            "type",
        ),
        (
            "function main() { const f = (): (n: int) => int => { return (): int => { return 7; }; }; }",
            "arity",
        ),
        (
            "function main() { const f = (b: boolean): () => int => { if (b) { return (): int => { return 1; }; } return (): int => { return 2; }; }; }",
            "unsupported",
        ),
        ("function main() { const f = (): () => int => {}; }", "type"),
        (
            "function main() { const f = (): () => int => { return 7; }; }",
            "type",
        ),
    ] {
        reject(source, code);
    }
    let source = format!(
        "function main() {{ const f = (): {}int => {{}}; }}",
        "() => ".repeat(200)
    );
    reject(&source, "unsupported");
}

#[test]
fn callable_return_checks_do_not_need_backend() {
    let source = "function main() { const f = (): () => int => { return (): int => { return 7; }; }; const g = f(); console.log(g()); }";
    let dir = support::TempDirectory::new();
    let path = dir.source(source);
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}
