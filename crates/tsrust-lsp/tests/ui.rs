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

#[test]
fn root_page_import_supports_navigation_and_button_behavior_help() {
    let page="import {Button,Node} from 'std:ui';export function Page():Node{return <Button disabled>Ready</Button>;}";
    let main="import {Page} from '../app/page';import {run} from 'std:ui';function main(){unwrap(run(Page(),'UI'));}";
    let mut session=support::session::Session::start(&[("app/page.tsrx",page),("src/main.tsr",main)]);
    session.open("app/page.tsrx",page);session.open("src/main.tsr",main);session.settle();
    let help=session.hover("app/page.tsrx",page,page.find("Button disabled").unwrap()).unwrap_or_default();
    assert!(help.contains("centered") && help.contains("focus outline"),"{help}");
    let help=session.hover("src/main.tsr",main,main.find("Page(),'UI'").unwrap()).unwrap_or_default();
    assert!(help.contains("Node"),"{help}");
}

#[test]
fn ui_class_completion_hover_and_theme_navigation_use_shared_contract() {
 let theme=tsrust_compiler::ui_project::theme::DEFAULT_JSON;
 let source="import {View,withTheme,Node} from 'std:ui';import theme from './ui.theme.json';function Page():Node{return withTheme(<View className=\"bg-primary p-4\" />,theme);}";
 let mut session=support::session::Session::start(&[("ui.theme.json",theme),("main.tsrx",source)]);
 session.open("main.tsrx",source);session.settle();let at=source.find("bg-primary").unwrap()+3;
 assert!(session.completion("main.tsrx",source,at).unwrap().contains(&"bg-primary".into()));
 assert!(session.hover("main.tsrx",source,at).unwrap().contains("Shared web/native"));
 let uri=session.uri("main.tsrx");let locations=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at}}));
 assert!(locations.as_array().is_some_and(|v|v.len()==2),"{locations}");
 session.open("ui.theme.json",theme);session.settle();let at=theme.find("primary\"").unwrap()+4;
 assert!(session.hover("ui.theme.json",theme,at).unwrap().contains("#RRGGBB"));
 let broken=theme.replacen("#2563eb","#12345680",1);session.change("ui.theme.json",&broken);
 let mut diagnostics=session.wait_for_diagnostics("ui.theme.json");
 for _ in 0..3 {if diagnostics.to_string().contains("ui-theme"){break;}diagnostics=session.wait_for_diagnostics("ui.theme.json");}
 assert!(diagnostics.to_string().contains("ui-theme"),"{diagnostics}");
 session.change("ui.theme.json",theme);session.settle();
 assert!(session.hover("main.tsrx",source,source.find("withTheme(<View").unwrap()).unwrap().contains("Scopes"));
}

#[test]
fn editable_component_variants_complete_and_navigate_to_owned_source() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-compiler/src/ui-catalog/components-1.0.0.json")).unwrap();
 let button=catalog[0]["files"][0]["content"].as_str().unwrap();
 let source="import {Button} from './button';import {Node} from 'std:ui';function Page():Node{return <Button variant=\"out\" disabled>Done</Button>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[("button.tsrx",button),("main.tsrx",source)]);session.open("main.tsrx",source);session.settle();
 let at=source.find("out\"").unwrap()+3;assert_eq!(session.completion("main.tsrx",source,at).unwrap(),vec!["outline"]);
 let inherited=session.completion("main.tsrx",source,source.find("variant=").unwrap()).unwrap();for field in ["disabled","className","onPress","size","variant"]{assert!(inherited.contains(&field.into()),"{inherited:?}");}
 let prop=source.find("variant=").unwrap();assert!(session.hover("main.tsrx",source,prop+2).unwrap().contains("outline"));
 let uri=session.uri("main.tsrx");let definition=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":prop+2}}));
 assert!(definition.to_string().contains("button.tsrx"),"{definition}");
}

#[test]
fn incompatible_ui_lock_publishes_a_readable_project_error() {
 let config=r#"{"ui":{"theme":"ui.theme.json","components":"components/ui","lock":"ui.lock.json"}}"#;
 let lock=r#"{"schemaVersion":1,"installed":[{"compilerApi":999,"runtimeAbi":"old","rust":"old","compilerSha256":"old"}]}"#;
 let source="function main(){}";
 let mut session=support::session::Session::start(&[("tsrust.config.json",config),("ui.lock.json",lock),("main.tsr",source)]);
 session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");assert!(diagnostics.to_string().contains("migrate explicitly"),"{diagnostics}");
}
