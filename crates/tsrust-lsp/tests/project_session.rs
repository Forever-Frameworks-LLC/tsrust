use lsp_types::{Diagnostic, Range};
use tsrust_lsp::{
    Snapshot,
    project_diagnostics::source_id,
    project_session::{Buckets, ProjectSession},
};
fn snapshot(name: &str, text: &str) -> Snapshot {
    Snapshot {
        uri: format!("file:///tsrust-session/{name}.tsr"),
        version: 1,
        generation: 1,
        text: text.into(),
    }
}
#[test]
fn shared_dependency_diagnostics_are_merged_and_released_per_entry() {
    let mut session = ProjectSession::default();
    let dep = snapshot("dep", "export function value(){}");
    let a = snapshot(
        "a",
        "import { value } from './dep.tsr';function main(){value();}",
    );
    let b = snapshot("b", &a.text);
    for snapshot in [&dep, &a, &b] {
        session.open(snapshot);
    }
    let dependency = source_id(&dep.uri).unwrap();
    let make = |message: &str| {
        let mut buckets = Buckets::new();
        buckets.insert(
            dependency.clone(),
            vec![Diagnostic {
                range: Range::default(),
                message: message.into(),
                ..Diagnostic::default()
            }],
        );
        buckets
    };
    let sa = session.prepare(&a).unwrap();
    sa.check(&source_id(&a.uri).unwrap()).unwrap();
    session.accept(&a.uri, &sa, make("A")).unwrap();
    let sb = session.prepare(&b).unwrap();
    sb.check(&source_id(&b.uri).unwrap()).unwrap();
    let merged = session.accept(&b.uri, &sb, make("B")).unwrap();
    assert_eq!(merged[0].diagnostics.len(), 2);
    let cleared = session.accept(&a.uri, &sa, Buckets::new()).unwrap();
    assert_eq!(cleared[0].diagnostics.len(), 1);
    assert_eq!(cleared[0].diagnostics[0].message, "B");
    let closed = session.close(&b.uri).1;
    assert!(closed[0].diagnostics.is_empty());
}
#[test]
fn stale_inputs_cannot_replace_publications_and_watches_requeue_dependents() {
    let mut session = ProjectSession::default();
    let dep = snapshot("dep", "export function value(){}");
    let entry = snapshot(
        "main",
        "import { value } from './dep.tsr';function main(){value();}",
    );
    session.open(&dep);
    session.open(&entry);
    let input = session.prepare(&entry).unwrap();
    input.check(&source_id(&entry.uri).unwrap()).unwrap();
    session.accept(&entry.uri, &input, Buckets::new()).unwrap();
    assert_eq!(session.changed_file(&dep.uri), vec![entry.uri.clone()]);
    assert!(
        session
            .accept(&entry.uri, &input, Buckets::new())
            .unwrap()
            .is_empty()
    );
    session.forget_project(&entry.uri);
    assert!(session.changed_file(&dep.uri).is_empty());
}

