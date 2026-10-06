mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

// Ignore private runtime-support loops. Source for-of lowering uses a binding
// named value<id>, or literal_element with a borrowed Cow proxy in its body.
fn source_iteration_lines(rust: &str) -> impl Iterator<Item = &str> {
    rust.lines().filter(|line| {
        let line = line.trim_start();
        // Emitted loops carry a `'loop_N: ` label for labeled control.
        let line = line
            .strip_prefix("'loop_")
            .and_then(|rest| rest.split_once(": "))
            .filter(|(id, _)| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))
            .map_or(line, |(_, rest)| rest);
        if line.starts_with("for literal_element in (") {
            return true;
        }
        let Some(rest) = line
            .strip_prefix("for value")
            .or_else(|| line.strip_prefix("for mut value"))
        else {
            return false;
        };
        rest.split_once(" in (").is_some_and(|(binding, _)| {
            !binding.is_empty() && binding.bytes().all(|byte| byte.is_ascii_digit())
        })
    })
}

#[test]
fn string_elements_move_into_literals_pushes_and_replacements() {
    let rust = execute(
        "function main() { const first = \"Pat\"; const a = [first, \"Sam\"]; const third = \"Alex\"; a.push(third); const next = \"Jo\"; a[0] = next; console.log(a[0]); console.log(a[1]); console.log(a[2]); console.log(a.length); }",
        "Jo\nSam\nAlex\n3\n",
        None,
    );
    assert!(rust.contains("Array<String>"));
    assert!(!rust.contains("Clone::clone("));
    for body in [
        "const s = \"Pat\"; const a = [s]; console.log(s);",
        "const a: string[] = []; const s = \"Pat\"; a.push(s); console.log(s);",
        "const a = [\"Sam\"]; const s = \"Pat\"; a[0] = s; console.log(s);",
        "const s = \"Pat\"; const a = [s, s];",
    ] {
        reject(&format!("function main() {{ {body} }}"), "transferred");
    }
}

#[test]
fn record_elements_support_fields_and_inferred_mutating_arguments() {
    let source = include_str!("fixtures/owned-arrays/users.tsr");
    let rust = execute(
        source,
        "Pat\ntrue\nSam\nfalse\nPat\nAlex\n",
        Some(&[("activate.user", Mode::Mutate)]),
    );
    assert!(rust.contains("RecordArray0"));
    assert!(rust.contains(".iter()"));
    assert!(source_iteration_lines(&rust).all(|line| !line.contains(".iter().copied()")));
    execute(
        "type U = { n: int; }; function main() { const first: U = { n: 1 }; const a = [first]; a.push({ n: 2 }); a[0].n = a[1].n; console.log(a[0].n); a[1] = { n: 3 }; console.log(a[1].n); }",
        "2\n3\n",
        None,
    );
}

#[test]
fn owned_array_contexts_match_element_types_exactly() {
    execute(
        "type U = { name: string; }; function empty(): U[] { return []; } function names(a: string[]): int { return a.length; } function main() { let a = empty(); a = [{ name: \"Pat\" }]; console.log(a[0].name); console.log(names([])); const b: string[] = []; b.push(\"Sam\"); console.log(b[0]); }",
        "Pat\n0\nSam\n",
        None,
    );
    for source in [
        "function main() { const a: string[] = [1]; }",
        "type U = { n: int; }; function main() { const a: U[] = [{}]; }",
    ] {
        reject(source, "type");
    }
    reject(
        "type U = { n: int; }; function main() { const a = [{ n: 1 }]; }",
        "type",
    );
}

#[test]
fn explicit_clone_produces_independent_elements_and_arrays() {
    execute(
        "type U = { name: string; }; function main() { const a: U[] = [{ name: \"Pat\" }]; const b = clone(a); b[0].name = \"Sam\"; const owned = clone(a[0]); console.log(a[0].name); console.log(b[0].name); console.log(owned.name); const s = [\"Alex\"]; const text = clone(s[0]); s[0] = \"Jo\"; console.log(text); console.log(s[0]); }",
        "Pat\nSam\nPat\nAlex\nJo\n",
        None,
    );
    execute(
        "function main() { const s = \"Pat\"; const t = clone(s); console.log(s); console.log(t); }",
        "Pat\nPat\n",
        None,
    );
}

