use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn bundled_ui_props_have_completion_and_behavioral_hover_before_project_analysis() {
    for (component,alias,partial,property,value,documentation) in [
        ("CodeEditor","Editor","lineN","lineNumbers","false","Defaults to true"),
        ("PlotTooltip","Tip","crossL","crossLine","\"both\"","Defaults to vertical"),
        ("PlotLine","Line","val","values","[1,2]","NaN and infinities throw"),
        ("Diff","Compare","contextL","contextLines","-1","Cannot recover lines omitted from the patch"),
    ] {
        let prefix=format!("import {{{component} as {alias}}} from 'std:ui';function main(){{const node=<{alias} ");
        let source=format!("{prefix}{partial}");
        let index=SourceIndex::new(&source);
        let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":source.len()}}),"file:///props.tsrx").unwrap();
        assert!(items.as_array().unwrap().iter().any(|item|item["label"]==property),"{component}: {items}");
        let source=format!("{prefix}{property}={{{value}}} />;}}");
        let index=SourceIndex::new(&source);let at=source.find(property).unwrap();
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at+3}}),"file:///props.tsrx").unwrap();
        assert!(hover.to_string().contains(documentation),"{component}: {hover}");
        let definition=index.request("textDocument/definition",&json!({"position":{"line":0,"character":at+3}}),"file:///props.tsrx").unwrap();
        assert_eq!(definition["uri"],"tsrust-stdlib:///std:ui","{component}");
    }
}
#[test]
fn bundled_ui_functions_keep_help_without_a_project_and_respect_shadowing() {
    let source="import {routeParam as param} from 'std:ui';function main(){console.log(param('/users/ada','/users/:id','id'));}";
    let index=SourceIndex::new(source);
    let at=source.find("param('/").unwrap();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at+2}}),"file:///routes.tsrx").unwrap();
    assert!(hover.to_string().contains("empty string") && hover.to_string().contains("Ownership:"),"{hover}");
    let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":at+"param(".len()}}),"file:///routes.tsrx").unwrap();
    let parameters=help["signatures"][0]["parameters"].as_array().unwrap();
    assert_eq!(parameters.len(),3,"{help}");
    assert!(parameters[1].to_string().contains("Named segments"),"{help}");
    let definition=index.request("textDocument/definition",&json!({"position":{"line":0,"character":at+2}}),"file:///routes.tsrx").unwrap();
    assert_eq!(definition["uri"],"tsrust-stdlib:///std:ui");
    let bundled=tsrust_compiler::project::bundled_source(&tsrust_compiler::project::SourceId("std:ui".into())).unwrap();
    let expected=bundled[..bundled.find("export function routeParam").unwrap()].bytes().filter(|byte|*byte==b'\n').count();
    assert_eq!(definition["range"]["start"]["line"].as_u64(),Some(expected as u64));
    let source="import {routeParam as param} from 'std:ui';function local(param:int):int{return param;}function main(){}";
    let index=SourceIndex::new(source);let at=source.find("return param").unwrap()+"return ".len();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at+2}}),"file:///routes.tsrx").unwrap();
    assert!(hover.to_string().contains("int") && !hover.to_string().contains("empty string"),"{hover}");
}
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

#[test]
fn recovered_member_edits_preserve_quoted_optional_and_identifier_access() {
    for (access, label) in [(".", "aria-label"), ("?.", "aria-label"), (".ar", "aria-label"), ("?.ar", "aria-label"), (".ti", "title"), ("?.ti", "title")] {
        let optional=access.starts_with("?.");
        let ty=if optional{"Props | undefined"}else{"Props"};
        let source=format!("interface Props {{'aria-label':string; title:string;}} function probe(props:{ty}){{/* 😀 */props{access};}}");
        let start=source.rfind("props").unwrap()+5;
        let cursor=start+access.len();
        let items=SourceIndex::new(&source).request("textDocument/completion",&json!({"position":{"line":0,"character":source[..cursor].encode_utf16().count()}}),"file:///props.tsr").unwrap();
        let item=items.as_array().unwrap().iter().find(|item|item["label"]==label).unwrap();
        let start=if label=="title"{cursor-2}else{start};
        let replacement=if label=="title"{"title"}else if optional{"?.[\"aria-label\"]"}else{"[\"aria-label\"]"};
        assert_eq!(item["textEdit"]["newText"],replacement,"{access}: {item}");
        assert_eq!(item["textEdit"]["range"],json!({"start":{"line":0,"character":source[..start].encode_utf16().count()},"end":{"line":0,"character":source[..cursor].encode_utf16().count()}}));
        let fixed=format!("{}{replacement}{}",&source[..start],&source[cursor..]);
        assert!(tsrust_compiler::check(&fixed).is_ok(),"{fixed}");
    }
}
