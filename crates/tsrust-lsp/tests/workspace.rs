use tsrust_compiler::project::SourceId;
use tsrust_lsp::workspace::Workspace;
fn id(name: &str) -> SourceId {
    SourceId(format!("/tsrust-virtual-workspace/{name}.tsr"))
}
fn open(workspace: &mut Workspace, name: &str, version: i32, text: &str) {
    workspace.open(id(name), version, text.into());
}
#[test]
fn unsaved_dependencies_are_checked_without_disk_sources() {
    let mut workspace = Workspace::default();
    open(
        &mut workspace,
        "main",
        1,
        "import { value } from './a.tsr';function main(){console.log(value());}",
    );
    open(
        &mut workspace,
        "a",
        7,
        "export function value():int{return 2;}",
    );
    let snapshot = workspace.snapshot();
    let checked = snapshot
        .check(&id("main"))
        .unwrap_or_else(|e| panic!("{}", e.message));
    assert_eq!(checked.graph.modules.len(), 2);
    assert_eq!(snapshot.version(&id("a")), Some(7));
    assert!(snapshot.sources()[&id("a")].contains("return 2"));
    assert!(workspace.register(id("main"), &snapshot));
    assert_eq!(workspace.affected(&id("a")), vec![id("main")]);
}
#[test]
fn stale_results_are_rejected_after_dependency_edit_close_and_reopen() {
    let mut workspace = Workspace::default();
    open(
        &mut workspace,
        "main",
        1,
        "import { value } from './a.tsr';function main(){value();}",
    );
    open(&mut workspace, "a", 1, "export function value(){}");
    let original = workspace.snapshot();
    original.check(&id("main")).unwrap();
    assert!(!workspace.change(&id("a"), 1, "ignored".into()));
    assert!(workspace.current(&original));
    assert!(workspace.change(&id("a"), 2, "export function value(){}".into()));
    assert!(!workspace.current(&original));
    assert!(!workspace.register(id("main"), &original));
    let edited = workspace.snapshot();
    edited.check(&id("main")).unwrap();
    assert!(workspace.close(&id("a")));
    assert!(!workspace.current(&edited));
    open(&mut workspace, "a", 2, "export function value(){}");
    assert!(!workspace.current(&edited));
    assert!(workspace.current(&workspace.snapshot()));
}
#[test]
fn unrelated_changes_do_not_invalidate_project_results() {
    let mut workspace = Workspace::default();
    open(&mut workspace, "main", 1, "function main(){}");
    let snapshot = workspace.snapshot();
    snapshot.check(&id("main")).unwrap();
    open(&mut workspace, "other", 1, "function main(){}");
    assert!(workspace.current(&snapshot));
    workspace.register(id("main"), &snapshot);
    assert!(workspace.affected(&id("other")).is_empty());
}
#[test]
fn missing_imports_are_tracked_and_opening_them_invalidates_failure() {
    let mut workspace = Workspace::default();
    open(
        &mut workspace,
        "main",
        1,
        "import { value } from './new.tsr';function main(){value();}",
    );
    let missing = workspace.snapshot();
    assert!(missing.check(&id("main")).is_err());
    assert!(workspace.register(id("main"), &missing));
    assert_eq!(workspace.affected(&id("new")), vec![id("main")]);
    open(&mut workspace, "new", 1, "export function value(){}");
    assert!(!workspace.current(&missing));
    assert!(workspace.snapshot().check(&id("main")).is_ok());
}
#[test]
fn watch_invalidation_preserves_snapshot_and_removes_entry_tracking() {
    let mut workspace = Workspace::default();
    open(&mut workspace, "main", 1, "function main(){}");
    let snapshot = workspace.snapshot();
    snapshot.check(&id("main")).unwrap();
    workspace.register(id("main"), &snapshot);
    workspace.invalidate_file(&id("main"));
    assert!(!workspace.current(&snapshot));
    assert!(snapshot.sources()[&id("main")].contains("function main"));
    workspace.remove_entry(&id("main"));
    assert!(workspace.affected(&id("main")).is_empty());
}

#[test]
fn disk_fallback_is_cached_and_unsaved_dependency_takes_precedence() {
    let directory = std::env::temp_dir().join(format!(
        "tsrust-workspace-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let cleanup = Cleanup(directory.clone());
    let entry = directory.join("main.tsr");
    let dependency = directory.join("a.tsr");
    std::fs::write(
        &entry,
        "import { value } from './a.tsr';function main(){console.log(value());}",
    )
    .unwrap();
    std::fs::write(&dependency, "export function value():int{return 2;}").unwrap();
    let entry = SourceId(
        std::fs::canonicalize(&entry)
            .unwrap()
            .to_str()
            .unwrap()
            .into(),
    );
    let dependency = SourceId(
        std::fs::canonicalize(&dependency)
            .unwrap()
            .to_str()
            .unwrap()
            .into(),
    );
    let mut workspace = Workspace::default();
    let disk = workspace.snapshot();
    assert!(disk.check(&entry).is_ok());
    std::fs::write(
        &dependency.0,
        "export function value():int{return missing;}",
    )
    .unwrap();
    assert!(disk.check(&entry).is_ok());
    assert!(disk.sources()[&dependency].contains("return 2"));
    assert!(
        !workspace.current(&disk),
        "disk write must invalidate before a watch event"
    );
    workspace.invalidate_file(&dependency);
    assert!(!workspace.current(&disk));
    let changed = workspace.snapshot();
    assert!(changed.check(&entry).is_err());
    workspace.open(
        dependency.clone(),
        5,
        "export function value():int{return 3;}".into(),
    );
    let unsaved = workspace.snapshot();
    assert!(unsaved.check(&entry).is_ok());
    assert!(unsaved.sources()[&dependency].contains("return 3"));
    std::fs::remove_file(&dependency.0).unwrap();
    assert!(
        workspace.current(&unsaved),
        "an open buffer remains authoritative over disk removal"
    );
    workspace.close(&dependency);
    let missing = workspace.snapshot();
    assert!(missing.check(&entry).is_err());
    assert!(workspace.current(&missing));
    std::fs::write(&dependency.0, "export function value():int{return 4;}").unwrap();
    assert!(
        !workspace.current(&missing),
        "disk creation must invalidate a cached missing import"
    );
    let recreated = workspace.snapshot();
    assert!(recreated.check(&entry).is_ok());
    let temporary = format!("{}.replacement", dependency.0);
    std::fs::write(&temporary, "export function value():int{return 5;}").unwrap();
    std::fs::rename(temporary, &dependency.0).unwrap();
    assert!(
        !workspace.current(&recreated),
        "atomic same-length replacement must invalidate"
    );
    drop(cleanup);
}
