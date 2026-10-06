use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn imported_enum_alias_keeps_member_definition_and_completion() {
    use tsrust_lsp::{project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace};
    let uri = "file:///tsrust-enums/main.tsr";
    let dependency = "file:///tsrust-enums/dep.tsr";
    let source = "import {Phase as State} from './dep.tsr';function main(){const phase:State=State.Ready;}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,source.into());
    workspace.open(source_id(dependency).unwrap(),1,"export enum Phase{Ready,Busy}".into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let position=json!({"position":{"line":0,"character":source.rfind("Ready").unwrap()}});
    let definition=index.request("textDocument/definition",&position,uri).unwrap().unwrap();
    assert_eq!(definition["uri"],dependency);
    let completion=index.request("textDocument/completion",&position,uri).unwrap().unwrap();
    assert!(completion.to_string().contains("Busy"),"{completion}");
}

#[test]
fn enum_members_have_hover_definition_and_completion() {
    let source = "enum Phase { Ready, Busy = 4 } function main(){const phase=Phase.Ready;}";
    tsrust_compiler::check(source).unwrap();
    let index = SourceIndex::new(source);
    let usage = source.rfind("Ready").unwrap();
    let hover = index.request("textDocument/hover", &json!({"position":{"line":0,"character":usage}}), "file:///enums.tsr").unwrap();
    assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("Phase.Ready"), "{hover}");
    let definition = index.request("textDocument/definition", &json!({"position":{"line":0,"character":usage}}), "file:///enums.tsr").unwrap();
    assert_eq!(definition["range"]["start"]["character"], source.find("Ready").unwrap());
    let completion = index.request("textDocument/completion", &json!({"position":{"line":0,"character":usage}}), "file:///enums.tsr").unwrap();
    assert!(completion.to_string().contains("Busy"), "{completion}");
    let formatted=tsrust_compiler::format(source).unwrap();
    tsrust_compiler::check(&formatted).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
}

#[test]
fn enum_keyword_has_highlighting() {
    let grammar: serde_json::Value = serde_json::from_str(include_str!("../../../editors/vscode/syntaxes/tsrust.tmLanguage.json")).unwrap();
    let rule = grammar["patterns"].as_array().unwrap().iter().find(|row| row["name"] == "storage.type.tsrust").unwrap();
    assert!(rule["match"].as_str().unwrap().contains("enum"));
}
