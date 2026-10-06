mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

const READ_FACTORY: &str =
    "function make(text: string): () => string { return (): string => { return text.trim(); }; }";
const ONCE_FACTORY: &str =
    "function make(text: string): () => string { return (): string => { return text; }; }";

#[test]
fn declaration_chains_move_concrete_environments_without_clones() {
    let source = format!(
        "{READ_FACTORY} function main() {{ const first = make(\" Pat \"); let second = first; const third = second; console.log(third()); console.log(third()); }}"
    );
    let rust = execute(&source, "Pat\nPat\n", None);
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
    for name in ["first", "second"] {
        let error = reject(
            &source.replace("console.log(third());", &format!("{name}();")),
            "closure-consumed",
        );
        assert_eq!(error.related.len(), 1);
    }
    execute(
        "function main() { const first = (n: int): int => { return n + 1; }; const next = first; console.log(next(6)); }",
        "7\n",
        None,
    );
}

#[test]
fn mutable_destinations_and_sources_reinitialize_independently() {
    execute(
        &format!(
            "{READ_FACTORY} function main() {{ let first = make(\"Pat\"); let second = first; first = make(\"Sam\"); second = make(\"Alex\"); console.log(first()); console.log(second()); }}"
        ),
        "Sam\nAlex\n",
        None,
    );
    reject(
        &format!(
            "{READ_FACTORY} function main() {{ let first = make(\"Pat\"); const second = first; second = make(\"Sam\"); }}"
        ),
        "unsupported",
    );
    reject(
        "function main() { const first = (): int => { return 1; }; let second = first; second = (): int => { return 2; }; }",
        "callable-origin",
    );
}

#[test]
fn moves_preserve_consuming_invocation_and_argument_contracts() {
    let source = format!(
        "{ONCE_FACTORY} function main() {{ const first = make(\"Pat\"); const next = first; console.log(next()); }}"
    );
    execute(&source, "Pat\n", None);
    reject(
        &source.replace("console.log(next());", "next(); next();"),
        "closure-consumed",
    );
    reject(
        &source.replace("const next = first;", "first(); const next = first;"),
        "closure-consumed",
    );
    execute(
        "function main() { const first = (s: string): string => { return s; }; const next = first; const text = \"Pat\"; console.log(next(text)); }",
        "Pat\n",
        None,
    );
    reject(
        "function main() { const first = (s: string): string => { return s; }; const next = first; const text = \"Pat\"; next(text); console.log(text); }",
        "transferred",
    );
}

#[test]
fn borrowed_moves_keep_loans_until_the_original_lexical_scope_ends() {
    execute(
        "function main() { let n = 0; if (true) { const first = (): void => { n = n + 1; }; if (true) { const next = first; next(); next(); } } console.log(n); }",
        "2\n",
        None,
    );
    reject(
        "function main() { let n = 0; const first = (): void => { n = n + 1; }; if (true) { const next = first; next(); } console.log(n); }",
        "capture-conflict",
    );
    execute(
        "function main() { const text = \" Pat \"; const first = (): string => { return text.trim(); }; const next = first; console.log(next()); console.log(text); }",
        "Pat\n Pat \n",
        None,
    );
    reject(
        "function main() { let text = \"Pat\"; const first = (): string => { return text.trim(); }; const next = first; text = \"Sam\"; }",
        "capture-conflict",
    );
    reject(
        "function main() { const first = (): int => { return 1; }; const reader = (): int => { return first(); }; const next = first; }",
        "capture-conflict",
    );
}

#[test]
fn returned_move_chains_preserve_the_original_escape_origin() {
    execute(
        include_str!("fixtures/callable-moves/returned-chain.tsr"),
        "Pat\nPat\n",
        None,
    );
    execute(
        "function make(text: string, flag: boolean): () => string { const first = (): string => { return text.trim(); }; if (flag) { const next = first; return next; } const last = first; return last; } function main() { const a = make(\" Pat \", true); const b = make(\" Sam \", false); console.log(a()); console.log(b()); }",
        "Pat\nSam\n",
        None,
    );
    execute(
        "function main() { const make = (text: string): () => string => { const first = (): string => { return text.trim(); }; const next = first; return next; }; const action = make(\" Pat \"); console.log(action()); }",
        "Pat\n",
        None,
    );
    // Escape promotion moves text when the original environment is created.
    reject(
        "function make(text: string): () => string { const first = (): string => { return text.trim(); }; const next = first; console.log(text); return next; } function main() {}",
        "transferred",
    );
}

