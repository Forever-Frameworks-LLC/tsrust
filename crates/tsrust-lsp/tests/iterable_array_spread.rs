use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn iterable_array_spread_types_and_array_of_help(){
 let source=r#"function main(){const keys=new Set<int>([1]);const copied=[...keys];const characters=[..."é🙂"];const map=new Map<string,int>([["a",2]]);const pairs=[...map];const empty:int[]=[...new Set()];const joined:(int|string)[]=[1,..."ab"];const values=Array.of(...new Set<int>([1]));}"#;
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("copied=[","int[]"),("characters=[","string[]"),("pairs=[","[string,int][]"),("empty:int","int[]"),("joined:(","(int|string)[]")]{let at=source.find(needle).unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///iterable-array-spread.tsr").unwrap();assert!(result.to_string().replace(" ","").contains(expected),"{needle}: {result}");}
 let at=source.find("of(...").unwrap();let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":at+3}}),"file:///iterable-array-spread.tsr").unwrap();assert!(help.to_string().contains("Unicode scalar"),"{help}");assert!(help.to_string().contains("Maps"),"{help}");
}

#[test]
fn array_of_documented_examples_match_spread_fixture(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();let entry=catalog["ArrayConstructor"].as_array().unwrap().iter().find(|x|x["name"]=="of").unwrap();
 let fixture=include_str!("../../tsrust-compiler/tests/fixtures/iterable-array-spread/documented.tsr");let output=include_str!("../../tsrust-compiler/tests/fixtures/iterable-array-spread/documented.stdout");
 let mut expected=String::new();for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();let source=if source.starts_with("type "){source.split_once("};").unwrap().1}else{source};assert!(fixture.contains(source));expected.push_str(example["output"].as_str().unwrap());}assert_eq!(expected,output);tsrust_compiler::check(fixture).unwrap();
}
