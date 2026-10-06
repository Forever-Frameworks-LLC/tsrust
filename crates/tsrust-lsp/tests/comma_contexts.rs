use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;
#[test]
fn comma_receivers_and_guards_keep_editor_types_and_formatting() {
    let source="function use(value:int|string):int{if(typeof value===(0,\"string\")){const length=value.length;return length;}return value;}function main(){const values=[1,2];(0,values).push(3);const text=(0,values).join(\",\");use(text);}";
    tsrust_compiler::check(source).unwrap();
    let index=SourceIndex::new(source);
    for(needle,expected) in [("length=","int"),("text=","string")] {
        let at=source.find(needle).unwrap();
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":at}}),"file:///comma-contexts.tsr").unwrap();
        assert!(hover.to_string().contains(expected),"{hover}");
    }
    let formatted=tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
    tsrust_compiler::check(&formatted).unwrap();
}
