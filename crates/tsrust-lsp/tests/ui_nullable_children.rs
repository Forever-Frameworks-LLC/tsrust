mod support;
#[test]
fn nullable_component_types_and_child_documentation_agree() {
 let source="import {Node,Text,View,Child,TextChild,signal} from 'std:ui';function Maybe():Node|null{const count=signal(1);const node=<Text>{count}</Text>;return node;}function Missing():undefined{const count=signal(false);return undefined;}function main(){const items:Child[]=[null,Maybe(),undefined];const text:TextChild[]= ['A',null,undefined,'B'];const root=<View><Missing/>{items.remove(1)}<Text>{null}AB{undefined}</Text></View>;}";
 let source=tsrust_compiler::format(source).unwrap();assert_eq!(tsrust_compiler::format(&source).unwrap(),source);
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",&source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,wanted) in [("Maybe():","Node | null"),("Child, TextChild","null"),("TextChild, signal","undefined")] {
  let offset=source.find(needle).unwrap()+2;let hover=session.hover("main.tsrx",&source,offset).unwrap_or_default();assert!(hover.contains(wanted),"{needle}: {hover}");
 }
}
