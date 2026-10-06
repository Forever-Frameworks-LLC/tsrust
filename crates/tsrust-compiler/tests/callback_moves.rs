mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn reusable_moves_transfer_local_borrows_and_preserve_caller_ownership() {
    let source = "function use(action: Mutable<Function<int>>) { const next = action; const last = next; console.log(last()); console.log(last()); } function main() { const f = (): int => { return 7; }; use(f); console.log(f()); }";
    let rust = execute(source, "7\n7\n7\n", None);
    assert_eq!(
        check(source).unwrap().contracts()["use.action"],
        Mode::Mutate
    );
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden));
    }
    for name in ["action", "next"] {
        let error = reject(
            &source.replace("console.log(last());", &format!("{name}();")),
            "callback-consumed",
        );
        assert_eq!(error.related.len(), 1);
    }
    execute(
        "function discard(action: Mutable<Function<int>>) { const next = action; } function main() { const f = (): int => { return 7; }; discard(f); console.log(f()); }",
        "7\n",
        None,
    );
}

#[test]
fn reusable_moves_forward_mutation_and_owned_arguments() {
    execute(
        "type U = { name: string; }; function call(action: Mutable<Function<string, [u: Mutable<U>]>>, u: U): string { return action(u); } function move(action: Mutable<Function<string, [u: Mutable<U>]>>, u: U): string { const next = action; console.log(call(next, u)); return next(u); } function main() { const f = (u: U): string => { u.name = \"Sam\"; return u.name.trim(); }; const user: U = { name: \"Pat\" }; console.log(move(f, user)); console.log(user.name); }",
        "Sam\nSam\nSam\n",
        None,
    );
    execute(
        "function use(action: Mutable<Function<int>>) { const next = action; console.log(next()); console.log(next()); } function main() { let n = 0; if (true) { const f = (): int => { n = n + 1; return n; }; use(f); console.log(f()); } console.log(n); }",
        "1\n2\n3\n3\n",
        None,
    );
    execute(
        "function call(action: Mutable<Function<string, [s: Owned<string>]>>): string { const next = action; return next(\"Pat\"); } function main() { console.log(call((s: string): string => { return s; })); }",
        "Pat\n",
        None,
    );
}

#[test]
fn once_moves_preserve_single_use_for_reusable_and_consuming_actuals() {
    let prefix = "function call(action: Once<Function<string>>): string { const next = action; const last = next; return last(); } ";
    for body in ["return text;", "return text.trim();"] {
        let source = format!(
            "{prefix} function main() {{ const text = \"Pat\"; const f: Once<Function<string>> = (): string => {{ {body} }}; console.log(call(f)); }}"
        );
        execute(&source, "Pat\n", None);
        reject(
            &source.replace("console.log(call(f));", "call(f); f();"),
            "closure-consumed",
        );
    }
    reject(
        "function bad(action: Once<Function<int>>) { const next = action; next(); next(); } function main() {}",
        "callback-consumed",
    );
    reject(
        "function bad(action: Once<Function<int>>) { action(); const next = action; } function main() {}",
        "callback-consumed",
    );
    execute(
        "function call(action: Once<Function<int>>): int { return action(); } function move(action: Once<Function<int>>): int { const next = action; return call(next); } function main() { console.log(move((): int => { return 7; })); }",
        "7\n",
        None,
    );
}

#[test]
fn owned_moves_retain_state_in_returned_wrappers() {
    execute(
        include_str!("fixtures/callback-moves/retained.tsr"),
        "1\n2\n1\n",
        None,
    );
    execute(
        "function retain(action: Function<string, [s: Owned<string>]>): (s: Owned<string>) => string { const next = action; return (s: string): string => { return next(s); }; } function main() { const f = retain((s: string): string => { return s; }); console.log(f(\"Pat\")); console.log(f(\"Sam\")); }",
        "Pat\nSam\n",
        None,
    );
    reject(
        "function bad(action: Function<int>) { const next = action; action(); } function main() {}",
        "callback-consumed",
    );
    let source = "function retain(action: Once<Function<int>>): () => int { const next = action; return (): int => { return next(); }; } function main() { const f = retain((): int => { return 7; }); console.log(f()); }";
    execute(source, "7\n", None);
    reject(
        &source.replace("console.log(f());", "f(); f();"),
        "closure-consumed",
    );
    reject(
        "function bad(action: Once<Function<int>>) { const next = action; next(); next(); } function main() {}",
        "callback-consumed",
    );
}

#[test]
fn captured_owned_moves_propagate_consuming_invocation() {
    execute(
        "function use(action: Function<int>) { const run = (): void => { const next = action; console.log(next()); console.log(next()); }; run(); } function main() { use((): int => { return 7; }); }",
        "7\n7\n",
        None,
    );
    let source = "function retain(action: Function<int>): () => int { const next = action; return (): int => { const last = next; return last(); }; } function main() { const f = retain((): int => { return 7; }); console.log(f()); }";
    execute(source, "7\n", None);
    reject(
        &source.replace("console.log(f());", "f(); f();"),
        "closure-consumed",
    );
    reject(
        "function bad(action: Function<int>) { const next = action; const reader = (): int => { return next(); }; const last = next; } function main() {}",
        "capture-conflict",
    );
}

