mod support;
use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn copied_map_type_survives_incomplete_editor_source(){
 let source="function main(){const original:Map<int,string>=new Map<int,string>();const copied=new Map(original);copied.get(1);const unfinished=;}";
 let index=SourceIndex::new(source);
 let position=source.find("get(1)").unwrap();
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":position}}),"file:///maps.tsr").unwrap();
 assert!(hover.to_string().contains("get(key: int): string | undefined"),"{hover}");
}
#[test]
fn map_catalog_examples_match_native_fixture_and_constructor_highlighting(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let fixture=include_str!("../../tsrust-compiler/tests/fixtures/maps/documented.tsr");
 let mut expected=String::new();
 let mut entries=catalog["Map"].as_array().unwrap().clone();
 entries.extend(catalog["globals"].as_array().unwrap().iter().filter(|entry|entry["name"]=="Map").cloned());
 for entry in entries {for example in entry["examples"].as_array().unwrap(){
  let source=example["source"].as_str().unwrap();assert!(support::contains_formatted_snippet(fixture,source));
  tsrust_compiler::check(&format!("function main(){{{source}}}")).unwrap();
  expected.push_str(example["output"].as_str().unwrap());
 }}
 assert_eq!(expected,include_str!("../../tsrust-compiler/tests/fixtures/maps/documented.stdout"));
 let grammar=include_str!("../../../editors/vscode/syntaxes/tsrust.tmLanguage.json");
 assert!(grammar.contains("Owned|Set|Map"));
}
#[test]
fn map_members_have_typed_hover_completion_and_parameter_help(){
 let source="function main(){const values:Map<string,int> =new Map<string,int>();values.set('a',1);const read=values.get('a');}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 let position=source.find("get('a')").unwrap();
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":position}}),"file:///maps.tsr").unwrap();
 assert!(hover.to_string().contains("get(key: string): int | undefined"),"{hover}");
 let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":position}}),"file:///maps.tsr").unwrap();
 for member in ["get","set","has","delete","clear","size"] {assert!(completion.to_string().contains(member),"{completion}");}
 let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":position+4}}),"file:///maps.tsr").unwrap();
 assert!(help.to_string().contains("get(key: string): int | undefined"),"{help}");
}

#[test]
fn inferred_map_constructor_help_and_formatting_are_consistent(){
 let source="function main(){const values=new Map<string,int>();values.set('a',1);}";
 let index=SourceIndex::new(source);
 let position=source.find("set('a'").unwrap();
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":position}}),"file:///maps.tsr").unwrap();
 assert!(hover.to_string().contains("set(key: string, value: int): void"),"{hover}");
 let open=source.find("();").unwrap()+1;
 let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":open}}),"file:///maps.tsr").unwrap();
 assert!(help.to_string().contains("new Map<string, int>"),"{help}");
 let formatted=tsrust_compiler::format(source).unwrap();tsrust_compiler::check(&formatted).unwrap();
 assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
}