#[test]
fn indexing_borrows_owned_values_and_rejects_implicit_extraction() {
    execute(
        "function read(s: string) { console.log(s); } function main() { const a = [\"  PAT  \"]; read(a[0]); console.log(a[0]); const b = a[0].trim().toLowerCase(); console.log(b); }",
        "  PAT  \n  PAT  \npat\n",
        Some(&[("read.s", Mode::Read)]),
    );
    for body in [
        "a.push(a[0]);",
        "a[0] = a[0];",
        "take(a[0]);",
    ] {
        reject(
            &format!(
                "function take(s: string): string {{ return s; }} function main() {{ const a = [\"Pat\"]; {body} }}"
            ),
            "borrowed-element",
        );
    }
    reject(
        "function get(a: string[]): string { return a[0]; } function main() {}",
        "borrowed-element",
    );
    reject(
        "type U = { n: int; }; function get(a: U[]): U { return a[0]; } function main() {}",
        "borrowed-element",
    );
    execute(
        "function get(a: string[]): string { return clone(a[0]); } function main() { const a = [\"Pat\"]; console.log(get(a)); console.log(a[0]); }",
        "Pat\nPat\n",
        Some(&[("get.a", Mode::Read)]),
    );
}

#[test]
fn iteration_borrows_owned_elements_and_temporaries_without_cloning() {
    reject(
        "function main() { for (const s of [\"Pat\"]) { s = \"Sam\"; } }",
        "borrowed-element",
    );
    let rust = execute(
        "function main() { const a = [\"Pat\", \"Sam\"]; for (const s of a) { console.log(s); console.log(s.toLowerCase()); } console.log(a[0]); for (const s of [\"Alex\"]) { console.log(s); } for (let s of a) { const owned = clone(s); console.log(owned); } }",
        "Pat\npat\nSam\nsam\nPat\nAlex\nPat\nSam\n",
        None,
    );
    // Source loops borrow strings. Private literal storage may copy static
    // descriptor references while constructing those borrowed string proxies.
    assert!(source_iteration_lines(&rust).all(|line| !line.contains(".copied()")));
    for body in [
        // `const alias = s;` is a readonly borrow; transferring it is rejected below.
        "take(s);",
        "s = \"Sam\";",
        "retain((): string => { return clone(s); });",
    ] {
        reject(
            &format!(
                "function take(s: string): string {{ return s; }} function retain(f: Function<string>) {{}} function main() {{ const a = [\"Pat\"]; for (let s of a) {{ {body} }} }}"
            ),
            "borrowed-element",
        );
    }
    reject(
        "function take(s: string): string { return s; } function main() { const a = [\"Pat\"]; for (const s of a) { const alias = s; take(alias); } }",
        "borrowed-place",
    );
    reject(
        "type U = { n: int; }; function main() { const a: U[] = [{ n: 1 }]; for (const u of a) { u.n = 2; } }",
        "borrowed-element",
    );
}

#[test]
fn element_argument_loans_protect_the_entire_array() {
    execute(
        "type U = { n: int; }; function pair(a: U, b: U): int { return a.n + b.n; } function main() { const a: U[] = [{ n: 1 }, { n: 2 }]; console.log(pair(a[0], a[1])); }",
        "3\n",
        None,
    );
    for call in ["pair(a[0], a[1]);", "grow(a[0], a);"] {
        reject(
            &format!(
                "type U = {{ n: int; }}; function pair(a: U, b: U) {{ a.n = b.n; }} function grow(u: U, a: U[]) {{ a.push({{ n: u.n }}); }} function main() {{ const a: U[] = [{{ n: 1 }}, {{ n: 2 }}]; {call} }}"
            ),
            "conflicting-access",
        );
    }
    reject(
        "type U = { n: int; }; function change(u: U) { u.n = 2; } function main() { const a: U[] = [{ n: 1 }]; for (const u of a) { change(a[0]); } }",
        "iteration-conflict",
    );
}

#[test]
fn indexed_record_writes_evaluate_operands_before_mutating() {
    execute(
        "type U = { name: string; n: int; }; function index(): int { console.log(1); return 0; } function text(): string { console.log(2); return \"Sam\"; } function grow(a: U[]): int { a.push({ name: \"Alex\", n: 7 }); return 7; } function main() { const a: U[] = [{ name: \"Pat\", n: 0 }]; a[index()].name = text(); a[0].n = grow(a); console.log(a[0].name); console.log(a[0].n); console.log(a.length); a.push(clone(a[0])); console.log(a[2].name); }",
        "1\n2\nSam\n7\n2\nSam\n",
        None,
    );
    reject(
        "type U = { n: int; }; function take(a: U[]): int { const moved = a; return 7; } function main() { const a: U[] = [{ n: 1 }]; a[0].n = take(a); }",
        "transferred",
    );
}

