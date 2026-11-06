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


#[test]
fn optional_finite_string_guard_repairs_receiver_help_in_changed_buffers(){
    let valid="type Phase='request-start'|'task-end';type Alias=Phase;function inspect(value?:ReadOnly<Alias>):int{if(value===undefined){return 0;}return value.length;}function main(){inspect('request-start');}";
    let source=tsrust_compiler::format(valid).unwrap();let id=source_id(uri()).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.clone());
    let snapshot=workspace.snapshot();let checked=snapshot.check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    let completion=ask(&index,&source,"length","textDocument/completion");
    let items=completion.as_array().expect("member completion list");assert!(!items.is_empty());
    for name in["length","startsWith","includes"]{assert!(items.iter().any(|item|item["label"]==name),"{name}: {completion}");}
    let hover=ask(&index,&source,"length","textDocument/hover");assert!(hover.to_string().contains("length"),"{hover}");
    let invalid=tsrust_compiler::format(&valid.replace("if(value===undefined){return 0;}","")).unwrap();
    assert!(workspace.change(&id,2,invalid));
    let errors=workspace.snapshot().check_with_diagnostics(&id).err().expect("unguarded optional member must fail");
    assert!(errors.iter().any(|error|error.code=="type"),"{errors:?}");
    assert!(workspace.change(&id,3,source.clone()));let snapshot=workspace.snapshot();let repaired=snapshot.check(&id).unwrap();
    let completion=ask(&ProjectIndex::checked(&repaired),&source,"length","textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="length"),"{completion}");
}
