mod support;

// `type Name = Type;` names any written type. The alias and its expansion
// are the same type. The independent Node run of the fixture passes.
#[test]
fn type_aliases_name_unions_arrays_tuples_collections_and_functions() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/type-aliases/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn an_alias_and_its_expansion_are_one_type() {
    tsrust_compiler::check("type Id = int | string; function take(id: int | string): int { return 1; } function give(): Id { return 2; } function main() { const a: Id = give(); take(a); const b: int | string = a; take(b); }").unwrap();
    tsrust_compiler::check("type Ints = int[]; function sum(values: int[]): int { return values.length; } function main() { const a: Ints = [1]; sum(a); }").unwrap();
}

#[test]
fn type_aliases_keep_static_contracts() {
    for (source, code) in [
        ("type Id = int | string; function main() { const a: Id = true; }", "type"),
        ("type Ints = int[]; function main() { const a: Ints = [\"x\"]; }", "type"),
        ("type A = B; type B = A; function main() { }", "recursive-alias"),
        ("type A = A[]; function main() { }", "recursive-alias"),
        ("type Id = int; type Id = string; function main() { }", "duplicate"),
        ("type Id = int; type Id = { x: int; }; function main() { }", "duplicate"),
        ("type int = string; function main() { }", "duplicate"),
        ("type Id = Missing | int; function main() { const a: Id = 1; }", "type"),
        ("type Id = ; function main() { }", "syntax"),
        ("function main() { type Id = int; }", "syntax"),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start, "{source}");
    }
}

// An error in a use of an alias points at the use, not at the alias.
#[test]
fn alias_diagnostics_point_at_the_use() {
    let source = "type Id = int | string; function main() { const a: Id = true; }";
    let error = tsrust_compiler::check(source).err().unwrap();
    assert!(error.span.start > source.find("function").unwrap(), "{error:?}");
}

// A view of an owned union call result stays valid while it is compared.
#[test]
fn owned_union_call_results_compare_in_place() {
    let source = r#"import { strictEqual } from "std:testing";
function first(ids: string[]): string | undefined { if (ids.length === 0) { return undefined; } return ids[0].slice(0); }
function main() {
    const ids = ["a"];
    const none: string[] = [];
    strictEqual(first(ids) === "a", true);
    strictEqual(first(ids) !== "b", true);
    strictEqual(first(none) === undefined, true);
    strictEqual(first(ids) !== undefined, true);
    strictEqual("a" === first(ids), true);
    strictEqual(first(ids) === first(ids), true);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

// An exported alias is usable in an importing module, also when it names
// types of its own module that the importer did not import.
#[test]
fn exported_aliases_expand_in_importing_modules() {
    let directory = support::TempDirectory::new();
    std::fs::write(
        directory.0.join("types.tsr"),
        "export type Id = int | string;\nexport type Ids = Id[];\ntype Hidden = int;\ntype Point = { x: Hidden; };\nexport type Points = Point[];\nexport function origin(): Points { return [{ x: 0 }]; }\nexport function kind(id: Id): string { return typeof id === \"string\" ? \"text\" : \"number\"; }\n",
    )
    .unwrap();
    let main = directory.0.join("main.tsr");
    std::fs::write(
        &main,
        "import { strictEqual } from \"std:testing\";\nimport { Id, Ids, Points, origin, kind } from \"./types.tsr\";\ntype Local = Ids;\nfunction main() {\n    const ids: Local = [1, \"b\"];\n    const one: Id = 2;\n    strictEqual(kind(one) + kind(ids[1]), \"numbertext\");\n    const points: Points = origin();\n    strictEqual(points[0].x, 0);\n}\n",
    )
    .unwrap();
    let output = support::run(support::cli().arg("run").arg(&main), 60);
    support::success(&output);
    for (body, code) in [
        ("import { Hidden } from \"./types.tsr\";\nfunction main() { const a: Hidden = 1; }\n", "module-private"),
        ("import { Id } from \"./types.tsr\";\ntype Id = int;\nfunction main() { }\n", "duplicate"),
        ("import { Id } from \"./types.tsr\";\nfunction main() { const a: Id = true; }\n", "type"),
    ] {
        let bad = directory.0.join("bad.tsr");
        std::fs::write(&bad, body).unwrap();
        let output = support::run(support::cli().arg("check").arg(&bad), 60);
        assert!(!output.status.success(), "{body}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")), "{body}: {}", String::from_utf8_lossy(&output.stderr));
    }
}
