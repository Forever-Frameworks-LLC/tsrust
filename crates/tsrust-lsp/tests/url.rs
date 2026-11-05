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
    for module in ["std:url","std:json"] {for annotation in ["Text","ReadOnly<Text>"] {
        let prelude=format!("import {{StringView as Text}} from '{module}';type Alias=Text;");
        let source=format!("{prelude}function inspect(value:{annotation}):void{{console.log(value.length);}}");
        let hover=request(&source,"value.length","textDocument/hover").to_string();assert!(hover.contains("int"),"{module}/{annotation}: {hover}");
        let source=format!("{prelude}function inspect(value:{annotation}):void{{value.;}}");
        let completion=request(&source,"value.","textDocument/completion");let labels=completion.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(labels,["length","startsWith","endsWith","includes","indexOf","lastIndexOf"].into_iter().collect(),"{module}/{annotation}: {completion}");
        let source=format!("{prelude}function inspect(value:Alias):void{{value.;}}");let aliases=request(&source,"value.","textDocument/completion");assert_eq!(aliases.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<std::collections::BTreeSet<_>>(),labels,"{module}: local alias identity");
    }}
}

#[test]
fn unwrapped_native_results_keep_payload_identity_during_syntax_recovery() {
    for(source,required,excluded)in[
        ("import {parseUrl} from 'std:url';function main(){const value=unwrap(parseUrl('https://example.com/'));value.;}function broken(){const = ;}",&["href","hostname","hash"][..],&["trim","seek"][..]),
        ("import {stat} from 'std:fs';function main(){const value=unwrap(stat('input'));value.;}function broken(){const = ;}",&["size","isFile","isDirectory"][..],&["seek","truncate","href"][..]),
    ]{
        let completion=request(source,"value.","textDocument/completion");let labels=completion.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<std::collections::BTreeSet<_>>();
        for name in required {assert!(labels.contains(name),"missing {name}: {completion}");}for name in excluded {assert!(!labels.contains(name),"unexpected {name}: {completion}");}
    }
    for expression in ["unwrap(missing)","unwrap('not a Result')","unwrap(parseUrl('https://example.com/'),parseUrl('https://other.example/'))"] {
        let source=format!("import {{parseUrl}} from 'std:url';function main(){{const value={expression};value.;}}function broken(){{const = ;}}");let completion=request(&source,"value.","textDocument/completion");assert!(completion.as_array().unwrap().is_empty(),"{expression}: {completion}");
    }
}

#[test]
fn shadowed_unwrap_bindings_keep_their_declared_result_types() {
    for(source,required,excluded)in[
        ("function unwrap(value:int):string{return 'known';}function main(){const value=unwrap(1);value.;}function broken(){const = ;}",&["trim","startsWith"][..],&["href","hostname"][..]),
        ("function inspect(unwrap:(input:int)=>int){const value=unwrap(1);value.;}function broken(){const = ;}",&["toString"][..],&["href","length","includes"][..]),
    ]{
        let completion=request(source,"value.","textDocument/completion");let labels=completion.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<std::collections::BTreeSet<_>>();
        for name in required {assert!(labels.contains(name),"missing {name}: {completion}");}for name in excluded {assert!(!labels.contains(name),"unexpected {name}: {completion}");}
    }
}
