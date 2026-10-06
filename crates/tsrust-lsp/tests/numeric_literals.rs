use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn numeric_literal_forms_have_checked_hover_and_stable_formatting(){
 let source="function main(){const hex=0xDE_AD;const fraction:number=.1_2e-0_1;const bits=0b1010_0011;const inferred=1_234.5_6e+0_1;}";
 tsrust_compiler::check(source).unwrap();let index=SourceIndex::new(source);
 for(needle,expected)in[("hex=","int"),("fraction:","number"),("bits=","int"),("inferred=","number")]{let at=source.find(needle).unwrap();let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///numeric-literals.tsr").unwrap();assert!(hover.to_string().contains(expected),"{hover}");}
 let formatted=tsrust_compiler::format(source).unwrap();for literal in ["0xDE_AD",".1_2e-0_1","0b1010_0011"]{assert!(formatted.contains(literal),"{formatted}");}assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);tsrust_compiler::check(&formatted).unwrap();
}
