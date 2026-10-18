use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn jsx_attributes_use_component_props_for_editor_help() {
    let prefix="interface Props {title:string; disabled:boolean;} function Button(props:Props):string{return props.title;} function main(){const button=<Button ";
    let source=format!("{prefix}ti");
    let index=SourceIndex::new(&source);
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":source.len()}}),"file:///counter.tsrx").unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="title"),"{items}");
    let source=format!("{prefix}title=\"Hi\" disabled />;}}");
    tsrust_compiler::check(&source).unwrap();
    let index=SourceIndex::new(&source);
    let at=source.rfind("title").unwrap();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at+2}}),"file:///counter.tsrx").unwrap();
    assert!(hover.to_string().contains("title: string"),"{hover}");
    let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":at}}),"file:///counter.tsrx").unwrap();
    assert!(help.to_string().contains("Props"),"{help}");
}
