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

#[test]
fn quoted_properties_use_exact_jsx_names_and_valid_member_edits() {
    let source="interface Props {'aria-label':string; 'data-slot'?:string;} function Label(props:Props):string{return props['aria-label'];} function main(){const label=<Label aria-label=\"Name\" data-slot=\"label\"/>;}";
    tsrust_compiler::check(source).unwrap();
    let index=SourceIndex::new(source);
    for needle in ["aria-label=", "data-slot="] {
        let at=source.find(needle).unwrap()+needle.find('-').unwrap()+2;
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///props.tsrx").unwrap();
        assert!(hover.to_string().contains(needle.trim_end_matches('=')),"{hover}");
        let definition=index.request("textDocument/definition",&json!({"position":{"line":0,"character":at}}),"file:///props.tsrx").unwrap();
        assert!(!definition.is_null(),"{definition}");
    }
    let source="interface Props {'aria-label':string;} function probe(props:Props){props.}";
    let index=SourceIndex::new(source); let at=source.find("props.}").unwrap()+6;
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at}}),"file:///props.tsr").unwrap();
    let item=items.as_array().unwrap().iter().find(|item|item["label"]=="aria-label").unwrap();
    assert_eq!(item["textEdit"]["newText"],"[\"aria-label\"]");
    assert_eq!(item["textEdit"]["range"]["start"]["character"],at-1);
}

#[test]
fn quoted_field_help_survives_incomplete_source() {
    let source="interface Props { 'aria-label':string; } function probe(props:Props){console.log(props['aria-label'].length);props.}";
    let index=SourceIndex::new(source);let at=source.rfind("props.}").unwrap()+6;
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at}}),"file:///props.tsr").unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="aria-label"),"{items}");
    let at=source.find("['aria-label']").unwrap()+5;
    let help=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///props.tsr").unwrap();
    assert!(help.to_string().contains("aria-label: string"),"{help}");
    let at=source.find(".length").unwrap()+3;
    let help=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///props.tsr").unwrap();
    assert!(help.to_string().contains("length"),"{help}");
}
