use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn set_relations_have_consistent_help_and_completion(){
 for name in ["isSubsetOf","isSupersetOf","isDisjointFrom"]{
  let source=format!("function main(){{const values=new Set<int>([1]);const result=values.{name}(new Set<string>(['a']));}}");
  tsrust_compiler::check(&source).unwrap();let index=SourceIndex::new(&source);let at=source.find(name).unwrap();
  for(method,offset)in[("textDocument/hover",at),("textDocument/signatureHelp",at+name.len()+1)]{
   let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///set-relations.tsr").unwrap();
   assert!(result.to_string().contains(&format!("{name}(other: Set<OtherKey>): boolean")),"{result}");
  }
  let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at}}),"file:///set-relations.tsr").unwrap();
  for member in ["isSubsetOf","isSupersetOf","isDisjointFrom"]{assert!(completion.to_string().contains(member),"{completion}");}
 }
}
