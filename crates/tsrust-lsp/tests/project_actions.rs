use serde_json::json;
use tsrust_lsp::{
    Snapshot,
    project_actions::ProjectActions,
    project_diagnostics::{convert, source_id},
    project_session::ProjectSession,
    workspace::Workspace,
};

#[test]
fn clone_project_edits_are_complete_versioned_and_expire_with_dependencies() {
    let root = "file:///tsrust-actions/main.tsr";
    let dep = "file:///tsrust-actions/dep.tsr";
    let entry = "import { User, take } from './dep.tsr';function main(){const user:User={name:\"😀\"};const other=user;const saved=take(user);console.log(saved.name);console.log(user.name);}";
    let library =
        "export type User={name:string;};export function take(user:User):User{return user;}";
    let mut session = ProjectSession::default();
    for (uri, text, version) in [(root, entry, 7), (dep, library, 3)] {
        session.open(&Snapshot {
            uri: uri.into(),
            version,
            generation: 1,
            text: text.into(),
        });
    }
    let input = session.workspace.snapshot();
    let error = input.check(&source_id(root).unwrap()).err().unwrap();
    assert_eq!(error.code, "transferred");
    let diagnostic = convert(&input, Some(&error)).unwrap();
    let params = json!({"range":diagnostic[&source_id(root).unwrap()][0].range,"context":{"diagnostics":[]}});
    let actions = ProjectActions::build(&input, &source_id(root).unwrap(), &error).unwrap();
    session.cache_actions(root, &input, Some(actions));
    let response = session.code_actions(root, &params).unwrap();
    assert_eq!(response.as_array().unwrap().len(), 1, "{response}");
    assert_eq!(
        response[0]["title"],
        "Clone before 2 transfers (copies data; may allocate)"
    );
    let change = &response[0]["edit"]["documentChanges"][0];
    assert_eq!(change["textDocument"]["uri"], root);
    assert_eq!(change["textDocument"]["version"], 7);
    assert_eq!(change["edits"].as_array().unwrap().len(), 2);
    let fixed = entry
        .replace("const other=user;", "const other=clone(user);")
        .replace("take(user)", "take(clone(user))");
    session.workspace.invalidate_file(&source_id(dep).unwrap());
    assert_eq!(session.code_actions(root, &params).unwrap(), json!([]));
    session.change(&Snapshot {
        uri: root.into(),
        version: 8,
        generation: 2,
        text: fixed,
    });
    assert_eq!(session.code_actions(root, &params).unwrap(), json!([]));
    assert!(
        session
            .workspace
            .snapshot()
            .check(&source_id(root).unwrap())
            .is_ok()
    );
}

