mod support;
#[test]
fn generic_alias_members_hover_and_imports_follow_compiler_expansion() {
    let types = "export interface Box<T>{value:T;}export type Values<T=string>=T[];export type Rows<T=int>=Box<T>[];";
    let source = "import {Values,Rows} from './types';type Optional<T>=T|undefined;function main(){const words:Values=['text'];words.length;const rows:Rows=[{value:7}];rows[0].value;}";
    let mut session = support::session::Session::start(&[("types.tsr", types)]);
    session.open("main.tsr", source);
    let diagnostics = session.wait_for_diagnostics("main.tsr");
    session.settle();
    assert_eq!(diagnostics.as_array().unwrap().len(), 0, "{diagnostics}");
    let at = source.find("words.length").unwrap() + 6;
    let fields = session.completion("main.tsr", source, at).unwrap();
    assert!(fields.contains(&"length".into()), "{fields:?}");
    let at = source.find("rows[0].value").unwrap() + 8;
    assert!(
        session
            .completion("main.tsr", source, at)
            .unwrap()
            .contains(&"value".into())
    );
    let hover = session
        .hover(
            "main.tsr",
            source,
            source.find("Values=['text']").unwrap() + 2,
        )
        .unwrap_or_default();
    assert!(
        hover.contains("T = string") && hover.contains("T[]"),
        "{hover}"
    );
}

#[test]
fn imported_generic_native_aliases_keep_defaults_members_and_definition() {
    let types="import {LruCache} from 'std:cache';export type Cache<K=int,V=string>=LruCache<K,V>;";
    let source="import {LruCache} from 'std:cache';import {Cache} from './types';function main(){const cache:Cache=new LruCache<int,string>(2);cache.size();}";
    let mut session=support::session::Session::start(&[("types.tsr",types)]);
    session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert_eq!(diagnostics.as_array().unwrap().len(),0,"{diagnostics}");
    let at=source.find("cache.size").unwrap()+6;
    let fields=session.completion("main.tsr",source,at).unwrap();assert!(fields.contains(&"get".into()) && fields.contains(&"size".into()),"{fields:?}");
    let at=source.find("Cache=new").unwrap()+2;
    let hover=session.hover("main.tsr",source,at).unwrap_or_default();assert!(hover.contains("K = int") && hover.contains("V = string"),"{hover}");
    let uri=session.uri("main.tsr");
    let definition=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at}}));
    assert!(definition.to_string().contains("types.tsr"),"{definition}");
}
