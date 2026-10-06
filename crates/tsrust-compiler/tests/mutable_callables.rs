mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

const READ_FACTORY: &str =
    "function make(text: string): () => string { return (): string => { return text.trim(); }; }";
const ONCE_FACTORY: &str =
    "function make(text: string): () => string { return (): string => { return text; }; }";

#[test]
fn owned_callable_replacement_keeps_concrete_type_and_new_state() {
    let source = format!(
        "{READ_FACTORY} function main() {{ let f = make(\" Pat \"); console.log(f()); f = make(\" Sam \"); console.log(f()); console.log(f()); }}"
    );
    let rust = execute(&source, "Pat\nSam\nSam\n", None);
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
    execute(
        "function main() { const make = (text: string): () => string => { return (): string => { return text.trim(); }; }; let f = make(\"Pat\"); f = make(\"Sam\"); console.log(f()); }",
        "Sam\n",
        None,
    );
}

#[test]
fn single_use_environments_reinitialize_after_calls_and_forwarding() {
    let source = format!(
        "{ONCE_FACTORY} function discard(action: Once<Function<string>>) {{}} function main() {{ let f = make(\"Pat\"); console.log(f()); f = make(\"Sam\"); discard(f); f = make(\"Alex\"); console.log(f()); }}"
    );
    execute(&source, "Pat\nAlex\n", None);
    reject(
        &format!("{ONCE_FACTORY} function main() {{ let f = make(\"Pat\"); f(); f = f; }}"),
        "closure-consumed",
    );
}

#[test]
fn replacement_can_invoke_the_old_environment_before_committing() {
    execute(
        &format!(
            "{ONCE_FACTORY} function main() {{ let f = make(\"Pat\"); f = make(f()); console.log(f()); }}"
        ),
        "Pat\n",
        None,
    );
    execute(
        &format!(
            "{READ_FACTORY} function main() {{ let f = make(\"Pat\"); f = f; console.log(f()); }}"
        ),
        "Pat\n",
        None,
    );
}

#[test]
fn named_replacement_moves_the_source_without_creating_an_alias() {
    let source = format!(
        "{READ_FACTORY} function main() {{ let first = make(\"Pat\"); const second = make(\"Sam\"); first = second; console.log(first()); }}"
    );
    execute(&source, "Sam\n", None);
    reject(
        &source.replace("console.log(first());", "console.log(second());"),
        "closure-consumed",
    );
    execute(
        &format!(
            "{READ_FACTORY} function main() {{ let first = make(\"Pat\"); let second = first; console.log(second()); }}"
        ),
        "Pat\n",
        None,
    );
}

#[test]
fn borrowed_mutable_callables_preserve_lexical_capture_loans() {
    execute(
        "function main() { let n = 0; if (true) { let f = (): void => { n = n + 1; }; f(); f = f; f(); } console.log(n); }",
        "2\n",
        None,
    );
    reject(
        "function main() { let n = 0; let f = (): void => { n = n + 1; }; f = f; console.log(n); }",
        "capture-conflict",
    );
    let source = format!(
        "{READ_FACTORY} function main() {{ let f = make(\"Pat\"); const reader = (): string => {{ return f(); }}; f = make(\"Sam\"); }}"
    );
    let error = reject(&source, "capture-conflict");
    assert_eq!(error.related.len(), 1);
}

#[test]
fn loop_back_edges_require_callable_reinitialization() {
    execute(
        include_str!("fixtures/mutable-callables/reinitialize.tsr"),
        "Pat\nSam\nSam\nSam\n",
        None,
    );
    execute(
        &format!(
            "{ONCE_FACTORY} function main() {{ let n = 0; let f = make(\"Pat\"); while (n < 3) {{ console.log(f()); f = make(\"Sam\"); n = n + 1; continue; }} console.log(f()); }}"
        ),
        "Pat\nSam\nSam\nSam\n",
        None,
    );
    reject(
        &format!(
            "{ONCE_FACTORY} function main() {{ let f = make(\"Pat\"); while (true) {{ f(); if (false) {{ f = make(\"Sam\"); }} }} }}"
        ),
        "loop-transfer",
    );
    reject(
        &format!(
            "{ONCE_FACTORY} function main() {{ let f = make(\"Pat\"); f(); while (true) {{ f = make(\"Sam\"); break; }} f(); }}"
        ),
        "closure-consumed",
    );
}

#[test]
fn local_and_owned_wrappers_can_replace_captured_callable_bindings() {
    execute(
        &format!(
            "{READ_FACTORY} function main() {{ let f = make(\"Pat\"); if (true) {{ const reset = (): void => {{ f = make(\"Sam\"); console.log(f()); }}; reset(); reset(); }} console.log(f()); }}"
        ),
        "Sam\nSam\nSam\n",
        None,
    );
    let source = format!(
        "{READ_FACTORY} function holder(): (text: Owned<string>) => string {{ let f = make(\"Pat\"); return (text: string): string => {{ f = make(text); return f(); }}; }} function main() {{ const reset = holder(); console.log(reset(\"Sam\")); console.log(reset(\"Alex\")); }}"
    );
    execute(&source, "Sam\nAlex\n", None);
    assert_eq!(
        check(&source)
            .unwrap()
            .closures()
            .iter()
            .find(|c| c.function == "holder")
            .unwrap()
            .invocation,
        Mode::Mutate
    );
}

#[test]
fn branches_join_restored_callable_availability() {
    execute(
        &format!(
            "{ONCE_FACTORY} function f(flag: boolean) {{ let action = make(\"Pat\"); action(); if (flag) {{ action = make(\"Sam\"); }} else {{ action = make(\"Alex\"); }} console.log(action()); }} function main() {{ f(true); f(false); }}"
        ),
        "Sam\nAlex\n",
        None,
    );
    reject(
        &format!(
            "{ONCE_FACTORY} function f(flag: boolean) {{ let action = make(\"Pat\"); action(); if (flag) {{ action = make(\"Sam\"); }} action(); }} function main() {{}}"
        ),
        "closure-consumed",
    );
}

#[test]
fn different_origins_and_immutable_targets_have_source_diagnostics() {
    for source in [
        "function main() { let f = (): int => { return 1; }; f = (): int => { return 2; }; }"
            .to_string(),
        format!(
            "{READ_FACTORY} function other(): () => string {{ return (): string => {{ return \"Sam\"; }}; }} function main() {{ let f = make(\"Pat\"); f = other(); }}"
        ),
    ] {
        let error = reject(&source, "callable-origin");
        assert_eq!(error.related.len(), 2);
    }
    reject(
        &format!(
            "{READ_FACTORY} function main() {{ const f = make(\"Pat\"); f = make(\"Sam\"); }}"
        ),
        "unsupported",
    );
    reject(
        &format!("{READ_FACTORY} function main() {{ let f = make(\"Pat\"); f = 7; }}"),
        "type",
    );
    reject(
        &format!(
            "{READ_FACTORY} function bad(action: () => string) {{ let f = make(\"Pat\"); f = action; }} function main() {{}}"
        ),
        "callback-escape",
    );
}

#[test]
fn mutable_callable_analysis_does_not_need_backend() {
    let dir = support::TempDirectory::new();
    let path=dir.source(&format!("{ONCE_FACTORY} function main() {{ let f = make(\"Pat\"); f(); f = make(\"Sam\"); console.log(f()); }}"));
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}
