use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn iterable_call_spread_preserves_results_help_and_recovery_types(){
 let source=r#"function first<T>(value:T):T{return value;}function main(){const value=first(...new Set<int>([8]));const text=String.fromCodePoint(...new Set<int>([65]));const result=String.raw({raw:["<",">"]},...new Set<string>(["x"]));try{first(...new Set<int>());}catch(error){const code=error.code;}}"#;
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("value=first","int"),("text=String","string"),("result=String","string"),("code=error","string")]{let at=source.find(needle).unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///iterable-call-spread.tsr").unwrap();assert!(result.to_string().contains(expected),"{needle}: {result}");}
 let at=source.find("fromCodePoint(...").unwrap();let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":at+14}}),"file:///iterable-call-spread.tsr").unwrap();assert!(help.to_string().contains("Sets of int"),"{help}");
}
#[test]
fn iterable_call_spread_catalog_examples_are_current(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();let fixture=include_str!("../../tsrust-compiler/tests/fixtures/iterable-call-spread/documented.tsr");let output=include_str!("../../tsrust-compiler/tests/fixtures/iterable-call-spread/documented.stdout");let mut expected=String::new();
 for(family,name)in[("string","concat"),("StringConstructor","fromCodePoint"),("StringConstructor","fromCharCode"),("StringConstructor","raw"),("Math","min"),("Math","max"),("Math","hypot")]{let entry=catalog[family].as_array().unwrap().iter().find(|x|x["name"]==name).unwrap();let example=entry["examples"].as_array().unwrap().last().unwrap();assert!(fixture.contains(example["source"].as_str().unwrap()));expected.push_str(example["output"].as_str().unwrap());}assert_eq!(expected,output);tsrust_compiler::check(fixture).unwrap();
}
