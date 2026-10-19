mod support;
#[test]
fn conditional_branches_keep_scalar_facts_and_normal_component_help() {
 let source="import {View,Text,Node,signal} from 'std:ui';function Page():Node{const visible=signal(true);const count=signal(1);return <View>{visible.get()?<Text>{count.get()+1}</Text>:null}</View>;}function main(){const node=Page();}";
 let source=tsrust_compiler::format(source).unwrap();assert_eq!(tsrust_compiler::format(&source).unwrap(),source);let mut session=support::session::Session::start(&[]);session.open("main.tsrx",&source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,result) in [("visible.get","get(): boolean"),("count.get","get(): int")] {let at=source.find(needle).unwrap()+needle.len()-2;let hover=session.hover("main.tsrx",&source,at).unwrap_or_default();assert!(hover.contains(result)&&hover.contains("Ownership:"),"{hover}");}
 let hover=session.hover("main.tsrx",&source,source.find("<View").unwrap()+2).unwrap_or_default();assert!(hover.contains("conditional")&&hover.contains("Ownership:"),"{hover}");
}
