use serde_json::json;
use tsrust_lsp::{
    intelligence::SourceIndex, project_diagnostics::source_id, project_intelligence::ProjectIndex,
    workspace::Workspace,
};

fn project_completion(source: &str, offset: usize) -> serde_json::Value {
    let uri = "file:///sqlite.tsr";
    let id = source_id(uri).unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id.clone(), 1, source.into());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &id.0).unwrap();
    ProjectIndex::new(&graph)
        .request(
            "textDocument/completion",
            &json!({"position":{"line":0,"character":offset}}),
            uri,
        )
        .unwrap()
        .unwrap()
}
#[test]
fn sqlite_alias_completion_and_parameter_help() {
    for (ty, members) in [
        ("Connection", vec!["prepare", "begin", "close"]),
        ("Statement", vec!["execute", "query", "finalize"]),
        ("Transaction", vec!["prepare", "commit", "rollback"]),
        ("Row", vec!["int", "number", "text", "blob", "isNull"]),
    ] {
        let source = format!(
            "import {{{ty} as Local}} from 'std:sqlite';function inspect(x:Local):void{{x.;}}function main(){{}}"
        );
        let offset = source.find("x.").unwrap() + 2;
        let result = project_completion(&source, offset);
        for member in members {
            assert!(
                result
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|item| item["label"] == member),
                "{ty}: {result}"
            );
        }
    }
    let source = "import {Statement} from 'std:sqlite';function inspect(x:Statement){x.execute([]);}function main(){}";
    let offset = source.find("execute(").unwrap() + 8;
    let help = SourceIndex::new(source)
        .request(
            "textDocument/signatureHelp",
            &json!({"position":{"line":0,"character":offset}}),
            "file:///sqlite.tsr",
        )
        .unwrap()
        .to_string();
    assert!(
        help.contains("parameters?") && help.contains("INTEGER"),
        "{help}"
    );
}
#[test]
fn every_sqlite_member_has_behavior_and_ownership_help() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    for family in [
        "SqliteConnection",
        "SqliteStatement",
        "SqliteTransaction",
        "SqliteRow",
    ] {
        for entry in catalog[family].as_array().unwrap() {
            let docs = entry["documentation"].as_str().unwrap();
            assert!(
                docs.contains("Ownership:") && docs.contains("```tsrust"),
                "{family}: {docs}"
            );
        }
    }
}

#[test]
fn inferred_sqlite_handles_complete_from_current_project() {
    for (setup, needle, member) in [
        ("const db=unwrap(open(':memory:'));db.;", "db.", "prepare"),
        (
            "const db=unwrap(open(':memory:'));const statement=unwrap(db.prepare('SELECT 1'));statement.;",
            "statement.",
            "query",
        ),
        (
            "const db=unwrap(open(':memory:'));const transaction=unwrap(db.begin());transaction.;",
            "transaction.",
            "commit",
        ),
        (
            "const db=unwrap(open(':memory:'));const statement=unwrap(db.prepare('SELECT 1 AS id'));const rows=unwrap(statement.query());rows[0].;",
            "rows[0].",
            "int",
        ),
    ] {
        let source = format!("import {{open}} from 'std:sqlite';function main(){{{setup}}}");
        let offset = source.rfind(needle).unwrap() + needle.len();
        let result = project_completion(&source, offset);
        assert!(
            result
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == member),
            "{source}: {result}"
        );
    }
}

#[test]
fn unknown_index_does_not_offer_array_members_but_literals_do() {
    for (source, expected) in [
        ("function main(){missing[0].;}", false),
        ("function main(){[1,2].;}", true),
    ] {
        let offset = source.rfind('.').unwrap() + 1;
        let result = SourceIndex::new(source)
            .request(
                "textDocument/completion",
                &json!({"position":{"line":0,"character":offset}}),
                "file:///sqlite.tsr",
            )
            .unwrap();
        assert_eq!(
            result
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "push"),
            expected,
            "{source}: {result}"
        );
    }
}
