mod support;

#[test]
fn polymorphic_values_preserve_array_patterns_and_one_capture_environment() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/generic-function-values/array-patterns.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn polymorphic_values_preserve_optional_prefix_and_generic_rest_packs() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/generic-function-values/rest-prefix.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
