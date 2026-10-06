use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn set_algebra_help_and_result_types_are_consistent(){
 let source="function main(){const left=new Set<int>([1]);const right=new Set<string>(['a']);const joined=left.union(right);const snapshot=joined.values();}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);let at=source.find("union(").unwrap();
 for(method,offset)in[("textDocument/hover",at),("textDocument/signatureHelp",at+6)]{
  let result=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///set-algebra.tsr").unwrap();
  assert!(result.to_string().contains("union(other: Set<OtherKey>): Set<int | OtherKey>"),"{result}");
 }
 let at=source.find("values()").unwrap();let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///set-algebra.tsr").unwrap();
 assert!(hover.to_string().contains("values(): (int|string)[]"),"joined keys missing: {hover}");
 let completion=index.request("textDocument/completion",&json!({"position":{"line":0,"character":at}}),"file:///set-algebra.tsr").unwrap();
 for name in ["union","intersection","difference","symmetricDifference"]{assert!(completion.to_string().contains(name),"{completion}");}
}

#[test]
fn map_union_arrays_and_placeholder_named_enum_keys_keep_exact_types(){
 for(source,needle,expected)in[
  ("function main(){const values=new Map<int,int|string>();const snapshot=values.values();}","values()","values(): (int|string)[]"),
  ("enum ValueKey{Ready}function main(){const values=new Map<ValueKey,int>();values.set(ValueKey.Ready,1);}","set(","set(key: ValueKey, value: int): void"),
 ]{
  tsrust_compiler::check(source).unwrap();let at=source.find(needle).unwrap();let result=SourceIndex::new(source).request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///map-signature.tsr").unwrap();
  assert!(result.to_string().contains(expected),"{result}");
 }
}
