mod support;
use support::session::Session;

#[test]
fn switch_narrowing_and_snapshot_mutation_agree_with_the_compiler() {
    let source = "function probe(value: ReadOnly<int | string>) { switch(value) { case \"x\": console.log(value.toUpperCase()); break; default: break; } } function main() {}";
    tsrust_compiler::check(source).unwrap();
    let mut session = Session::start(&[]);
    session.open("main.tsr", source);
    assert_eq!(session.wait_for_diagnostics("main.tsr"), serde_json::json!([]));
    let at = source.find("toUpperCase").unwrap();
    let hover = session.hover("main.tsr", source, at + 2).unwrap_or_default();
    assert!(hover.contains("toUpperCase"), "{hover}");
    let members = session.completion("main.tsr", source, at).unwrap();
    assert!(members.contains(&"toUpperCase".to_owned()), "{members:?}");

    let invalid = "function main() { let value: int | boolean = true; switch(value) { case (value = 7): break; case true: const wrong: boolean = value; break; } }";
    assert!(tsrust_compiler::check(invalid).is_err());
    session.change("main.tsr", invalid);
    let diagnostics = session.wait_for_diagnostics("main.tsr");
    assert!(diagnostics.as_array().unwrap().iter().any(|diagnostic| diagnostic["code"] == "type"), "{diagnostics}");
}
