use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn array_from_collection_results_and_help_are_consistent(){
 let source="function main(){const values=new Set<int>([1]);const copy=Array.from(values);const map=new Map<string,int>([['a',2]]);const pairs=Array.from(map);const labels=Array.from(map,([key,value])=>{return key+String(value);});const joined=Array.from(new Set<int|string>([1,'x']));}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("copy=Array","int[]"),("pairs=Array","[string, int][]"),("labels=Array","string[]"),("joined=Array","(int|string)[]")]{let at=source.find(needle).unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///array-from-collections.tsr").unwrap();assert!(result.to_string().replace(" ", "").contains(&expected.replace(" ", "")),"{result}");}
 let at=source.find("from(map);").unwrap();
 for(method,offset)in[("textDocument/hover",at),("textDocument/signatureHelp",at+5)]{let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///array-from-collections.tsr").unwrap();assert!(result.to_string().contains("Map<Key, Value>"),"{result}");assert!(result.to_string().contains("readonly"),"{result}");assert!(result.to_string().replace(" ", "").contains("[string,int][]"),"{result}");}
}
#[test]
fn array_from_documented_examples_are_current(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();let entry=catalog["ArrayConstructor"].as_array().unwrap().iter().find(|x|x["name"]=="from").unwrap();
 let fixture=include_str!("../../tsrust-compiler/tests/fixtures/array-from-collections/documented.tsr");let output=include_str!("../../tsrust-compiler/tests/fixtures/array-from-collections/documented.stdout");
 let mut expected=String::new();for example in entry["examples"].as_array().unwrap(){assert!(fixture.contains(example["source"].as_str().unwrap()));expected.push_str(example["output"].as_str().unwrap());}assert_eq!(expected,output);tsrust_compiler::check(fixture).unwrap();
}
