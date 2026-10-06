mod support;

// `text + value` and `text += value` join a string with an int, number,
// boolean, null or undefined, converted as `String(value)` converts it.
// The independent Node run of the fixture passes.
#[test]
fn strings_concatenate_with_scalar_values() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/string-concatenation/scalars.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn string_concatenation_keeps_static_contracts() {
    for (source, code) in [
        ("function main() { const a = \"a\" + [1]; }", "type"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; const a = \"a\" + p; }", "type"),
        ("function main() { const a: int = \"a\" + 1; }", "type"),
        ("function main() { const a = \"a\" - 1; }", "type"),
        ("function main() { const a = \"a\" * 2; }", "type"),
        ("function main() { let n = 1; n += \"a\"; }", "type"),
        ("function main() { let text = \"a\"; text -= 1; }", "type"),
        ("function main() { const f = (): int => 1; const a = \"a\" + f; }", "type"),
    ] {
        support::reject(source, code);
    }
}