#[test]
fn captures_and_owned_callback_contracts_preserve_element_access() {
    execute(
        "type U = { n: int; }; function use(f: Mutable<Function<void, [u: Mutable<U>]>>, u: U) { f(u); } function take(f: Once<Function<U, [u: Owned<U>]>>, u: U): U { return f(u); } function main() { const a: U[] = [{ n: 1 }]; use((u: U): void => { u.n = 7; }, a[0]); const u = take((u: U): U => { return u; }, clone(a[0])); console.log(u.n); console.log(a[0].n); }",
        "7\n7\n",
        None,
    );
    execute(
        "function use(f: Mutable<Function<string>>): string { return f(); } function main() { for (const s of [\"Pat\", \"Sam\"]) { const f = (): string => { return s.toLowerCase(); }; console.log(use(f)); } }",
        "pat\nsam\n",
        None,
    );
    execute(
        "type U = { name: string; }; function use(f: Mutable<Function<string>>): string { return f(); } function main() { const a: U[] = [{ name: \"Pat\" }]; for (const u of a) { const f = (): string => { return u.name.trim(); }; console.log(use(f)); } }",
        "Pat\n",
        None,
    );
    execute(
        "function make(a: string[]): () => string { return (): string => { return clone(a[0]); }; } function identity(f: Function<string[], [a: Owned<string[]>]>): (a: Owned<string[]>) => string[] { return f; } function main() { const f = make([\"Pat\"]); console.log(f()); console.log(f()); const g = identity((a: string[]): string[] => { return a; }); const a = g([\"Sam\"]); console.log(a[0]); }",
        "Pat\nPat\nSam\n",
        None,
    );
    reject(
        "type U = { n: int; }; function main() { const a: U[] = [{ n: 1 }]; for (const u of a) { const f = (): void => { u.n = 2; }; } }",
        "borrowed-element",
    );
    reject(
        "type U = { n: int; }; function main() { const a: U[] = [{ n: 1 }]; const f = (): int => { return a[0].n; }; a[0].n = 2; }",
        "capture-conflict",
    );
}

#[test]
fn whole_owned_arrays_follow_moves_reinitialization_and_loop_joins() {
    execute(
        "function take(a: string[]): string[] { return a; } function main() { let a = [\"Pat\"]; const b = take(a); a = [\"Sam\"]; console.log(b[0]); console.log(a[0]); for (const n of [1, 2]) { const moved = a; a = [\"Alex\"]; console.log(moved[0]); } console.log(a[0]); }",
        "Pat\nSam\nSam\nAlex\nAlex\n",
        Some(&[("take.a", Mode::Consume)]),
    );
    reject(
        "function main() { const a = [\"Pat\"]; const moved = a; console.log(a[0]); }",
        "transferred",
    );
    reject(
        "function main() { const a = [\"Pat\"]; while (true) { const moved = a; } }",
        "loop-transfer",
    );
}

#[test]
fn owned_element_bounds_checks_remain_active_under_optimization() {
    for statement in [
        "console.log(a[-1].name);",
        "a[1].name = \"Sam\";",
        "console.log(clone(a[1]).name);",
    ] {
        // Clone results need a named binding for field access in this subset.
        let statement = statement.replace(
            "console.log(clone(a[1]).name);",
            "const u = clone(a[1]); console.log(u.name);",
        );
        let source = format!(
            "type U = {{ name: string; }}; function main() {{ const a: U[] = [{{ name: \"Pat\" }}]; {statement} }}"
        );
        let dir = support::TempDirectory::new();
        let path = dir.0.join("program.rs");
        std::fs::write(&path, check(&source).unwrap().emit_rust()).unwrap();
        let start = source.rfind("a[").unwrap();
        let end = start + source[start..].find(']').unwrap() + 1;
        for optimize in ["0", "3"] {
            let binary = dir.0.join("program");
            support::success(&support::run(
                support::rustc()
                    .arg("--edition=2024")
                    .arg(&path)
                    .arg("-C")
                    .arg(format!("opt-level={optimize}"))
                    .arg("-C")
                    .arg("overflow-checks=no")
                    .arg("-o")
                    .arg(&binary),
                60,
            ));
            let output = support::run(&mut std::process::Command::new(&binary), 10);
            assert!(!output.status.success());
            assert!(String::from_utf8(output.stderr).unwrap().contains(&format!(
                "array index out of bounds at source bytes {start}..{end}"
            )));
        }
    }
}

#[test]
fn owned_array_analysis_is_backend_independent_and_depth_bounded() {
    let dir = support::TempDirectory::new();
    let path = dir.source(include_str!("fixtures/owned-arrays/users.tsr"));
    for command in ["check", "emit"] {
        support::success(&support::run(
            support::cli().arg(command).arg(&path).env("PATH", ""),
            10,
        ));
    }
    let loops = (0..100)
        .map(|i| format!("for (const s{i} of [\"Pat\"]) {{"))
        .collect::<String>();
    let source = format!(
        "function main() {{ {loops} console.log(s99); {} }}",
        "}".repeat(100)
    );
    let rust = check(&source).unwrap().emit_rust();
    assert_eq!(source_iteration_lines(&rust).count(), 100);
    assert!(source_iteration_lines(&rust).all(|line| !line.contains(".copied()")));
    reject(
        "function main() { const a: void[][] = []; }",
        "array-element",
    );
    reject(
        "type U = { n: int; }; type Box = { value: void; }; function main() {}",
        "unsupported",
    );
}