#[test]
fn moving_captured_callables_makes_the_wrapper_single_use() {
    let source = format!(
        "{READ_FACTORY} function main() {{ const first = make(\"Pat\"); const wrap = (): void => {{ const next = first; console.log(next()); console.log(next()); }}; wrap(); }}"
    );
    execute(&source, "Pat\nPat\n", None);
    assert_eq!(
        check(&source)
            .unwrap()
            .closures()
            .iter()
            .find(|c| c.function == "main")
            .unwrap()
            .invocation,
        Mode::Consume
    );
    reject(
        &source.replace("wrap(); }", "wrap(); wrap(); }"),
        "closure-consumed",
    );
    execute(
        "function main() { let n = 0; if (true) { const first = (): void => { n = n + 1; }; const wrap = (): void => { const next = first; next(); next(); }; wrap(); } console.log(n); }",
        "2\n",
        None,
    );
    execute(
        &format!(
            "{READ_FACTORY} function wrap(): () => string {{ const first = make(\"Pat\"); return (): string => {{ const next = first; return next(); }}; }} function main() {{ const action = wrap(); console.log(action()); }}"
        ),
        "Pat\n",
        None,
    );
    reject(
        "function bad(text: string): () => string { const first = (): string => { return text.trim(); }; const next = first; return (): string => { return next(); }; } function main() {}",
        "closure-escape",
    );
}

#[test]
fn branches_join_transfers_and_exclude_terminated_paths() {
    execute(
        &format!(
            "{READ_FACTORY} function use(flag: boolean) {{ const first = make(\"Pat\"); if (flag) {{ const next = first; console.log(next()); return; }} console.log(first()); }} function main() {{ use(true); use(false); }}"
        ),
        "Pat\nPat\n",
        None,
    );
    reject(
        &format!(
            "{READ_FACTORY} function use(flag: boolean) {{ const first = make(\"Pat\"); if (flag) {{ const next = first; next(); }} first(); }} function main() {{}}"
        ),
        "closure-consumed",
    );
    execute(
        &format!(
            "{READ_FACTORY} function use(flag: boolean) {{ let first = make(\"Pat\"); if (flag) {{ const next = first; first = make(\"Sam\"); }} console.log(first()); }} function main() {{ use(true); use(false); }}"
        ),
        "Sam\nPat\n",
        None,
    );
}

#[test]
fn loop_moves_require_restoration_on_back_edges() {
    execute(
        &format!(
            "{ONCE_FACTORY} function main() {{ let first = make(\"Pat\"); let n = 0; while (n < 2) {{ const next = first; console.log(next()); first = make(\"Sam\"); n = n + 1; continue; }} console.log(first()); }}"
        ),
        "Pat\nSam\nSam\n",
        None,
    );
    reject(
        &format!(
            "{READ_FACTORY} function main() {{ const first = make(\"Pat\"); while (true) {{ const next = first; next(); }} }}"
        ),
        "loop-transfer",
    );
    reject(
        &format!(
            "{READ_FACTORY} function main() {{ const first = make(\"Pat\"); while (true) {{ const next = first; break; }} first(); }}"
        ),
        "closure-consumed",
    );
    execute(
        "function main() { let n = 0; while (n < 2) { const first = (): int => { return 7; }; const next = first; console.log(next()); n = n + 1; } }",
        "7\n7\n",
        None,
    );
}

#[test]
fn callback_forwarding_and_parameter_moves_preserve_contracts() {
    execute(
        &format!(
            "{READ_FACTORY} function twice(action: Mutable<Function<string>>) {{ console.log(action()); console.log(action()); }} function main() {{ const first = make(\"Pat\"); const next = first; twice(next); console.log(next()); }}"
        ),
        "Pat\nPat\nPat\n",
        None,
    );
    execute(
        &format!(
            "{READ_FACTORY} function keep(action: Function<string>): () => string {{ return (): string => {{ return action(); }}; }} function main() {{ const first = make(\"Pat\"); const next = first; const saved = keep(next); console.log(saved()); }}"
        ),
        "Pat\n",
        None,
    );
    for contract in ["Function<string>", "Once<Function<string>>", "Owned<Function<string>>", "Once<Function<string>>"] {
        execute(
            &format!(
                "function move(action: {contract}) {{ const next = action; console.log(next()); }} function main() {{ move((): string => {{ return \"Pat\"; }}); }}"
            ),
            "Pat\n",
            None,
        );
    }
    reject(
        "function main() { const first = (): int => { return 1; }; const next: int = first; }",
        "unsupported",
    );
    reject(
        "function main() { const first = (): int => { return 1; }; console.log(first); }",
        "unsupported",
    );
}

#[test]
fn declaration_move_analysis_is_backend_independent() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/callable-moves/returned-chain.tsr"));
    for command in ["check", "emit"] {
        let out = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&out);
    }
}
