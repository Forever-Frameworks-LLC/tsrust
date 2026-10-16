mod support;
#[test]
fn retained_state_example_has_current_types_and_help(){
 use support::session::Session;
 let source=include_str!("../../../examples/http-state/main.tsr");let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();
 let offset=source.find("config.greeting").unwrap()+7;let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains("string"),"{hover}");
 let completion=session.completion("main.tsr",source,offset).unwrap();assert!(completion.iter().any(|name|name=="greeting"),"{completion:?}");
 let offset=source.find("app.get").unwrap()+4;let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains("retained")&&hover.contains("read-only"),"{hover}");
}
