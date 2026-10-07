mod support;

use support::{execute, reject};
use tsrust_compiler::Mode;

#[test]
fn original_numeric_user_example() {
    let rust = execute(
        include_str!("fixtures/integers/original_user.tsr"),
        "pat\n",
        Some(&[("normalize.user", Mode::Read)]),
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn decimal_boundaries_and_scalar_copy_contracts() {
    execute(
        r#"
function outer(value: int): int { return identity(value); }
function identity(value: int): int {
    const saved = value;
    value;
    console.log(value);
    return saved;
}
function pair(first: int, second: int) { console.log(first); console.log(second); }
function unused(value: int) {}
function main() {
    const min: int = -9223372036854775808;
    const max = 9223372036854775807;
    const copy = outer(min);
    console.log(min);
    console.log(copy);
    pair(max, max);
    console.log(max);
    console.log(00042);
    console.log(-00042);
    console.log(-0);
    unused(0);
}
"#,
        "-9223372036854775808\n-9223372036854775808\n-9223372036854775808\n9223372036854775807\n9223372036854775807\n9223372036854775807\n42\n-42\n0\n",
        Some(&[
            ("outer.value", Mode::Copy),
            ("identity.value", Mode::Copy),
            ("pair.first", Mode::Copy),
            ("pair.second", Mode::Copy),
            ("unused.value", Mode::Copy),
        ]),
    );
}

#[test]
fn mixed_fields_copy_mutate_clone_and_move() {
    execute(
        r#"
type User = { id: int; name: string; };
function getId(user: User): int { return user.id; }
function change(user: User, id: int) { user.id = id; user.name = "Sam"; }
function take(user: User): User { user.id = -1; return user; }
function create(id: int): User { return { name: "Pat", id: id }; }
function main() {
    const user = create(7);
    const id = user.id;
    user.id;
    user.id = user.id;
    const backup = clone(user);
    change(user, 9);
    console.log(id);
    console.log(getId(backup));
    console.log(user.id);
    console.log(backup.name);
    console.log(user.name);
    const moved = take(user);
    console.log(moved.id);
    console.log(backup.id);
}
"#,
        "7\n7\n9\nPat\nSam\n-1\n7\n",
        Some(&[
            ("getId.user", Mode::Read),
            ("change.user", Mode::Mutate),
            ("change.id", Mode::Copy),
            ("take.user", Mode::Consume),
            ("create.id", Mode::Copy),
        ]),
    );
}

#[test]
fn copied_arguments_release_access_before_later_arguments() {
    let rust = execute(
        r#"
type Item = { id: int; };
function change(item: Item): int { item.id = 8; return item.id; }
function show(first: int, second: int) { console.log(first); console.log(second); }
function mutate(id: int, item: Item) { item.id = 9; console.log(id); }
function consume(id: int, item: Item): Item { console.log(id); return item; }
function main() {
    const item: Item = { id: 7 };
    show(item.id, change(item));
    mutate(item.id, item);
    const moved = consume(item.id, item);
    console.log(moved.id);
}
"#,
        "7\n8\n8\n9\n9\n",
        None,
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn scalar_fields_do_not_relax_record_ownership_or_existing_loans() {
    for (source, code) in [
        (
            r#"type Item = { id: int; };
function main() { const x: Item = { id: 1 }; const y = x; console.log(x.id); }"#,
            "transferred",
        ),
        (
            r#"type Item = { id: int; };
function take(x: Item): Item { return x; }
function main() { const x: Item = { id: 1 }; take(x); console.log(x.id); }"#,
            "transferred",
        ),
        (
            r#"type Item = { id: int; };
function both(x: Item, id: int) { x.id = id; }
function main() { const x: Item = { id: 1 }; both(x, x.id); }"#,
            "conflicting-access",
        ),
        (
            r#"type Item = { id: int; };
function both(x: Item, id: int): Item { return x; }
function main() { const x: Item = { id: 1 }; both(x, x.id); }"#,
            "transferred",
        ),
        (
            r#"type Item = { id: int; };
function take(x: Item): int { const saved = x; return saved.id; }
function main() { const x: Item = { id: 1 }; x.id = take(x); }"#,
            "transferred",
        ),
        (
            r#"type Item = { id: int; };
function change(x: Item): int { x.id = 2; return x.id; }
function both(x: Item, id: int) { console.log(x.id); }
function main() { const x: Item = { id: 1 }; both(x, change(x)); }"#,
            "conflicting-access",
        ),
    ] {
        let diagnostic = reject(source, code);
        assert!(!diagnostic.related.is_empty());
    }
}

#[test]
fn integer_type_and_syntax_errors_are_frontend_errors() {
    for (source, code) in [
        (r#"function main() { const x: int = "1"; }"#, "type"),
        (r#"function main() { const x: string = 1; }"#, "type"),
        (
            r#"function f(x: int) {} function main() { f("1"); }"#,
            "type",
        ),
        (
            r#"function f(x: string) {} function main() { f(1); }"#,
            "type",
        ),
        (
            r#"function f(): int { return "1"; } function main() {}"#,
            "type",
        ),
        (r#"function f(): int {} function main() {}"#, "type"),
        (
            r#"type Item = { id: int; }; function main() { const x: Item = { id: "1" }; }"#,
            "type",
        ),
        (
            r#"type Item = { name: string; }; function main() { const x: Item = { name: 1 }; }"#,
            "type",
        ),
        (
            r#"type Item = { id: int; }; function main() { const x: Item = { id: 1 }; x.id = "2"; }"#,
            "type",
        ),
        (
            r#"function main() { const x = 1; console.log(x.name); }"#,
            "type",
        ),
        (r#"function main() { const x = 1; x.trim(); }"#, "type"),
        (r#"function main() { const x = 1; x = 2; }"#, "unsupported"),
        (r#"function main() { clone(1); }"#, "unsupported"),
        (
            r#"type int = { id: int; }; function main() {}"#,
            "duplicate",
        ),
        (r#"function main() { const x = 1e+; }"#, "syntax"),
        (r#"function main() { const x = 1n; }"#, "unsupported"),
    ] {
        reject(source, code);
    }
}

#[test]
fn out_of_range_literals_have_exact_source_spans() {
    for literal in [
        "9223372036854775808",
        "-9223372036854775809",
        &"9".repeat(1000),
    ] {
        let source = format!("function main() {{ /* café */ const x = {literal}; }}");
        let diagnostic = reject(&source, "integer-range");
        assert_eq!(&source[diagnostic.span.start..diagnostic.span.end], literal);
    }
}

#[test]
fn invalid_integer_build_fails_before_backend_lookup() {
    let directory = support::TempDirectory::new();
    let source = directory.source("function main() { console.log(9223372036854775808); }");
    let binary = directory.0.join("program");
    let result = support::run(
        support::cli()
            .arg("build")
            .arg(source)
            .arg("-o")
            .arg(&binary)
            .env("PATH", ""),
        10,
    );
    assert_eq!(result.status.code(), Some(1));
    let error = String::from_utf8(result.stderr).unwrap();
    assert!(error.contains("integer-range"), "{error}");
    assert!(!error.contains("rustup"));
    assert!(!binary.exists());
}
