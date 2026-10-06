mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn recursive_tree_traversal_borrows_the_callback_and_its_data() {
    let rust = execute(
        include_str!("fixtures/recursive-callbacks/visit.tsr"),
        "6\n",
        Some(&[("visit.node", Mode::Read), ("visit.action", Mode::Mutate)]),
    );
    assert!(rust.contains("F1:"));
    assert!(!rust.contains("Box<"));
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn reusable_callbacks_run_in_source_order_and_retain_state() {
    execute(
        "function sum(c: Mutable<Function<int, [n: int]>>, n: int): int { if (n === 0) { return c(0); } return c(n) + sum(c, n - 1); } function main() { let calls = 0; if (true) { const add = (n: int): int => { calls = calls + 1; console.log(n); return n; }; console.log(sum(add, 3)); console.log(add(7)); } console.log(calls); }",
        "3\n2\n1\n0\n6\n7\n7\n5\n",
        Some(&[("sum.c", Mode::Mutate), ("sum.n", Mode::Copy)]),
    );
    execute(
        "function ignore(c: Mutable<Function<void>>, n: int) { if (n > 0) { ignore(c, n - 1); } } function main() { ignore((): void => { console.log(7); }, 3); console.log(1); }",
        "1\n",
        Some(&[("ignore.c", Mode::Read), ("ignore.n", Mode::Copy)]),
    );
}

#[test]
fn mutual_callback_forwarding_preserves_parameter_aliases_and_order() {
    let f = "function f(c: Mutable<Function<int, [n: int]>>, n: int): int { if (n === 0) { return c(7); } const next = c; return g(next, n - 1); }";
    let g = "function g(c: Mutable<Function<int, [n: int]>>, n: int): int { return f(c, n); }";
    for functions in [format!("{f}{g}"), format!("{g}{f}")] {
        execute(
            &format!(
                "{functions} function main() {{ const add = (n: int): int => {{ return n + 1; }}; console.log(f(add, 3)); console.log(add(1)); }}"
            ),
            "8\n2\n",
            Some(&[
                ("f.c", Mode::Mutate),
                ("f.n", Mode::Copy),
                ("g.c", Mode::Mutate),
                ("g.n", Mode::Copy),
            ]),
        );
    }
}

#[test]
fn single_use_recursion_owns_environments_and_explicit_copies_preserve_originals() {
    execute(
        "function call(c: Once<Function<string, [value: int]>>, n: int): string { if (n === 0) { return c(7); } const next = c; return call(next, n - 1); } function main() { const name = \"Pat\"; const once = (value: int): string => { console.log(value); return name; }; console.log(call(once, 4)); }",
        "7\nPat\n",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
    execute(
        "type A = { name: string; }; function call(c: Once<Function<string>>, n: int): string { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const a: A = { name: \"Pat\" }; const snapshot=clone(a);console.log(call((): string => { return snapshot.name.trim(); }, 3)); a.name = \"Sam\"; console.log(a.name); }",
        "Pat\nSam\n",
        Some(&[("call.c", Mode::Consume), ("call.n", Mode::Copy)]),
    );
}

#[test]
fn single_use_forwarding_rejects_repeat_invocation_and_caller_reuse() {
    reject(
        "function call(c: Once<Function<int>>, n: int): int { if (n === 0) { return c(); } const value = call(c, n - 1); c(); return value; } function main() {}",
        "callback-consumed",
    );
    reject(
        "function call(c: Once<Function<int>>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const c = (): int => { return 7; }; console.log(call(c, 2)); c(); }",
        "closure-consumed",
    );
}

#[test]
fn recursive_callback_signatures_preserve_owned_argument_access_and_lifetimes() {
    execute(
        "type A = { name: string; }; function read(c: Mutable<Function<string, [item: ReadOnly<A>]>>, a: A, n: int): string { if (n === 0) { return c(a); } return read(c, a, n - 1); } function main() { const owner: A = { name: \"Pat\" }; const c = (item: A): string => { console.log(owner.name); return item.name.trim(); }; console.log(read(c, owner, 2)); }",
        "Pat\nPat\n",
        Some(&[
            ("read.c", Mode::Mutate),
            ("read.a", Mode::Read),
            ("read.n", Mode::Copy),
        ]),
    );
    execute(
        "type A = { n: int; }; function change(c: Mutable<Function<void, [item: Mutable<A>]>>, a: A, n: int) { if (n === 0) { c(a); return; } change(c, a, n - 1); } function main() { const a: A = { n: 1 }; change((item: A): void => { item.n = 7; }, a, 2); console.log(a.n); }",
        "7\n",
        Some(&[
            ("change.c", Mode::Mutate),
            ("change.a", Mode::Mutate),
            ("change.n", Mode::Copy),
        ]),
    );
}

#[test]
fn callback_receiver_and_capture_loans_remain_active_through_later_operands() {
    reject(
        "function other(c: Mutable<Function<int>>): int { return c(); } function read(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return read(c, other(c)); } function main() {}",
        "conflicting-access",
    );
    reject(
        "type A = { n: int; }; function read(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return read(c, n - 1); } function main() { const a: A = { n: 1 }; const c = (): int => { return a.n; }; read(c, 2); a.n = 7; }",
        "capture-conflict",
    );
}

#[test]
fn data_parameter_transfers_stabilize_alongside_recursive_callback_access() {
    execute(
        "type A = { n: int; }; function choose(c: Mutable<Function<A, [item: Owned<A>]>>, a: A, b: A, n: int): A { if (n === 0) { return c(a); } return choose(c, b, a, n - 1); } function main() { const c = (item: A): A => { return item; }; const a = choose(c, { n: 1 }, { n: 7 }, 1); console.log(a.n); const b = c({ n: 9 }); console.log(b.n); }",
        "7\n9\n",
        Some(&[
            ("choose.c", Mode::Mutate),
            ("choose.a", Mode::Consume),
            ("choose.b", Mode::Consume),
            ("choose.n", Mode::Copy),
        ]),
    );
}

#[test]
fn callback_recursion_preserves_shared_iteration_and_loop_transfer_rules() {
    execute(
        "type A = { n: int; }; function visit(a: A[], c: Mutable<Function<void, [item: ReadOnly<A>]>>, n: int) { for (const item of a) { c(item); } if (n > 0) { visit(a, c, n - 1); } } function main() { const a: A[] = [{ n: 1 }, { n: 2 }]; let sum = 0; if (true) { const add = (item: A): void => { sum = sum + item.n; }; visit(a, add, 1); } console.log(sum); }",
        "6\n",
        Some(&[
            ("visit.a", Mode::Read),
            ("visit.c", Mode::Mutate),
            ("visit.n", Mode::Copy),
        ]),
    );
    reject(
        "function visit(c: Once<Function<void, [n: int]>>, n: int) { for (const item of [1, 2]) { c(item); } if (n > 0) { visit(c, n - 1); } } function main() {}",
        "loop-transfer",
    );
}

fn permutation_program(width: usize, repeated: bool) -> String {
    let params = (0..width)
        .map(|i| format!("c{i}: Mutable<Function<int>>"))
        .collect::<Vec<_>>()
        .join(", ");
    let swap = (0..width)
        .map(|i| format!("c{}", if i < 2 { 1 - i } else { i }))
        .collect::<Vec<_>>()
        .join(", ");
    let rotate = (0..width)
        .map(|i| format!("c{}", (i + 1) % width))
        .collect::<Vec<_>>()
        .join(", ");
    let declarations = (0..width)
        .map(|i| {
            if repeated {
                format!("const c{i} = make({i});")
            } else {
                format!("const c{i} = (): int => {{ return {i}; }};")
            }
        })
        .collect::<String>();
    let arguments = (0..width)
        .map(|i| format!("c{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "function make(n: int): () => int {{ return (): int => {{ return n; }}; }} function choose({params}, n: int): int {{ if (n === 0) {{ return c0(); }} if (n === 1) {{ return choose({swap}, n - 1); }} return choose({rotate}, n - 1); }} function main() {{ {declarations} console.log(choose({arguments}, 2)); }}"
    )
}

#[test]
fn instance_limits_count_concrete_types_and_reject_excessive_permutations() {
    execute(&permutation_program(7, true), "2\n", None);
    for width in [4, 5] {
        execute(&permutation_program(width, false), "2\n", None);
    }
    let error = reject(&permutation_program(7, false), "unsupported");
    assert_eq!(
        error.message,
        "Recursive callback instantiations exceed the prototype limit of 128 per native target."
    );
    assert!(!error.related.is_empty());
    // Concrete closure methods are native roots even when the enclosing value
    // is never invoked. An unused generic declaration has no concrete vector.
    let source = permutation_program(7, false);
    let (definitions, main) = source.split_once("function main() {").unwrap();
    let main = main.trim().strip_suffix('}').unwrap();
    execute(&format!("{definitions} function main() {{}}"), "", None);
    reject(
        &format!("{definitions} function main() {{ const run = (): void => {{{main} }}; }}"),
        "unsupported",
    );
    let mut source = String::from(
        "function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function main() {",
    );
    for i in 0..128 {
        source.push_str(&format!("call((): int => {{ return {i}; }}, 1);"));
    }
    execute(&format!("{source}}}"), "", None);
    let error = reject(
        &format!("{source}call((): int => {{ return 128; }}, 1);}}"),
        "unsupported",
    );
    assert_eq!(
        error.message,
        "Recursive callback instantiations exceed the prototype limit of 128 per native target."
    );
    execute(
        "function choose(a: Mutable<Function<int>>, b: Mutable<Function<int>>, n: int): int { if (n === 0) { return a(); } return choose(b, a, n - 1); } function main() { const a = (): int => { return 1; }; const b = (): int => { return 7; }; console.log(choose(a, b, 3)); }",
        "7\n",
        Some(&[
            ("choose.a", Mode::Mutate),
            ("choose.b", Mode::Mutate),
            ("choose.n", Mode::Copy),
        ]),
    );
}

#[test]
fn fresh_closed_callbacks_can_replace_forwarded_values_without_type_growth() {
    execute(
        "function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call((): int => { return n; }, n - 1); } function main() { console.log(call((): int => { return 7; }, 3)); }",
        "1\n",
        Some(&[("call.c", Mode::Mutate), ("call.n", Mode::Copy)]),
    );
    execute(
        "function make(n: int): () => int { return (): int => { return n; }; } function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call(make(n), n - 1); } function main() { console.log(call(make(7), 3)); }",
        "1\n",
        Some(&[
            ("make.n", Mode::Copy),
            ("call.c", Mode::Mutate),
            ("call.n", Mode::Copy),
        ]),
    );
    reject(
        "function call(c: Mutable<Function<int>>, n: int): int { if (n === 0) { return c(); } return call((): int => { return c(); }, n - 1); } function main() {}",
        "callback-escape",
    );
}

#[test]
fn recursive_callback_type_errors_and_callable_result_deferrals_are_precise() {
    for source in [
        "function call(c: Function<int>, n: int): () => int { return call(c, n - 1); } function main() {}",
        "function call(c: Once<Function<int>>, n: int): () => int { return call(c, n - 1); } function main() {}",
    ] {
        assert!(check(source).is_ok());
        let concrete = source.replace(
            "function main() {}",
            "function main() { call((): int => { return 7; }, 1); }",
        );
        assert!(
            reject(&concrete, "unsupported")
                .message
                .contains("environment")
        );
    }
    let error = reject(
        "function call(c: Mutable<Function<int>>, n: int): () => int { return call(c, n - 1); } function main() {}",
        "unsupported",
    );
    assert!(error.message.contains("environment"));
    assert!(!error.related.is_empty());
    reject(
        "function call(c: Mutable<Function<int>>): () => int { return c; } function main() {}",
        "callback-escape",
    );
    reject(
        "function call(c: Mutable<Function<string>>, n: int): string { if (n === 0) { return c(); } return call(c, n - 1); } function main() { const name = \"Pat\"; call((): string => { return name; }, 2); }",
        "callback-once",
    );
    reject(
        "function call(c: Once<Function<int>>, n: int): int { if (n === 0) { return c(); } return call(c, n - 1); } function forward(c: Mutable<Function<int>>): int { return call(c, 2); } function main() {}",
        "callback-contract",
    );
    reject(
        "type A = { n: int; }; function call(c: Mutable<Function<void, [item: ReadOnly<A>]>>, a: A, n: int) { if (n === 0) { c(a); return; } call(c, a, n - 1); } function main() { const a: A = { n: 1 }; call((item: A): void => { item.n = 7; }, a, 2); }",
        "callback-access",
    );
}

#[test]
fn recursive_callback_cycles_retain_source_depth_guards() {
    let mut source = String::new();
    for i in 0..20 {
        source.push_str(&format!("function f{i}(c: Mutable<Function<int>>, n: int): int {{ if (n === 0) {{ return c(); }} return f{}(c, n - 1); }}", (i+1)%20));
    }
    source.push_str("function main() { console.log(f0((): int => { return 7; }, 40)); }");
    execute(&source, "7\n", None);
    let mut source = String::new();
    for i in 0..200 {
        source.push_str(&format!(
            "function f{i}(c: Mutable<Function<int>>): int {{ return f{}(c); }}",
            (i + 1) % 200
        ));
    }
    source.push_str("function main() {}");
    reject(&source, "unsupported");
}

#[test]
fn callback_inference_and_instance_checks_need_no_backend_and_preserve_metadata() {
    let source = include_str!("fixtures/recursive-callbacks/visit.tsr");
    let analysis = check(source).unwrap();
    assert_eq!(analysis.closures().len(), 1);
    assert_eq!(analysis.to_json(), check(source).unwrap().to_json());
    assert_eq!(analysis.emit_rust(), check(source).unwrap().emit_rust());
    let dir = support::TempDirectory::new();
    let path = dir.source(source);
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
}
