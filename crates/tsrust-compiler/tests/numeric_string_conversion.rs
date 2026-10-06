mod support;
#[test]
fn complete_numeric_text_converts_with_shared_scalar_union_and_enum_rules() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/numeric-string-conversion/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
