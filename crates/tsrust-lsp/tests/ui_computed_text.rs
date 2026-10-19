mod support;
#[test]
fn computed_getters_keep_scalar_hover_completion_and_source_errors() {
 let source="import {Text,Node,signal,TextBinding} from 'std:ui';function Page():Node{const count=signal(1);return <Text>{count.get()+1}</Text>;}function main(){const node=Page();}";
 let source=tsrust_compiler::format(source).unwrap();assert_eq!(tsrust_compiler::format(&source).unwrap(),source);let mut session=support::session::Session::start(&[]);session.open("main.tsrx",&source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("get()").unwrap()+1;let hover=session.hover("main.tsrx",&source,at).unwrap_or_default();assert!(hover.contains("get(): int")&&hover.contains("Ownership:"),"{hover}");
 let fields=session.completion("main.tsrx",&source,source.find("count.get").unwrap()+"count.".len()).unwrap();assert_eq!(fields,vec!["get","set"]);
 let binding=source.find("TextBinding").unwrap()+2;let hover=session.hover("main.tsrx",&source,binding).unwrap_or_default();assert!(hover.contains("expression")&&hover.contains("Ownership:"),"{hover}");
}