#[test]
fn mutable_callback_locals_preserve_parameter_origin() {
    for contract in ["Mutable<Function<int>>", "ReadOnly<Function<int>>", "Function<int>", "Once<Function<int>>", "Owned<Function<int>>", "Once<Function<int>>"] {
        let source = format!(
            "function use(action: {contract}): int {{ let next = action; const saved = next; next = saved; next = next; return next(); }} function main() {{ console.log(use((): int => {{ return 7; }})); }}"
        );
        execute(&source, "7\n", None);
        reject(
            &format!(
                "function bad(a: {contract}, b: {contract}) {{ let next = a; next = b; }} function main() {{}}"
            ),
            "callable-origin",
        );
        reject(
            &format!(
                "function bad(a: {contract}) {{ let next = a; next = (): int => {{ return 7; }}; }} function main() {{}}"
            ),
            "callable-origin",
        );
    }
    reject(
        "function bad(action: Mutable<Function<int>>) { const next = action; next = next; } function main() {}",
        "unsupported",
    );
    reject(
        "function bad(action: Once<Function<int>>) { let next = action; next(); next = next; } function main() {}",
        "callback-consumed",
    );
}

#[test]
fn branch_and_loop_joins_track_moved_callback_bindings() {
    execute(
        "function use(action: Mutable<Function<int>>, flag: boolean) { if (flag) { const next = action; console.log(next()); return; } console.log(action()); } function main() { const f = (): int => { return 7; }; use(f, true); use(f, false); }",
        "7\n7\n",
        None,
    );
    reject(
        "function bad(action: Mutable<Function<int>>, flag: boolean) { if (flag) { const next = action; } action(); } function main() {}",
        "callback-consumed",
    );
    execute(
        "function use(action: Mutable<Function<int>>) { let next = action; let n = 0; while (n < 2) { const saved = next; console.log(saved()); next = saved; n = n + 1; continue; } console.log(next()); } function main() { use((): int => { return 7; }); }",
        "7\n7\n7\n",
        None,
    );
    reject(
        "function bad(action: Mutable<Function<int>>) { while (true) { const next = action; } } function main() {}",
        "loop-transfer",
    );
    reject(
        "function bad(action: Mutable<Function<int>>) { while (true) { const next = action; break; } action(); } function main() {}",
        "callback-consumed",
    );
}

#[test]
fn moves_do_not_relax_callback_escape_or_contract_rules() {
    for contract in ["Mutable<Function<int>>", "ReadOnly<Function<int>>"] {
        reject(
            &format!(
                "function bad(action: {contract}) {{ const next = action; const f = (): int => {{ return next(); }}; }} function main() {{}}"
            ),
            "callback-escape",
        );
        reject(
            &format!(
                "function bad(action: {contract}) {{ const f = (): void => {{ const next = action; }}; }} function main() {{}}"
            ),
            "callback-escape",
        );
    }
    for contract in ["Mutable<Function<int>>", "ReadOnly<Function<int>>", "Function<int>", "Once<Function<int>>", "Owned<Function<int>>", "Once<Function<int>>"] {
        let source = format!(
            "function identity(action: {contract}): () => int {{ const next = action; return next; }} function main() {{}}"
        );
        if !matches!(contract, "ReadOnly<Function<int>>" | "Mutable<Function<int>>") {
            execute(&source, "", None);
        } else {
            reject(&source, "callback-escape");
        }
        reject(
            &format!(
                "function bad(action: {contract}) {{ const next = action; console.log(next); }} function main() {{}}"
            ),
            "callback-escape",
        );
    }
    reject(
        "function take(action: Function<int>) {} function bad(action: Mutable<Function<int>>) { const next = action; take(next); } function main() {}",
        "callback-contract",
    );
    reject(
        "function take(action: Mutable<Function<int>>) {} function bad(action: Once<Function<int>>) { const next = action; take(next); } function main() {}",
        "callback-contract",
    );
    reject(
        "function pair(a: Mutable<Function<int>>, b: Mutable<Function<int>>) { a(); b(); } function bad(action: Mutable<Function<int>>) { const next = action; pair(next, next); } function main() {}",
        "conflicting-access",
    );
}

#[test]
fn specialized_moves_preserve_declared_once_and_reusable_access() {
    execute(
        "function use(borrowed: Mutable<Function<int>>, owned: Function<int>) { const first = borrowed; const second = owned; console.log(first()); console.log(second()); } function main() { const f = (): int => { return 7; }; use(f, (): int => { return 8; }); console.log(f()); }",
        "7\n8\n7\n",
        None,
    );
    let source = "function use(action: Once<Function<int>>): int { const next = action; return next(); } function main() { console.log(use((): int => { return 7; })); }";
    let rust = execute(source, "7\n", None);
    assert!(rust.contains("OnceCallback::invoke_once"));
    execute(
        "function use(action: Function<int>): int { const next = action; return next(); } function forward(action: Function<int>): int { const next = action; return use(next); } function main() { console.log(forward((): int => { return 7; })); }",
        "7\n",
        None,
    );
    let source = include_str!("fixtures/callback-moves/retained.tsr");
    assert_eq!(
        check(source).unwrap().emit_rust(),
        check(source).unwrap().emit_rust()
    );
}

#[test]
fn callback_moves_check_and_emit_without_the_backend() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/callback-moves/retained.tsr"));
    for command in ["check", "emit"] {
        let output = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&output);
    }
}