#[test]
fn dependency_syntax_fix_targets_the_dependency_and_requires_whole_project_success() {
    let root = "file:///tsrust-actions/main.tsr";
    let dep = "file:///tsrust-actions/dep.tsr";
    let entry = "import { value } from './dep.tsr';function main(){console.log(value());}";
    let library = "export function value():string {const text=\"😀\"\nreturn text;}";
    for bad in [false, true] {
        let mut workspace = Workspace::default();
        let entry = if bad {
            entry.replace(
                "console.log(value());",
                "const bad:int=true;console.log(value());",
            )
        } else {
            entry.into()
        };
        workspace.open(source_id(root).unwrap(), 5, entry);
        workspace.open(source_id(dep).unwrap(), 9, library.into());
        let input = workspace.snapshot();
        let error = input.check(&source_id(root).unwrap()).err().unwrap();
        assert_eq!(error.location.source, source_id(dep).unwrap());
        let buckets = convert(&input, Some(&error)).unwrap();
        let params = json!({"range":buckets[&source_id(dep).unwrap()][0].range});
        let actions = ProjectActions::build(&input, &source_id(root).unwrap(), &error).unwrap();
        let response = actions.request(&source_id(dep).unwrap(), &params).unwrap();
        assert_eq!(response.len(), usize::from(!bad), "{response:?}");
        if !bad {
            let change = &response[0]["edit"]["documentChanges"][0];
            assert_eq!(change["textDocument"]["uri"], dep);
            assert_eq!(change["textDocument"]["version"], 9);
            assert_eq!(change["edits"][0]["newText"], ";");
        }
        assert!(
            actions
                .request(&source_id(root).unwrap(), &params)
                .unwrap()
                .is_empty()
        );
        let mut only = params.clone();
        only["context"] = json!({"only":["refactor"]});
        assert!(
            actions
                .request(&source_id(dep).unwrap(), &only)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn closed_dependency_fix_is_offered_only_while_disk_matches_the_checked_text() {
    let directory = std::env::temp_dir().join(format!("tsrust-closed-fix-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let dep_path = directory.join("dep.tsr");
    let library = "export function value():string {const text=\"😀\"\nreturn text;}";
    std::fs::write(&dep_path, library).unwrap();
    let root = format!("file://{}/main.tsr", directory.display());
    let dep = format!("file://{}", dep_path.display());
    let mut workspace = Workspace::default();
    workspace.open(
        source_id(&root).unwrap(),
        3,
        "import { value } from './dep.tsr';function main(){console.log(value());}".into(),
    );
    let input = workspace.snapshot();
    let error = input.check(&source_id(&root).unwrap()).err().unwrap();
    assert_eq!(error.location.source, source_id(&dep).unwrap());
    let buckets = convert(&input, Some(&error)).unwrap();
    let params = json!({"range":buckets[&source_id(&dep).unwrap()][0].range});
    let actions = ProjectActions::build(&input, &source_id(&root).unwrap(), &error).unwrap();
    let response = actions.request(&source_id(&dep).unwrap(), &params).unwrap();
    assert_eq!(response.len(), 1, "{response:?}");
    let change = &response[0]["edit"]["documentChanges"][0];
    assert_eq!(change["textDocument"]["uri"], dep);
    // A closed file has no editor version; the edit applies to the verified disk text.
    assert!(change["textDocument"]["version"].is_null(), "{change}");
    assert_eq!(change["edits"][0]["newText"], ";");
    // Same length and a restored modification time: only a content comparison detects this.
    let modified = std::fs::metadata(&dep_path).unwrap().modified().unwrap();
    std::fs::write(&dep_path, library.replace("text", "tezt")).unwrap();
    std::fs::File::options().write(true).open(&dep_path).unwrap().set_modified(modified).unwrap();
    assert!(actions.request(&source_id(&dep).unwrap(), &params).unwrap().is_empty());
    std::fs::write(&dep_path, library).unwrap();
    assert_eq!(actions.request(&source_id(&dep).unwrap(), &params).unwrap().len(), 1);
    std::fs::remove_file(&dep_path).unwrap();
    assert!(actions.request(&source_id(&dep).unwrap(), &params).unwrap().is_empty());
    std::fs::remove_dir_all(&directory).unwrap();
}

#[test]
fn session_offers_closed_dependency_fixes_until_the_file_changes() {
    let directory = std::env::temp_dir().join(format!("tsrust-closed-session-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    let dep_path = directory.join("dep.tsr");
    let library = "export function value():string {const text=\"a\"\nreturn text;}";
    std::fs::write(&dep_path, library).unwrap();
    let root = format!("file://{}/main.tsr", directory.display());
    let dep = format!("file://{}", dep_path.display());
    let mut session = ProjectSession::default();
    let snapshot = Snapshot {
        uri: root.clone(),
        version: 4,
        generation: 1,
        text: "import { value } from './dep.tsr';function main(){console.log(value());}".into(),
    };
    session.open(&snapshot);
    let input = session.prepare(&snapshot).unwrap();
    let error = input.check(&source_id(&root).unwrap()).err().unwrap();
    let buckets = convert(&input, Some(&error)).unwrap();
    let params = json!({"range":buckets[&source_id(&dep).unwrap()][0].range});
    let actions = ProjectActions::build(&input, &source_id(&root).unwrap(), &error).unwrap();
    session.cache_actions(&root, &input, Some(actions));
    let response = session.code_actions(&dep, &params).unwrap();
    let change = &response[0]["edit"]["documentChanges"][0];
    assert_eq!(change["textDocument"]["uri"], dep);
    assert!(change["textDocument"]["version"].is_null());
    // Opening the dependency makes the editor buffer the authority; the cached edit expires.
    session.open(&Snapshot { uri: dep.clone(), version: 1, generation: 2, text: library.into() });
    assert_eq!(session.code_actions(&dep, &params).unwrap(), json!([]));
    std::fs::remove_dir_all(&directory).unwrap();
}
