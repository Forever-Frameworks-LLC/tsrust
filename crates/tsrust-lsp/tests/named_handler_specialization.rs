use tsrust_lsp::{project_diagnostics::source_id, workspace::Workspace};

#[test]
fn named_handler_project_checks_without_internal_failure_and_keeps_ownership_errors() {
    let mut workspace = Workspace::default();
    let owner = source_id("file:///ui-b4/owner.tsr").unwrap();
    let main = source_id("file:///ui-b4/main.tsr").unwrap();
    workspace.open(
        owner,
        1,
        include_str!("../../../experiments/ui-tsrust-owner-handles/witnesses/owner.tsr").to_owned(),
    );
    workspace.open(
        main.clone(),
        1,
        include_str!(
            "../../../experiments/ui-tsrust-owner-handles/witnesses/named-self-replacement.tsr"
        )
        .to_owned(),
    );
    assert!(workspace.snapshot().check_with_diagnostics(&main).is_ok());
    for (name, text) in [
        ("empty.tsr", ""),
        ("comment.tsr", "// Inert discovery fixture.\n"),
    ] {
        let id = source_id(&format!("file:///ui-b4/{name}")).unwrap();
        workspace.open(id.clone(), 1, text.into());
        assert!(workspace.snapshot().check_with_diagnostics(&id).is_ok());
        assert!(workspace.snapshot().check_with_diagnostics(&main).is_ok());
    }
    let invalid = source_id("file:///ui-b4/invalid.tsr").unwrap();
    let source =
        include_str!("../../../experiments/ui-state-lifecycle/witnesses/consumed-owner.tsr")
            .lines()
            .filter(|line| !line.contains("@tsrust ignore"))
            .collect::<Vec<_>>()
            .join("\n");
    workspace.open(invalid.clone(), 1, source);
    let result = workspace.snapshot().check_with_diagnostics(&invalid);
    assert!(result.is_err());
    assert!(
        result
            .err()
            .unwrap()
            .iter()
            .any(|error| error.code == "transferred")
    );
}
