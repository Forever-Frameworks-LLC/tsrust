mod support;
#[test]
fn migration_completion_and_help_explain_recovery(){
 let source=include_str!("../../../examples/sql-migrations/main.tsr");let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();let offset=source.find("pool.migrate").unwrap()+5;
 let position=source.find("pool.migrate(").unwrap()+"pool.migrate(".len();let before=&source[..position];let line=before.bytes().filter(|b|*b==b'\n').count();let character=before.rsplit('\n').next().unwrap().len();let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":line,"character":character}}));assert!(help.to_string().contains("directory: string"),"{help}");
 let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains("migrate(directory: string)")&&hover.contains("migration-dirty")&&hover.contains("Ownership"),"{hover}");let completion=session.completion("main.tsr",source,offset).unwrap();assert!(completion.iter().any(|name|name=="migrate"),"{completion:?}");
}
