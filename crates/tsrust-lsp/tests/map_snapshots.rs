use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn map_snapshots_preserve_key_value_and_tuple_editor_types(){
 let source="function main(){const values=new Map<string,int>([['a',1]]);const pairs=values.entries();const numbers=values.values();const keys=values.keys();}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for (method,signature) in [("entries","entries(): [string, int][]"),("values","values(): int[]"),("keys","keys(): string[]")]{
  let position=source.find(&format!("{method}()")).unwrap();
  let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":position}}),"file:///map-snapshots.tsr").unwrap();
  assert!(hover.to_string().contains(signature),"{hover}");
  let help=index.request("textDocument/signatureHelp",&json!({"position":{"line":0,"character":position+method.len()+1}}),"file:///map-snapshots.tsr").unwrap();
  assert!(help.to_string().contains(signature),"{help}");
 }
 let formatted=tsrust_compiler::format(source).unwrap();tsrust_compiler::check(&formatted).unwrap();
}
