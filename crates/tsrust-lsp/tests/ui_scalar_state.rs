mod support;
#[test]
fn typed_state_members_return_payload_types_and_keep_help() {
 for (initial,ty) in [("'text'","string"),("true","boolean"),("1.25","number"),("1","int")] {
  let source=format!("import {{signal}} from 'std:ui';function main(){{const state=signal({initial});state.get();state.set({initial});}}");
  let mut session=support::session::Session::start(&[]);session.open("main.tsr",&source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
  let at=source.find("state.get").unwrap()+6;let fields=session.completion("main.tsr",&source,at).unwrap();assert_eq!(fields,vec!["get","set"]);
  let hover=session.hover("main.tsr",&source,at).unwrap_or_default();assert!(hover.contains(&format!("get(): {ty}")) && hover.contains("Reads") && hover.contains("Ownership:"),"{hover}");
 }
}

#[test]
fn generic_signal_alias_keeps_public_defaults_and_lifetime_documentation() {
 let source="import {Signal,signal} from 'std:ui';function main(){const count:Signal=signal(1);count.get();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let hover=session.hover("main.tsr",source,source.find("Signal=signal").unwrap()+2).unwrap_or_default();
 assert!(hover.contains("T = int") && hover.contains("defaults to int") && hover.contains("Ownership:"),"{hover}");
}
