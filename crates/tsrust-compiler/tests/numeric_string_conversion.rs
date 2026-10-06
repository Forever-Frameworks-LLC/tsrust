mod support;
#[test]
fn complete_numeric_text_converts_with_shared_scalar_union_and_enum_rules() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/numeric-string-conversion/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn repeated_string_identity_hint_is_bounded() {
    let directory=support::TempDirectory::new();
    let source=format!("function main(){{const text=\"1\"{};console.log(Number(text));}}", ".valueOf()".repeat(48));
    let output=support::run(support::cli().arg("emit").arg(directory.source(&source)),15);
    support::success(&output);
}
