use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn string_iteration_shows_character_type_and_methods(){
 let source="function main(){for(const character of 'abc'){const size=character.length;}}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);let at=source.find("character.length").unwrap();
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///string-iteration.tsr").unwrap();assert!(hover.to_string().contains("character: string"),"{hover}");
 let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at+10}}),"file:///string-iteration.tsr").unwrap();assert!(completion.to_string().contains("toUpperCase"),"{completion}");
}
