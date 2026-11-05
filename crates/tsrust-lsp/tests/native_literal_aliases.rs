use serde_json::{Value, json};
use tsrust_lsp::{project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace};
fn uri() -> &'static str { if cfg!(windows) { "file:///C:/tsrust-aliases/main.tsr" } else { "file:///tsrust-aliases/main.tsr" } }
fn ask(index: &ProjectIndex, source: &str, needle: &str, method: &str) -> Value {
    let offset = source.rfind(needle).unwrap(); let prefix = &source[..offset];
    let position = json!({"line":prefix.bytes().filter(|b| *b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});
    index.request(method, &json!({"position":position}), uri()).unwrap().unwrap()
}
#[test]
fn native_literal_alias_hover_and_import_completion_use_checked_literals() {
    let source = tsrust_compiler::format("import {SeekOrigin as Origin} from 'std:fs'; function main(){const origin:Origin='start';console.log(origin);}").unwrap();
    let mut workspace = Workspace::default(); workspace.open(source_id(uri()).unwrap(), 1, source.clone());
    let checked = workspace.snapshot().check(&source_id(uri()).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &source, "Origin =", "textDocument/hover");
    let text = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(text.contains("start") && text.contains("current") && text.contains("end") && text.contains("Ownership"), "{hover}");
    let source = "import { Seek } from 'std:fs'; function main(){}";
    let mut workspace = Workspace::default(); workspace.open(source_id(uri()).unwrap(), 1, source.into());
    let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &source_id(uri()).unwrap().0).unwrap();
    let completion = ask(&ProjectIndex::new(&graph), source, " }", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "SeekOrigin"), "{completion}");
}
