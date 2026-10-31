mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn scalar_iteration_preserves_order_and_the_source_array() {
    let source = include_str!("fixtures/for-of/sum.tsr");
    let rust = execute(source, "8\n5\ntrue\nfalse\n", None);
    assert!(rust.contains(".iter().copied()"));
    assert!(!rust.contains("Clone::clone("));
    assert_eq!(check(source).unwrap().contracts()["sum.values"], Mode::Read);
}

#[test]
fn iterable_expressions_evaluate_once_and_temporary_storage_lives_through_loop() {
    execute(
        "function make(): int[] { console.log(7); return [1, 2]; } function main() { for (const n of make()) { console.log(n); } for (const flag of [true, false]) { console.log(flag); } }",
        "7\n1\n2\ntrue\nfalse\n",
        None,
    );
    execute(
        "function take(a: int[]): int[] { return a; } function main() { const a = [1, 2]; for (const n of take(a)) { console.log(n); } }",
        "1\n2\n",
        None,
    );
    reject(
        "function take(a: int[]): int[] { return a; } function main() { const a: int[] = []; for (const n of take(a)) {} console.log(a.length); }",
        "transferred",
    );
    execute(
        "function main() { const a = [1, 2]; for (const n of clone(a)) { a.push(n); } console.log(a.length); }",
        "4\n",
        None,
    );
}

#[test]
fn iteration_bindings_are_fresh_copies_with_lexical_scope() {
    execute(
        "function main() { const a = [1, 2]; for (let n of a) { n = n + 10; console.log(n); } console.log(a[0]); for (const n of a) { console.log(n); } }",
        "11\n12\n1\n1\n2\n",
        None,
    );
    reject(
        "function main() { for (const n of [1]) { n = 7; } }",
        "unsupported",
    );
    reject(
        "function main() { for (const n of [1]) {} console.log(n); }",
        "name",
    );
    reject(
        "function main() { const n = 7; for (const n of [1]) {} }",
        "duplicate",
    );
    execute(
        "function main() { for (let n of [1, 2]) { if (true) { const add = (): void => { n = n + 1; }; add(); } console.log(n); } }",
        "2\n3\n",
        None,
    );
}

#[test]
fn break_continue_and_nested_loops_target_the_nearest_loop() {
    execute(
        "function main() { let total = 0; for (const n of [1, 2, 3, 4]) { if (n === 2) { continue; } if (n === 4) { break; } total = total + n; } console.log(total); for (const outer of [1, 2]) { for (const inner of [3, 4]) { console.log(outer + inner); break; } let n = 0; while (n < 2) { n = n + 1; if (n === 1) { continue; } console.log(n); } } }",
        "4\n4\n2\n5\n2\n",
        None,
    );
    reject(
        "function main() { for (const n of [1]) { const f = (): void => { break; }; } }",
        "loop-control",
    );
    reject(
        "function main() { for (const n of [1]) { continue; console.log(n); } }",
        "unsupported",
    );
}

#[test]
fn active_iteration_blocks_mutation_replacement_and_transfer() {
    for body in [
        "a.push(3);",
        "a[0] = 3;",
        "a = [3];",
        "const moved = a;",
        "mutate(a);",
        "take(a);",
        "const change = (): void => { a.push(3); };",
        "const moved = (): int[] => { return a; };",
    ] {
        let source = format!(
            "function mutate(a: int[]) {{ a.push(7); }} function take(a: int[]) {{ const owned = a; }} function main() {{ let a = [1, 2]; for (const n of a) {{ {body} }} }}"
        );
        let error = reject(&source, "iteration-conflict");
        assert_eq!(error.related.len(), 1);
    }
    execute(
        "function main() { let a = [1, 2]; for (const n of a) { console.log(a.length); } a.push(3); a = [4]; console.log(a[0]); }",
        "2\n2\n4\n",
        None,
    );
    execute(
        "function main() { const a = [1]; for (const n of a) { break; } a.push(2); console.log(a.length); }",
        "2\n",
        None,
    );
}

