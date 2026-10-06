mod support;

// `interface Name { ... }` declares a record type. `extends` adds the fields
// of other record types in front of its own.
#[test]
fn interfaces_declare_record_types_with_inherited_fields() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/interfaces/shapes.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

// A type declared with a body and an interface are the same kind of type.
#[test]
fn interfaces_and_record_types_extend_each_other() {
    support::execute(
        "type Base = { id: int; }; interface Item extends Base { label: string; } interface Late extends Later { extra: int; } interface Later { first: int; } function main() { const item: Item = { id: 1, label: \"a\" }; const late: Late = { first: 2, extra: 3 }; console.log(item, late); }",
        "{ id: 1, label: 'a' } { first: 2, extra: 3 }\n",
        None,
    );
}

#[test]
fn interfaces_keep_static_contracts() {
    for (source, code) in [
        ("interface A { x: int; } function main() { const a: A = { }; }", "type"),
        ("interface A { x: int; } interface B extends A { y: int; } function main() { const b: B = { y: 1 }; }", "type"),
        ("interface A { x: int; } interface B extends A { y: int; } function take(a: A): int { return a.x; } function main() { const b: B = { x: 1, y: 2 }; take(b); }", "type"),
        ("interface A extends Missing { x: int; } function main() { }", "type"),
        ("interface A extends B { x: int; } interface B extends A { y: int; } function main() { }", "recursive-interface"),
        ("interface A { x: int; } interface B extends A { x: string; } function main() { }", "duplicate"),
        ("interface A { x: int; } interface A { y: int; } function main() { }", "duplicate"),
        ("interface A { x: int; } type A = int; function main() { }", "duplicate"),
        ("type Id = int; interface A extends Id { x: int; } function main() { }", "type"),
        ("interface A { run(): int; } function main() { }", "unsupported"),
        ("interface A extends Box<int> { x: int; } interface Box<T> { value: T; } function main() { }", "unsupported"),
        ("function main() { interface A { x: int; } }", "syntax"),
        ("interface { x: int; } function main() { }", "syntax"),
    ] {
        support::reject(source, code);
    }
}
