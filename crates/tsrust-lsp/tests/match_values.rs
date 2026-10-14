use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn expression_match_payloads_and_results_have_checked_hover_and_navigation() {
    for (source, needle, ty) in [
        ("function main(){const item:Result<string,int>=Ok(\"Ada\");const answer=match(item){Ok(user)=>user.length,Err(code)=>code,};console.log(answer);}", "user.length", "string"),
        ("type User={name:string;};type Guest={code:int;};function get(item:User|Guest):int{return match(item){User(user)=>user.name.length,Guest(guest)=>guest.code,};}", "user.name", "User"),
    ] {
        tsrust_compiler::check(source).unwrap();
        let index=SourceIndex::new(source);
        let offset=source.find(needle).unwrap();
        let position=json!({"position":{"line":0,"character":offset}});
        let hover=index.request("textDocument/hover",&position,"file:///match.tsr").unwrap();
        assert!(hover.to_string().contains(ty),"{hover}");
        let definition=index.request("textDocument/definition",&position,"file:///match.tsr").unwrap();
        assert!(!definition.is_null(),"{definition}");
        let outside=source.rfind('}').unwrap();
        let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":outside}}),"file:///match.tsr").unwrap();
        assert!(!items.as_array().unwrap().iter().any(|item|item["label"]=="user"),"arm binding leaked: {items}");
    }
}

#[test]
fn pattern_completion_omits_written_alternatives() {
    for (source, present, absent) in [
        ("type User={name:string;};type Guest={code:int;};function get(item:User|Guest):int{return match(item){User(user)=>user.name.length, ","Guest","User"),
        ("function get(item:boolean):int{return match(item){true=>1, ","false","true"),
    ] {
        let index=SourceIndex::new(source);
        let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":source.len()}}),"file:///match.tsr").unwrap();
        let items=items.as_array().unwrap();
        assert!(items.iter().any(|item|item["label"]==present),"{items:?}");
        assert!(!items.iter().any(|item|item["label"]==absent),"{items:?}");
    }
}
