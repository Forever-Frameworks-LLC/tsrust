mod support;

fn native(source: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn nested_declarations_hoist_recurse_shadow_and_pass_as_values() {
    native(include_str!("fixtures/nested-function-declarations/hoisting.tsr"));
}

#[test]
fn nested_declarations_borrow_captures_and_transfer_escaping_environments() {
    native(include_str!("fixtures/nested-function-declarations/captures.tsr"));
}

#[test]
fn nested_declarations_keep_generic_default_rest_and_deep_scope_signatures() {
    native(include_str!("fixtures/nested-function-declarations/signatures.tsr"));
}
