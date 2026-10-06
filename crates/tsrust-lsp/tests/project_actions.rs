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
