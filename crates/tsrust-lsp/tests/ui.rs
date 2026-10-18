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

#[test]
fn counter_state_help_explains_checked_updates_and_ownership() {
    let source="import {signal} from 'std:ui';function main(){const owner=signal(0);owner.set(owner.get() + 1);}";
    let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
    for (needle,expected) in [("signal(0)","independent"),("set(owner","Replaces"),("get()","Reads")] {
        let hover=session.hover("main.tsr",source,source.find(needle).unwrap()).unwrap_or_default();assert!(hover.contains(expected),"{hover}");
    }
}
