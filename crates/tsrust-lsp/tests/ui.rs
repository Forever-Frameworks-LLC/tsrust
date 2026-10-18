mod support;
#[test]
fn ui_component_and_document_help_use_checked_project_types() {
    let source = "import {Text,document} from 'std:ui';function main(){const node=<Text>Hello</Text>;const html=document(node,'Demo');console.log(html);}";
    let mut session = support::session::Session::start(&[]);
    session.open("main.tsrx", source);
    session.settle();
    for (needle, expected) in [("Text>Hello", "text"), ("document(node", "HTML")] {
        let at = source.find(needle).unwrap();
        let hover = session.hover("main.tsrx", source, at).unwrap_or_default();
        assert!(hover.contains(expected), "{hover:?}; {:?}", session.log);
    }
}
