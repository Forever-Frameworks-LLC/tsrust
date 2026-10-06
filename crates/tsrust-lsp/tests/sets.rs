use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn typed_set_members_have_hover_and_completion() {
    let source = "function main(){const values:Set<int> =new Set<int>();values.add(2);const count=values.size;}";
    tsrust_compiler::check(source).unwrap();
    let index=SourceIndex::new(source);
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find("add(2)").unwrap()}}),"file:///sets.tsr").unwrap();
    assert!(hover.to_string().contains("add(value: int): void"),"{hover}");
    let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":source.find("add(2)").unwrap()}}),"file:///sets.tsr").unwrap();
    for member in ["add","has","delete","clear","size","values","keys"] {assert!(completion.to_string().contains(member),"{completion}");}
}

#[test]
fn set_constructor_tokens_have_highlighting() {
    let grammar:serde_json::Value=serde_json::from_str(include_str!("../../../editors/vscode/syntaxes/tsrust.tmLanguage.json")).unwrap();
    let text=grammar.to_string();
    assert!(text.contains("new"));
    assert!(text.contains("Owned|Set"));
}

#[test]
fn inferred_set_and_constructor_help_preserve_key_type() {
    let source="function main(){const values=new Set<int>([1]);const found=values.has(1);}";
    let index=SourceIndex::new(source);
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.find("has(1)").unwrap()}}),"file:///sets.tsr").unwrap();
    assert!(hover.to_string().contains("has(value: int): boolean"),"{hover}");
    let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":source.find("has(1)").unwrap()+4}}),"file:///sets.tsr").unwrap();
    assert!(help.to_string().contains("has(value: int): boolean"),"{help}");
    let constructor=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":source.find("([1]").unwrap()+1}}),"file:///sets.tsr").unwrap();
    assert!(constructor.to_string().contains("new Set<int>"),"{constructor}");
    let formatted=tsrust_compiler::format(source).unwrap();
    tsrust_compiler::check(&formatted).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
}

#[test]
fn set_catalog_examples_match_the_native_example_fixture() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let fixture=include_str!("../../tsrust-compiler/tests/fixtures/sets/documented.tsr");
    let mut expected=String::new();
    let mut entries=catalog["Set"].as_array().unwrap().clone();
    entries.extend(catalog["globals"].as_array().unwrap().iter().filter(|entry|entry["name"]=="Set").cloned());
    for entry in entries {for example in entry["examples"].as_array().unwrap(){
        let source=example["source"].as_str().unwrap();assert!(fixture.contains(source));
        tsrust_compiler::check(&format!("function main(){{{source}}}")).unwrap();
        expected.push_str(example["output"].as_str().unwrap());
    }}
    assert_eq!(expected,include_str!("../../tsrust-compiler/tests/fixtures/sets/documented.stdout"));
}
