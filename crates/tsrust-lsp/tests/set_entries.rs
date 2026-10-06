use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn set_entries_hover_and_help_preserve_tuple_key_types(){
 let source="function main(){const values=new Set<string>(['a']);const pairs=values.entries();}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);let at=source.find("entries()").unwrap();
 for (method,offset) in [("textDocument/hover",at),("textDocument/signatureHelp",at+8)]{
  let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///set-entries.tsr").unwrap();
  assert!(result.to_string().contains("entries(): [string, string][]"),"{result}");
 }
 let formatted=tsrust_compiler::format(source).unwrap();tsrust_compiler::check(&formatted).unwrap();
}
