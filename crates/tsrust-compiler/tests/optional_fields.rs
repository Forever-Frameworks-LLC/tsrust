mod support;

// `name?: Type` declares a record field that a literal may leave out. The
// field is always present in the record and holds `undefined` when omitted.
// The independent Node run of the fixture passes.
#[test]
fn optional_fields_can_be_omitted_and_hold_undefined() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/optional-fields/records.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

// A record has a fixed shape: an omitted optional field is a key whose value
// is undefined, so it is listed and printed.
#[test]
fn omitted_optional_fields_stay_in_the_record_shape() {
    support::execute(
        "type Point = { x: int; y?: int; }; function main() { const point: Point = { x: 1 }; console.log(point, Object.keys(point).join(\",\"), \"y\" in point, point.y); }",
        "{ x: 1, y: undefined } x,y true undefined\n",
        None,
    );
}

#[test]
fn optional_fields_keep_static_contracts() {
    for (source, code) in [
        ("type P = { x: int; y?: int; }; function main() { const p: P = { y: 1 }; }", "type"),
        ("type P = { x: int; y?: int; }; function main() { const p: P = { x: 1, y: \"a\" }; }", "type"),
        ("type P = { x: int; y?: int; }; function main() { const p: P = { x: 1 }; const n: int = p.y; }", "type"),
        ("type P = { x: int; y?: int; }; function main() { const p: P = { x: 1 }; console.log(p.y + 1); }", "type"),
        ("type P = { x: int; y?: int; }; function main() { const p: P = { x: 1, z: 2 }; }", "type"),
        ("type P = { x?: ; }; function main() { }", "syntax"),
        ("type P = { x ?: int; y?: int; y?: int; }; function main() { }", "duplicate"),
    ] {
        support::reject(source, code);
    }
}
