use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn map_iteration_has_tuple_and_pattern_types(){
 let source="function main(){const values=new Map<string,int>([['a',3]]);for(const entry of values){const name=entry[0];}for(const [key,value] of values){const copy=value;}}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for (needle,expected) in [("entry[0]","[string, int]"),("copy=value","int")]{
  let at=source.find(needle).unwrap()+if needle=="copy=value"{5}else{0};
  let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///map-iteration.tsr").unwrap();
  assert!(result.to_string().contains(expected),"{result}");
 }
}
