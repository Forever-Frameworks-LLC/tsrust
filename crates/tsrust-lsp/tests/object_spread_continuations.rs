use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn nested_spread_completion_and_hover_keep_checked_fields(){
    let value=format!("{}source{}","{...".repeat(32),"}".repeat(32));
    let source=format!("type Item={{value:int;text:string;}};function main(){{const source:Item={{text:'retained',value:7}};const result={value};console.log(result.value);}}");
    tsrust_compiler::check(&source).unwrap();
    let index=SourceIndex::new(&source);let at=source.find("result.value").unwrap()+7;
    let params=json!({"position":{"line":0,"character":at}});
    let result=index.request("textDocument/completion",&params,"file:///object-spread.tsr").unwrap();
    let text=result.to_string();assert!(text.contains("value"),"{result}");assert!(text.contains("text"),"{result}");
    let hover=index.request("textDocument/hover",&params,"file:///object-spread.tsr").unwrap();
    assert!(hover.to_string().contains("int"),"{hover}");
}