#[test]
fn nested_read_loans_and_callback_captures_preserve_array_access() {
    execute(
        "function main() { const a = [1, 2]; for (const n of a) { for (const m of a) { console.log(n + m); } const read = (): int => { return a.length; }; console.log(read()); } a.push(3); console.log(a.length); }",
        "2\n3\n2\n3\n4\n2\n3\n",
        None,
    );
    reject(
        "function main() { const a = [1]; const change = (): void => { a.push(2); }; for (const n of a) {} }",
        "capture-conflict",
    );
    execute(
        "function use(action: () => int) { console.log(action()); } function main() { for (const n of [1, 2]) { use((): int => { return n; }); } }",
        "1\n2\n",
        None,
    );
    execute(
        "function make(a: int[]): () => int { return (): int => { let total = 0; for (const n of a) { total = total + n; } return total; }; } function main() { const f = make([1, 2]); console.log(f()); console.log(f()); }",
        "3\n3\n",
        None,
    );
}

#[test]
fn moved_outer_values_require_restoration_on_repeating_paths() {
    execute(
        "function take(text: string) { const saved = text; console.log(saved); } function main() { let text = \"Pat\"; for (const n of [1, 2]) { take(text); text = \"Sam\"; continue; } console.log(text); }",
        "Pat\nSam\nSam\n",
        None,
    );
    reject(
        "function main() { const text = \"Pat\"; for (const n of [1, 2]) { const moved = text; } }",
        "loop-transfer",
    );
    reject(
        "function main() { const text = \"Pat\"; for (const n of [1]) { const moved = text; break; } console.log(text); }",
        "transferred",
    );
    execute(
        "function use(flag: boolean): string { const text = \"Pat\"; for (const n of [1]) { if (flag) { return text; } } return text; } function main() { console.log(use(true)); console.log(use(false)); }",
        "Pat\nPat\n",
        None,
    );
}

#[test]
fn empty_iteration_and_returned_closures_keep_conservative_flow_rules() {
    execute(
        "function main() { const a: int[] = []; let n = 0; for (const value of a) { n = n + 1; } console.log(n); }",
        "0\n",
        None,
    );
    reject(
        "function bad(a: int[]): int { for (const n of a) { return n; } } function main() {}",
        "type",
    );
    reject(
        "function bad(): int { for (const n of [1]) { return n; } } function main() {}",
        "type",
    );
    reject(
        "function make(a: int[]): () => int { const fallback = (): int => { return 0; }; for (const n of a) { const f = (): int => { return n; }; return f; } return fallback; } function main() {}",
        "unsupported",
    );
    execute(
        "function retain(action: Function<int>): () => int { return action; } function main() { for (const n of [1, 2]) { const f = retain((): int => { return n; }); console.log(f()); } }",
        "1\n2\n",
        None,
    );
}

#[test]
fn unsupported_iteration_forms_and_deep_loops_have_source_diagnostics() {
    reject(
        "function main() { for (const n of 7) {} }",
        "iteration-type",
    );
    tsrust_compiler::check("function main() { for (const n of \"Pat\") {} }").unwrap();
    reject("function main() { for (const n of []) {} }", "array-type");
    reject("function main() { for (n of [1]) {} }", "unsupported");
    // for...in is supported; an existing variable in its place is not.
    reject("function main() { for (n in [1]) {} }", "unsupported");
    reject(
        "function main() { for (let n = 0; n < 2 n = n + 1) {} }",
        "syntax",
    );
    let loops: String = (0..200).map(|level| format!("for (const n{level} of [1]) {{")).collect();
    support::accept_deep(&format!("function main() {{ {loops} {} }}", "}".repeat(200)));
    let loops = (0..100)
        .map(|index| format!("for (const n{index} of [1]) {{"))
        .collect::<String>();
    let source = format!(
        "function main() {{ {loops} console.log(n99); {} }}",
        "}".repeat(100)
    );
    let rust = check(&source)
        .expect("valid nesting below the limit")
        .emit_rust();
    assert_eq!(rust.matches(".iter().copied()").count(), 100);
}

#[test]
fn for_of_analysis_and_emission_need_no_backend() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/for-of/sum.tsr"));
    for command in ["check", "emit"] {
        let output = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&output);
    }
}
