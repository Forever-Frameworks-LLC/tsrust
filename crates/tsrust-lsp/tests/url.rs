use serde_json::{json,Value};
use tsrust_lsp::intelligence::SourceIndex;
fn request(source:&str,needle:&str,method:&str)->Value {
    let offset=source.find(needle).unwrap()+needle.len()-usize::from(method=="textDocument/hover");
    SourceIndex::new(source).request(method,&json!({"position":{"line":0,"character":offset}}),"file:///url.tsr").unwrap()
}
#[test]
fn computed_property_hover_comes_from_read_and_write_metadata() {
    let source="import {Url as Address} from 'std:url';function inspect(value:Address):void{console.log(value.pathname);console.log(value.port);console.log(value.origin);}";
    let pathname=request(source,"value.pathname","textDocument/hover").to_string();
    for expected in ["StringView","assign string","UrlError","Ownership"] {assert!(pathname.contains(expected),"{pathname}");}
    assert!(!pathname.contains("ResultUnwrap")&&!pathname.contains("__set")&&!pathname.contains("__tsrust_private_"));
    let origin=request(source,"value.origin","textDocument/hover").to_string();assert!(origin.contains("read-only"),"{origin}");
    let port=request(source,"value.port","textDocument/hover").to_string();assert!(port.contains("int")&&port.contains("undefined"),"{port}");
}
#[test]
fn aliases_complete_properties_and_query_methods_without_duplicates() {
    let properties=request("import {Url as Address} from 'std:url';function inspect(value:Address):void{value.;}","value.","textDocument/completion");
    let labels=properties.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<Vec<_>>();
    assert_eq!(labels.iter().filter(|label|**label=="pathname").count(),1);
    for name in ["href","hostname","port","search","hash","origin"] {assert!(labels.contains(&name),"{properties}");}
    let methods=request("import {SearchParams as Query} from 'std:url';function inspect(value:Query):void{value.;}","value.","textDocument/completion").to_string();
    for name in ["getAll","append","entries","sort"] {assert!(methods.contains(name),"{methods}");}
}
#[test]
fn query_and_parse_signature_help_preserve_public_contracts() {
    let source="import {SearchParams,parseUrl} from 'std:url';function inspect(params:SearchParams):void{params.get('x');parseUrl('https://example.com/');}";
    let get=request(source,"params.get(","textDocument/signatureHelp").to_string();assert!(get.contains("StringView")&&get.contains("undefined"),"{get}");
    let parse=request(source,"parseUrl(","textDocument/signatureHelp").to_string();assert!(parse.contains("base")&&parse.contains("UrlError"),"{parse}");
}

#[test]
fn shared_string_view_export_preserves_native_identity_in_editor_aliases() {
    let source="import {StringView as Text} from 'std:url';function inspect(value:Text):void{console.log(value.length);}";
    let hover=request(source,"value.length","textDocument/hover").to_string();
    assert!(hover.contains("int"),"{hover}");
    let completion=request("import {StringView as Text} from 'std:url';function inspect(value:Text):void{value.;}","value.","textDocument/completion").to_string();
    assert!(completion.contains("startsWith")&&completion.contains("includes"),"{completion}");
}
