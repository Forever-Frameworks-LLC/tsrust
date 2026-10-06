use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn set_for_each_help_and_callback_parameters_keep_collection_types(){
 let source="function main(){const values=new Set<string>(['a']);values.forEach((value,key,source)=>{const found=source.has(key);});}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);let at=source.find("forEach(").unwrap();
 for (method,offset) in [("textDocument/hover",at),("textDocument/signatureHelp",at+8)]{
  let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///set-for-each.tsr").unwrap();
  assert!(result.to_string().contains("value: string, key: string, source: Set<string>"),"{result}");
 }
 let at=source.find("has(key)").unwrap();let result=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///set-for-each.tsr").unwrap();
 assert!(result.to_string().contains("has(value: string): boolean"),"{result}");
}
