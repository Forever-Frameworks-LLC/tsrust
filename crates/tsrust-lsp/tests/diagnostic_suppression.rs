use tsrust_lsp::{diagnostics, workspace::Workspace, project_diagnostics::{source_id, convert_all}};

const SOURCE: &str = "function main() {\n// @tsrust ignore - intentional editor fixture\nconst value: int =\n  \"wrong\";\n}\n";

#[test]
fn ignored_multiline_statement_has_no_single_file_diagnostics() {
    assert!(tsrust_compiler::check(SOURCE).is_err());
    assert!(diagnostics(SOURCE, "file:///ignore.tsr").unwrap().is_empty());
}

#[test]
fn ignored_project_diagnostics_use_the_unsaved_source_snapshot() {
    let id = source_id("file:///tsrust-ignore-fixture/main.tsr").unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id.clone(), 1, SOURCE.into());
    let snapshot = workspace.snapshot();
    let errors = snapshot.check_with_diagnostics(&id).err().unwrap();
    assert!(!errors.is_empty(), "Strict project checking remains invalid");
    let published = convert_all(&snapshot, &errors).unwrap();
    assert!(published[&id].is_empty());
}
