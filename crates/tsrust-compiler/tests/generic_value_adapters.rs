mod support;

fn native(source: &str, expected: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn generic_value_adapters_preserve_borrowed_and_owned_factory_state() {
    native(include_str!("fixtures/generic-value-adapters/contracts.tsr"), "11\n12\n13\n");
}

#[test]
fn generic_value_adapters_supply_contextual_array_callbacks() {
    native(include_str!("fixtures/generic-value-adapters/callbacks.tsr"), "11\n12\n13\n");
}

#[test]
fn generic_value_adapters_support_record_and_array_storage() {
    native(include_str!("fixtures/generic-value-adapters/storage.tsr"), "11\n21\n22\n");
}
