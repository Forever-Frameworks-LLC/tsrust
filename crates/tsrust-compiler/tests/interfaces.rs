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

// An interface can extend a record type that its file imports.
#[test]
fn interfaces_extend_imported_record_types() {
    let directory = support::TempDirectory::new();
    std::fs::write(
        directory.0.join("types.tsr"),
        "type Hidden = { depth: int; };\nexport interface Named { name: string; inner?: Hidden; }\nexport interface Player extends Named { score: int; }\nexport function top(): Player { return { name: \"Pat\", score: 3 }; }\n",
    )
    .unwrap();
    let main = directory.0.join("main.tsr");
    std::fs::write(
        &main,
        "import { strictEqual } from \"std:testing\";\nimport { Player, Named, top } from \"./types.tsr\";\ninterface Local extends Named { extra?: int; }\nfunction main() {\n    const player: Player = top();\n    const local: Local = { name: \"x\" };\n    strictEqual(player.name + String(player.score) + local.name, \"Pat3x\");\n    strictEqual(local.inner?.depth === undefined, true);\n}\n",
    )
    .unwrap();
    support::success(&support::run(support::cli().arg("run").arg(&main), 60));
    let bad = directory.0.join("bad.tsr");
    std::fs::write(&bad, "import { top } from \"./types.tsr\";\ninterface Local extends Missing { extra: int; }\nfunction main() { top(); }\n").unwrap();
    let output = support::run(support::cli().arg("check").arg(&bad), 60);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[type]"), "{}", String::from_utf8_lossy(&output.stderr));
}
