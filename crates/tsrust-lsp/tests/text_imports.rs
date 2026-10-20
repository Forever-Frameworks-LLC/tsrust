mod support;

#[test]
fn raw_text_imports_offer_string_members_and_track_unsaved_asset_edits(){
 let source="import prose from './prose.txt';function main(){console.log(prose.trim());}";
 let text="This is not source { café\r\n";
 let mut session=support::session::Session::start(&[("main.tsr",source),("prose.txt",text)]);session.open("main.tsr",source);session.open("prose.txt",text);session.settle();
 let at=source.find("prose.trim").unwrap()+"prose.".len();let members=session.completion("main.tsr",source,at).unwrap();assert!(members.contains(&"trim".into())&&members.contains(&"startsWith".into()),"{members:?}");
 let hover=session.hover("main.tsr",source,at+1).unwrap();assert!(hover.contains("whitespace")&&hover.contains("receiver"),"{hover}");
 let definition=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":source.find("prose.trim").unwrap()}}));
 assert_eq!(definition["uri"],session.uri("prose.txt"),"{definition}");assert_eq!(definition["range"]["start"],serde_json::json!({"line":0,"character":0}));
 session.change("prose.txt","Changed { this is still plain text 🦀");session.settle();assert_eq!(session.completion("main.tsr",source,at).unwrap(),members);
}
