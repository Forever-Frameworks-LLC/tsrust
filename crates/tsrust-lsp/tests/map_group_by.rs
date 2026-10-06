use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn map_group_by_has_static_help_and_checked_result_types(){
 let source="function main(){const groups=Map.groupBy([1,2],(value:int,index:int):int=>{return value%2;});const size=groups.size;}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("groups=Map","Map<int, int[]>"),("size=groups","int")]{let at=source.find(needle).unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///map-group-by.tsr").unwrap();assert!(result.to_string().contains(expected),"{result}");}
 let namespace=source.find("Map.groupBy").unwrap();let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":namespace}}),"file:///map-group-by.tsr").unwrap();assert!(hover.to_string().contains("Static members: groupBy"),"{hover}");
 let at=source.find("groupBy(").unwrap();let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":at+8}}),"file:///map-group-by.tsr").unwrap();assert!(help.to_string().contains("independently owned"),"{help}");
 let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at}}),"file:///map-group-by.tsr").unwrap();assert!(items.to_string().contains("groupBy"),"{items}");
}
#[test]
fn map_group_by_catalog_example_checks(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();let entry=&catalog["MapConstructor"][0];let example=&entry["examples"][0];let fixture=include_str!("../../tsrust-compiler/tests/fixtures/map-group-by/documented.tsr");// The fixture may be formatted; compare the example without whitespace.
 let compact=|text:&str|text.chars().filter(|ch|!ch.is_whitespace()).collect::<String>();assert!(compact(fixture).contains(&compact(example["source"].as_str().unwrap())));assert_eq!(example["output"].as_str().unwrap(),include_str!("../../tsrust-compiler/tests/fixtures/map-group-by/documented.stdout"));tsrust_compiler::check(fixture).unwrap();
}
