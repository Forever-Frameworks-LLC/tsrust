mod support;

#[test]
fn native_byte_type_imports_and_result_payloads_execute() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-owned-bytes/values.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}
