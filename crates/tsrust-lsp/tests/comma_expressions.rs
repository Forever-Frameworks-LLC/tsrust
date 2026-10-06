use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn comma_results_have_final_type_and_stable_formatting(){
 let source="function main(){const selected=(1,\"last\");let i=0,j=3;for(i=0,j=3;i<3;i++,j++){}const numbers:int[]=(0,[]);numbers.push(2);}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("selected=","string"),("numbers:","int[]")]{let at=source.find(needle).unwrap();let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///comma-expressions.tsr").unwrap();assert!(hover.to_string().contains(expected),"{hover}");}
 let formatted=tsrust_compiler::format(source).unwrap();assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);tsrust_compiler::check(&formatted).unwrap();
}