#[cfg(unix)]
#[test]
fn disk_dependency_rename_recreate_and_symlink_retarget_invalidate_cached_inputs() {
    use std::{fs, os::unix::fs::symlink};
    use tsrust_lsp::project_diagnostics::{convert, file_uri};
    use tsrust_lsp::project_intelligence::ProjectIndex;
    let directory = std::env::temp_dir().join(format!(
        "tsrust-module-watch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    let directory = fs::canonicalize(directory).unwrap();
    let old = directory.join("dep.tsr");
    let renamed = directory.join("renamed.tsr");
    let alias = directory.join("alias.tsr");
    let replacement = directory.join("replacement.tsr");
    fs::write(&old, "export function value():int{return 1;}").unwrap();
    fs::write(&replacement, "export function value():int{return 2;}").unwrap();
    symlink(&old, &alias).unwrap();
    let uri = |path: &std::path::Path| {
        file_uri(&tsrust_compiler::project::SourceId(
            path.to_str().unwrap().into(),
        ))
        .unwrap()
        .to_string()
    };
    let root = Snapshot {
        uri: uri(&directory.join("main.tsr")),
        version: 1,
        generation: 1,
        text: "import { value } from './alias.tsr';function main(){console.log(value());}".into(),
    };
    let mut session = ProjectSession::default();
    session.open(&root);
    let input = session.prepare(&root).unwrap();
    let checked = input.check(&source_id(&root.uri).unwrap()).unwrap();
    session.cache_index(&root.uri, &input, Some(ProjectIndex::new(&checked.graph)));
    session
        .accept(&root.uri, &input, convert(&input, None).unwrap())
        .unwrap();
    let params =
        serde_json::json!({"position":{"line":0,"character":root.text.rfind("value()").unwrap()}});
    assert!(
        session
            .request("textDocument/definition", &params, &root.uri)
            .is_some()
    );
    // After removal, canonicalization cannot recover the old symlink target.
    fs::remove_file(&alias).unwrap();
    assert_eq!(session.changed_file(&uri(&alias)), vec![root.uri.clone()]);
    assert!(!session.workspace.current(&input));
    assert!(
        session
            .request("textDocument/definition", &params, &root.uri)
            .is_none()
    );
    let missing = session.prepare(&root).unwrap();
    assert_eq!(
        missing
            .check(&source_id(&root.uri).unwrap())
            .err()
            .unwrap()
            .code,
        "module-load"
    );
    session
        .accept(&root.uri, &missing, convert(&missing, None).unwrap())
        .unwrap();
    symlink(&replacement, &alias).unwrap();
    assert_eq!(session.changed_file(&uri(&alias)), vec![root.uri.clone()]);
    let recreated = session.prepare(&root).unwrap();
    let checked = recreated.check(&source_id(&root.uri).unwrap()).unwrap();
    session.cache_index(
        &root.uri,
        &recreated,
        Some(ProjectIndex::new(&checked.graph)),
    );
    session
        .accept(&root.uri, &recreated, convert(&recreated, None).unwrap())
        .unwrap();
    let definition = session
        .request("textDocument/definition", &params, &root.uri)
        .unwrap()
        .unwrap();
    assert_eq!(definition["uri"], uri(&replacement));
    // Normal disk rename: update the importing buffer, then remove the old dependency.
    let updated = Snapshot {
        uri: root.uri.clone(),
        version: 2,
        generation: 2,
        text: root.text.replace("./alias.tsr", "./dep.tsr"),
    };
    session.change(&updated);
    let before = session.prepare(&updated).unwrap();
    before.check(&source_id(&root.uri).unwrap()).unwrap();
    session
        .accept(&root.uri, &before, convert(&before, None).unwrap())
        .unwrap();
    fs::rename(&old, &renamed).unwrap();
    assert_eq!(session.changed_file(&uri(&old)), vec![root.uri.clone()]);
    assert!(!session.workspace.current(&before));
    assert!(
        session
            .prepare(&updated)
            .unwrap()
            .check(&source_id(&root.uri).unwrap())
            .is_err()
    );
    let updated = Snapshot {
        uri: root.uri.clone(),
        version: 3,
        generation: 3,
        text: updated.text.replace("./dep.tsr", "./renamed.tsr"),
    };
    session.change(&updated);
    assert!(
        session
            .prepare(&updated)
            .unwrap()
            .check(&source_id(&root.uri).unwrap())
            .is_ok()
    );
}

#[test]
fn open_aliases_keep_independent_versions_and_survive_other_alias_close() {
    let original = snapshot("dep", "export function value():int{return 1;}");
    let alias = Snapshot {
        uri: "file://localhost/tsrust-session/dep.tsr".into(),
        version: 10,
        generation: 1,
        text: "export function value():int{return 2;}".into(),
    };
    let root = snapshot(
        "main",
        "import { value } from './dep.tsr';function main(){console.log(value());}",
    );
    let mut session = ProjectSession::default();
    for buffer in [&original, &alias, &root] {
        session.open(buffer);
    }
    let input = session.prepare(&root).unwrap();
    input.check(&source_id(&root.uri).unwrap()).unwrap();
    session.accept(&root.uri, &input, Buckets::new()).unwrap();
    let dep = source_id(&original.uri).unwrap();
    assert_eq!(input.version(&dep), Some(10));
    let changed = Snapshot {
        version: 2,
        text: "export function value():int{return 3;}".into(),
        ..original.clone()
    };
    assert_eq!(session.change(&changed), vec![root.uri.clone()]);
    let latest = session.prepare(&root).unwrap();
    latest.check(&source_id(&root.uri).unwrap()).unwrap();
    assert_eq!(latest.version(&dep), Some(2));
    assert!(latest.sources()[&dep].contains("return 3"));
    assert!(
        session.change(&changed).is_empty(),
        "stale versions are checked per URI"
    );
    assert_eq!(session.close(&original.uri).0, vec![root.uri.clone()]);
    assert!(!session.workspace.current(&latest));
    let remaining = session.prepare(&root).unwrap();
    remaining.check(&source_id(&root.uri).unwrap()).unwrap();
    assert_eq!(remaining.version(&dep), Some(10));
    assert!(remaining.sources()[&dep].contains("return 2"));
    assert_eq!(session.close(&alias.uri).0, vec![root.uri.clone()]);
    assert!(
        session
            .prepare(&root)
            .unwrap()
            .check(&source_id(&root.uri).unwrap())
            .is_err()
    );
}
