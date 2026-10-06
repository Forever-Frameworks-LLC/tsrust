use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn map_for_each_help_and_callback_parameters_keep_key_and_value_types(){
 let source="function main(){const values=new Map<string,int>([['a',1]]);values.forEach((value,key,source)=>{const found=source.has(key);});}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);let at=source.find("forEach(").unwrap();
 for (method,offset) in [("textDocument/hover",at),("textDocument/signatureHelp",at+8)]{
  let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///map-for-each.tsr").unwrap();
  assert!(result.to_string().contains("value: int, key: string, source: Map<string, int>"),"{result}");
 }
 let at=source.find("has(key)").unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///map-for-each.tsr").unwrap();
 assert!(result.to_string().contains("has(key: string): boolean"),"{result}");
}
