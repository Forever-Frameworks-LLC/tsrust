mod support;
#[test]
fn local_content_components_keep_prop_help_and_new_utility_completion(){
 let source="import {Kbd,KbdGroup} from './kbd';import {Separator} from './separator';import {View,Node} from 'std:ui';function Page():Node{return <View as=\"code\"><KbdGroup><Kbd className=\"w-fit text-xs\">K</Kbd></KbdGroup><Separator orientation=\"vertical\"/></View>;}";
 let mut session=support::session::Session::start(&[("kbd.tsrx",include_str!("../../../examples/ui-content/src/components/ui/kbd.tsrx")),("separator.tsrx",include_str!("../../../examples/ui-content/src/components/ui/separator.tsrx"))]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,expected) in [("as=","kbd"),("w-fit","min-content"),("text-xs","12px/16px"),("orientation=","horizontal")] {
  let at=source.find(needle).unwrap()+1;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains(expected),"{needle}: {hover}");
 }
 let at=source.find("w-fit").unwrap()+2;assert!(session.completion("main.tsrx",source,at).unwrap().iter().any(|value|value=="w-fit"));
 let bad=source.replace("orientation=\"vertical\"","orientation=\"diagonal\"");session.change("main.tsrx",&bad);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty());
}
