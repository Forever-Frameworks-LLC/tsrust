mod support;

// `text[position]` is the character at a position, as a new string. A
// position outside the string is a catchable range error, like an array
// index; outside any try it ends the program with the same kind of report. The independent Node run of the fixture passes.
#[test]
fn string_indexing_selects_characters_and_checks_bounds() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/string-indexing/positions.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

// Positions count whole Unicode characters, as charAt, at and for...of do.
#[test]
fn string_indexing_counts_unicode_characters() {
    support::execute(
        "function main() { const text = \"a€😀b\"; console.log(text[0], text[1], text[2], text[3], text[2] === text.charAt(2)); }",
        "a € 😀 b true\n",
        None,
    );
}

#[test]
fn uncaught_string_index_reports_its_position() {
    let directory = support::TempDirectory::new();
    let path = directory.source("function main() { const text = \"abc\"; let position = 7; console.log(text[position]); }");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("string index out of range at source bytes"), "{stderr}");
}

#[test]
fn string_indexing_keeps_static_contracts() {
    for (source, code) in [
        ("function main() { const text = \"abc\"; const a = text[\"0\"]; }", "type"),
        ("function main() { const text = \"abc\"; const a = text[1.5]; }", "type"),
        ("function main() { const text = \"abc\"; const a: int = text[0]; }", "type"),
        ("function main() { let text = \"abc\"; text[0] = \"x\"; }", "unsupported"),
        ("function main() { let text = \"abc\"; text[0] += \"x\"; }", "unsupported"),
        ("function main() { const n = 5; const a = n[0]; }", "type"),
        ("function pick(text: string | undefined): string { return text[0]; } function main() { pick(\"ab\"); }", "type"),
        ("function main() { let text = \"abc\"; text[0]++; }", "unsupported"),
        ("function main() { let text = \"abc\"; const a = (text[0] = \"x\"); }", "unsupported"),
        ("function main() { let text = \"abc\"; text[0] ||= \"x\"; }", "unsupported"),
    ] {
        support::reject(source, code);
    }
}

// A statically absent receiver: the selected arm is dead but must keep its
// type so the result view can be inferred.
#[test]
fn absent_optional_selections_compare_with_undefined() {
    support::execute(
        "function main() { const values: string[] | undefined = undefined; const text: string | undefined = undefined; console.log(values?.[0] === undefined, text?.toUpperCase() === undefined); }",
        "true true\n",
        None,
    );
}

// With a present text, an optional position is checked like a plain one.
#[test]
fn optional_string_position_checks_bounds_when_present() {
    support::execute(
        "function pick(text: string | undefined, position: int): string { try { const found = text?.[position]; return found ?? \"absent\"; } catch (error) { return \"range\"; } } function main() { console.log(pick(\"ab\", 1), pick(undefined, 9), pick(\"ab\", 9), pick(\"ab\", -1)); }",
        "b absent range range\n",
        None,
    );
}
