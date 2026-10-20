mod support;

#[test]
fn application_modes_visibility_and_window_options_have_checked_editor_help() {
 let theme=include_str!("../../../examples/ui-application-state/tsrust.config.json");
 let source="import {View,Text,Button,Node,signal,withTheme,run,WindowOptions} from 'std:ui';import config from './theme.json';function Page():Node{const mode=signal('light');const hidden=signal(false);return withTheme(<View windowDragRegion tabIndex={0} role=\"region\" aria-label=\"Article\" className=\"h-screen overflow-auto\"><Button onPress={()=>mode.set('dark')}>Dark</Button><View hidden={hidden.get()}><Text>Preview</Text></View></View>,config.ui.theme,mode);}function main(){const options:WindowOptions={width:1040,titlebar:'custom'};const result=run(Page(),'Docs',options);}";
 let mut session=support::session::Session::start(&[("theme.json",theme)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,expected) in [("tabIndex={","sequential"),("overflow-auto","scroll"),("h-screen","viewport"),("hidden={","retaining"),("windowDragRegion tab","disabled"),("withTheme(<","Signal<string>"),("run(Page()","WindowOptions"),("width:1040","16384"),("WindowOptions={","creation")] {
  let help=session.hover("main.tsrx",source,source.find(needle).unwrap()+2).unwrap_or_default();assert!(help.contains(expected),"{needle}: {help}");
 }
 let broken=source.replace("titlebar:'custom'","titlebar:'frameless'");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}
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
 let broken=theme.replacen("#2563eb","#12345g80",1);session.change("ui.theme.json",&broken);
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

#[test]
fn inline_config_theme_supports_help_navigation_and_unsaved_validation() {
 let theme:serde_json::Value=serde_json::from_str(tsrust_compiler::ui_project::theme::DEFAULT_JSON).unwrap();
 let config=serde_json::json!({"schemaVersion":1,"name":"inline-ui","build":{"entry":"src/main.tsrx","output":"target/app"},"ui":{"target":{"native":false,"web":true,"embedded":false},"theme":theme,"components":"src/components/ui","lock":"ui-components.lock.json"}}).to_string();
 let source="import {View,withTheme,Node} from 'std:ui';import config from '../tsrust.config.json';function main(){const node=withTheme(<View className=\"bg-primary p-4\"/>,config.ui.theme);}";
 let mut session=support::session::Session::start(&[("tsrust.config.json",&config),("src/main.tsrx",source)]);
 session.open("src/main.tsrx",source);session.open("tsrust.config.json",&config);session.settle();
 let target_at=config.find("\"native\"").unwrap()+3;assert!(session.hover("tsrust.config.json",&config,target_at).unwrap().contains("Boolean"));assert!(session.completion("tsrust.config.json",&config,target_at).unwrap().contains(&"native".into()));
 let at=config.find("\"primary\"").unwrap()+4;assert!(session.hover("tsrust.config.json",&config,at).unwrap().contains("#RRGGBB"));
 assert!(session.completion("tsrust.config.json",&config,at).unwrap().contains(&"primary".into()));
 let uri=session.uri("src/main.tsrx");let location=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":source.find("bg-primary").unwrap()+4}}));
 assert_eq!(location.as_array().unwrap().len(),2,"{location}");assert!(location.to_string().contains("tsrust.config.json"));
 let bad=config.replacen("#2563eb","#12345g80",1);session.change("tsrust.config.json",&bad);
 let mut diagnostics=session.wait_for_diagnostics("tsrust.config.json");for _ in 0..3{if diagnostics.to_string().contains("#RRGGBB"){break;}diagnostics=session.wait_for_diagnostics("tsrust.config.json");}
 assert!(diagnostics.to_string().contains("#RRGGBB"),"{diagnostics}");
 assert!(session.completion("src/main.tsrx",source,source.find("bg-primary").unwrap()+4).unwrap().contains(&"bg-primary".into()));
 session.change("tsrust.config.json",&config);session.settle();assert!(session.hover("src/main.tsrx",source,source.find("withTheme(<").unwrap()+3).unwrap().contains("Scopes"));
}

#[test]
fn composed_child_help_and_optional_handler_behavior_are_current() {
 let source="import {Button,View,Text,Child,Node} from 'std:ui';function Page():Node{return <Button><View as=\"span\">Ready</View></Button>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);session.settle();
 assert!(session.hover("main.tsrx",source,source.find("Button><").unwrap()+2).unwrap().contains("onPress is optional"));
 assert!(session.hover("main.tsrx",source,source.find("Child,Node").unwrap()+2).unwrap().contains("non-retaining"));
}

#[test]
fn owning_child_array_forwarding_preserves_editor_types() {
    let source="import {View,Text,Child,Node} from 'std:ui';function Wrap(children:Node[]):Node{return View({children:children});}function main(){const text=['first'];const mixed:Child[]=text;const child=Text({children:['owned']});const nodes=[child];const view=View({children:nodes});}";
    let mut session=support::session::Session::start(&[]);
    session.open("main.tsrx",&source);
    let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();
    assert_eq!(diagnostics.as_array().unwrap().len(),0,"{diagnostics}");
    let hover=session.hover("main.tsrx",&source,source.find("mixed:Child").unwrap()).unwrap_or_default();
    assert!(hover.contains("Child") || hover.contains("Node"),"{hover}");
}

#[test]
fn union_child_arrays_reject_resource_copying_in_editor() {
    let source="import {Text,Child} from 'std:ui';function main(){const child:Child=<Text>owned</Text>;const children=[child];const copy=clone(children);}";
    let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);
    let diagnostics=session.wait_for_diagnostics("main.tsrx");
    assert!(diagnostics.to_string().contains("clone-type"),"{diagnostics}");
}

#[test]
fn layout_utility_help_and_theme_references_come_from_shared_rules() {
    let config=serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string();
    let source="import {View,withTheme} from 'std:ui';import config from './tsrust.config.json';function main(){const node=withTheme(<View className=\"grid grid-cols-2 w-64 pl-3\"/>,config.ui.theme);}";
    let mut session=support::session::Session::start(&[("tsrust.config.json",&config)]);session.open("main.tsrx",source);session.settle();
    let at=source.find("grid-cols-2").unwrap()+4;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("zero minimum"),"{hover}");
    let at=source.find("w-64").unwrap()+2;let uri=session.uri("main.tsrx");let definitions=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at}}));assert_eq!(definitions.as_array().unwrap().len(),1,"{definitions}");
    let at=source.find("pl-3").unwrap()+2;let uri=session.uri("main.tsrx");let definitions=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at}}));assert_eq!(definitions.as_array().unwrap().len(),2,"{definitions}");
}

#[test]
fn color_opacity_help_and_theme_ranges_match_runtime_rules() {
 let theme=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-colors/tsrust.config.json")).unwrap();
 let source="import {View,withTheme} from 'std:ui';import config from './tsrust.config.json';function main(){const node=withTheme(<View className=\"bg-primary/50 ring-1 ring-foreground/10\"/>,config.ui.theme);}";
 let mut session=support::session::Session::start(&[("tsrust.config.json",&theme)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 let hover=session.hover("main.tsrx",source,source.find("bg-primary/50").unwrap()+4).unwrap();assert!(hover.contains("multiplies") && hover.contains("8-bit"),"{hover}");
 let at=source.find("bg-primary/50").unwrap()+4;let uri=session.uri("main.tsrx");let locations=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at}}));assert_eq!(locations.as_array().unwrap().len(),2,"{locations}");
 session.open("tsrust.config.json",&theme);session.settle();let bad=theme.replace("oklch(0.6 0.12 260 / 0.8)","oklch(0.6 0.8 30)");session.change("tsrust.config.json",&bad);let diagnostics=session.wait_for_diagnostics("tsrust.config.json");assert!(diagnostics.to_string().contains("sRGB gamut"),"{diagnostics}");
}

#[test]
fn extended_jsx_forwarding_metadata_and_structural_help_are_checked() {
 let source="import {View,ViewProps,Text,Node} from 'std:ui';interface Props extends ViewProps {size?:'sm'|'default';}function Forward(props:Props):Node{return <View {...props} data-size=\"sm\"/>;}function main(){const node=<Forward size=\"sm\"><Text>Owned</Text></Forward>;const other=<View className=\"group-data-[size=sm]/panel:hover:bg-primary/50\"/>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert_eq!(diagnostics.as_array().unwrap().len(),0,"{diagnostics}");
 let help=session.hover("main.tsrx",source,source.find("data-size=").unwrap()+3).unwrap_or_default();assert!(help.contains("metadata"),"{help}");
 let at=source.find("group-data-").unwrap()+7;let help=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(help.contains("excluding self"),"{help}");
 let at=source.find("bg-primary/50").unwrap()+13;assert!(session.completion("main.tsrx",source,at).unwrap().contains(&"group-data-[size=sm]/panel:hover:bg-primary/50".into()));
 let broken=source.replace("<View {...props} data-size=", "<View {...props} size=");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.to_string().contains("Unknown JSX attribute"),"{diagnostics}");
}

#[test]
fn card_parts_spacing_typography_and_radius_help_follow_owned_sources() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-compiler/src/ui-catalog/components-3.0.0.json")).unwrap();let card=catalog[0]["files"][0]["content"].as_str().unwrap();
 let source="import {Card,CardHeader,CardTitle} from './card';import {View,ThemeRadius} from 'std:ui';function main(){const card=<Card size=\"sm\"><CardHeader><CardTitle>Title</CardTitle></CardHeader></Card>;const n=<View className=\"[--panel:--spacing(4)] p-(--panel) text-sm font-medium rounded-xl overflow-hidden\"/>;}";
 let mut session=support::session::Session::start(&[("card.tsrx",card)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert_eq!(diagnostics.as_array().unwrap().len(),0,"{diagnostics}");
 for (needle,expected) in [("CardHeader><","Header grid"),("p-(--panel)","missing variable"),("text-sm","14px/20px"),("rounded-xl","radius.xl"),("overflow-hidden","pointer targets")] {
  let help=session.hover("main.tsrx",source,source.find(needle).unwrap()+3).unwrap_or_default();assert!(help.to_lowercase().contains(&expected.to_lowercase()),"{needle}: {help}");
 }
}

#[test]
fn semantic_content_orientation_fit_and_window_scrolling_help_match_checked_props() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-compiler/src/ui-catalog/semantic-content-1.0.0.json")).unwrap();
 let kbd=catalog[0]["files"][0]["content"].as_str().unwrap();let separator=catalog[1]["files"][0]["content"].as_str().unwrap();
 let source="import {Kbd} from './kbd';import {Separator} from './separator';import {View,run} from 'std:ui';function main(){const node=<View><Kbd className=\"w-fit text-xs\">Ctrl</Kbd><Separator orientation=\"vertical\"/><View as=\"code\">Code</View></View>;const result=run(node,'Demo');}";
 let mut session=support::session::Session::start(&[("kbd.tsrx",kbd),("separator.tsrx",separator)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,expected) in [("orientation=","vertical"),("w-fit","fit-content"),("text-xs","12"),("run(node","PageUp")] {
  let help=session.hover("main.tsrx",source,source.find(needle).unwrap()+2).unwrap_or_default();assert!(help.contains(expected),"{needle}: {help}");
 }
 let broken=source.replace("orientation=\"vertical\"","orientation=\"diagonal\"");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}

#[test]
fn generated_documentation_types_have_nested_member_help_without_running_a_build(){
 let model=tsrust_compiler::documentation::MODEL_SOURCE;
 let source="import {DocCatalog} from './content-types';function read(value:ReadOnly<DocCatalog>):int{return value.pages[0].document.nodes[0].source.start;}function main(){}";
 let mut session=support::session::Session::start(&[("content-types.tsr",model)]);session.open("main.tsr",source);session.settle();
 let at=source.find("source.start").unwrap()+7;let hover=session.hover("main.tsr",source,at).unwrap_or_default();assert!(hover.contains("int"),"{hover}");
 let at=source.find("source.start").unwrap()+7;let names=session.completion("main.tsr",source,at).unwrap();for name in ["start","end"]{assert!(names.contains(&name.into()),"{names:?}");}
}

#[test]
fn typed_json_literal_union_help_and_inference_agree(){
 let source="type Status='draft'|'published';function main(){const status=JSON.parse<Status>('\"draft\"');console.log(status);}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("parse<").unwrap()+1;let hover=session.hover("main.tsr",source,at).unwrap_or_default();assert!(hover.contains("literal")&&hover.contains("shape-mismatch"),"{hover}");
}

#[test]
fn documentation_example_card_keeps_owned_children_and_source_props_in_editor(){
 let card=include_str!("../../../docs/src/components/docs/example-card.tsrx");
 let button=include_str!("../../../docs/src/components/ui/button.tsrx");let surface=include_str!("../../../docs/src/components/ui/card.tsrx");let textarea=include_str!("../../../docs/src/components/ui/textarea.tsrx");
 let source="import {ExampleCard} from './components/docs/example-card';import {Text,Node} from 'std:ui';function Page():Node{return (<ExampleCard id=\"example\" title=\"Example\" description=\"Live preview\" source=\"example.tsrx\" code=\"export function Example(){}\"><Text>Live child</Text></ExampleCard>);}function main(){}";
 let mut session=support::session::Session::start(&[("components/docs/example-card.tsrx",card),("components/ui/button.tsrx",button),("components/ui/card.tsrx",surface),("components/ui/textarea.tsrx",textarea)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let hover=session.hover("main.tsrx",source,source.find("code=\"").unwrap()+1).unwrap_or_default();assert!(hover.contains("string"),"{hover}");
 let broken=source.replace("code=\"export function Example(){}\"","code={true}");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}

#[test]
fn explicit_children_completion_hover_and_conflicts_match_the_compiler(){
 let source="import {View,Text,Node} from 'std:ui';function Page():Node{return (<View children={[<Text>Owned child</Text>]}/>);}function main(){}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("children=").unwrap();let help=session.hover("main.tsrx",source,at+2).unwrap_or_default();assert!(help.contains("children")&&help.contains("once"),"{help}");
 let partial=source.replace("children={[<Text>Owned child</Text>]}","chi");session.change("main.tsrx",&partial);session.settle();let names=session.completion("main.tsrx",&partial,partial.find("chi/").unwrap()+3).unwrap();assert!(names.contains(&"children".into()),"{names:?}");
 let broken=source.replace("/>);",">Duplicate</View>);");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}

#[test]
fn application_links_location_and_display_type_help_match_checked_source(){
 let source="import {View,Text,Link,Node,location,navigate,back,forward} from 'std:ui';function App():Node{const path=location();navigate('/docs');return <View><Link href=\"/docs\" aria-label=\"Read docs\">Docs</Link><Text className=\"text-3xl\">{path.get()}</Text><View hidden={path.get()!=='/docs'}><Text>Article</Text></View></View>;}function main(){}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("<Link ").unwrap()+6;let completions=session.completion("main.tsrx",source,at).unwrap();for name in ["href","children","className","aria-label"]{assert!(completions.contains(&name.to_owned()),"{completions:?}");}
 let at=source.find("navigate('/docs')").unwrap()+10;let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsrx")},"position":{"line":0,"character":at}}));assert!(signature.to_string().contains("path: string"),"{signature}");
 for (needle,expected) in [("location();","single application"),("href=","absolute"),("text-3xl","36"),("<Link ","anchor")]{let at=source.find(needle).unwrap()+if needle=="<Link "{2}else{1};let help=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(help.to_lowercase().contains(&expected.to_lowercase()),"{needle}: {help}");}
}

#[test]
fn inline_text_and_clipboard_help_match_paired_components(){
 let source="import {Text,CopyButton} from 'std:ui';function main(){const node=<Text as=\"p\">Read <Text as=\"strong\">bold</Text> <Text as=\"em\">café</Text></Text>;const copy=<CopyButton value=\"exact\" className=\"text-syntax-string\"/>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,expected) in [("Text as=\"p\"","paragraph"),("CopyButton value=","clipboard"),("value=\"exact\"","Unicode"),("text-syntax-string","syntax-string")] {let help=session.hover("main.tsrx",source,source.find(needle).unwrap()+1).unwrap_or_default();assert!(help.to_lowercase().contains(&expected.to_lowercase()),"{needle}: {help}");}
 let choices=session.completion("main.tsrx",source,source.find("value=\"exact\"").unwrap()).unwrap();assert!(choices.contains(&"value".into()));assert!(choices.contains(&"className".into()));
 let theme=tsrust_compiler::ui_project::theme::DEFAULT_JSON;
 for (file,text) in [("ui.theme.json",theme.to_owned()),("palette.json",theme.to_owned()),("tsrust.config.json",format!("{{\"ui\":{{\"theme\":{theme}}}}}"))] {
  session.open(file,&text);session.settle();let at=text.find("syntaxKeyword").unwrap()+4;let help=session.hover(file,&text,at).unwrap_or_default();assert!(help.contains("Optional code syntax color"),"{file}: {help}");let keys=session.completion(file,&text,at).unwrap();assert!(keys.contains(&"syntaxKeyword".into()));
 }
}

#[test]
fn visibility_and_external_links_offer_matching_help_and_diagnostics(){
 let source="import {View,Text,Link,Node,signal} from 'std:ui';function Page():Node{const seen=signal(false);return <View onVisibilityChange={(visible)=>{seen.set(visible);}}><Text className=\"text-7xl leading-tight\">{seen.get() ? 'Seen' : 'Waiting'}</Text><Link href=\"https://github.com/Forever-Frameworks-LLC/tsrust\">GitHub</Link></View>;}function main(){}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("onVisibilityChange=").unwrap();let help=session.hover("main.tsrx",source,at+3).unwrap_or_default();assert!(help.contains("clipping ancestors")&&help.contains("non-retaining"),"{help}");
 let choices=session.completion("main.tsrx",source,at).unwrap();assert!(choices.contains(&"onVisibilityChange".into()),"{choices:?}");
 let help=session.hover("main.tsrx",source,source.find("Link href=").unwrap()+1).unwrap_or_default();assert!(help.contains("HTTPS")&&help.contains("history unchanged"),"{help}");
 let help=session.hover("main.tsrx",source,source.find("text-7xl").unwrap()+3).unwrap_or_default();assert!(help.contains("96px"),"{help}");
 let broken=source.replace("(visible)=>","(visible:string)=>");session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}

#[test]
fn local_component_gallery_examples_keep_checked_props_and_change_help() {
 let files=[
  ("components/ui/button.tsrx",include_str!("../../../docs/src/components/ui/button.tsrx")),
  ("components/ui/card.tsrx",include_str!("../../../docs/src/components/ui/card.tsrx")),
  ("components/ui/input.tsrx",include_str!("../../../docs/src/components/ui/input.tsrx")),
  ("components/ui/textarea.tsrx",include_str!("../../../docs/src/components/ui/textarea.tsrx")),
  ("components/ui/label.tsrx",include_str!("../../../docs/src/components/ui/label.tsrx")),
  ("components/ui/kbd.tsrx",include_str!("../../../docs/src/components/ui/kbd.tsrx")),
  ("components/ui/separator.tsrx",include_str!("../../../docs/src/components/ui/separator.tsrx")),
  ("examples/card-layout.tsrx",include_str!("../../../docs/src/examples/card-layout.tsrx")),
  ("examples/card-project.tsrx",include_str!("../../../docs/src/examples/card-project.tsrx")),
  ("examples/input-states.tsrx",include_str!("../../../docs/src/examples/input-states.tsrx")),
  ("examples/multiline-draft.tsrx",include_str!("../../../docs/src/examples/multiline-draft.tsrx")),
  ("examples/field-labels.tsrx",include_str!("../../../docs/src/examples/field-labels.tsrx")),
  ("examples/keyboard-shortcuts.tsrx",include_str!("../../../docs/src/examples/keyboard-shortcuts.tsrx")),
  ("examples/section-dividers.tsrx",include_str!("../../../docs/src/examples/section-dividers.tsrx")),
 ];
 let mut session=support::session::Session::start(&files);
 for (file,source) in &files[7..] {
  session.open(file,source);let diagnostics=session.wait_for_diagnostics(file);session.settle();
  assert!(diagnostics.as_array().unwrap().is_empty(),"{file}: {diagnostics}");
 }
 let (file,source)=files[9];
 let at=source.find("event.target.value").unwrap()+13;
 assert!(session.hover(file,source,at).unwrap_or_default().contains("string"));
 let at=source.find("aria-invalid=").unwrap()+3;
 assert!(session.hover(file,source,at).unwrap_or_default().contains("validation"));
 let (file,source)=files[13];let at=source.find("orientation=").unwrap();
 let names=session.completion(file,source,at).unwrap();assert!(names.contains(&"orientation".into()),"{names:?}");
 let broken=source.replace("orientation=\"vertical\"","orientation=\"diagonal\"");session.change(file,&broken);
 let diagnostics=session.wait_for_diagnostics(file);assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
}

#[test]
fn aspect_ratio_component_required_props_and_geometry_help_agree(){
 let component=include_str!("../../../docs/src/components/ui/aspect-ratio.tsrx");
 let source="import {AspectRatio} from './aspect-ratio';import {Text,Node} from 'std:ui';function Page():Node{return <AspectRatio ratio={1.5} className=\"aspect-video\"><Text>Ratio</Text></AspectRatio>;}function main(){}";
 let mut session=support::session::Session::start(&[("aspect-ratio.tsrx",component)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("ratio={").unwrap();let fields=session.completion("main.tsrx",source,at).unwrap();for name in ["ratio","children","className","id"]{assert!(fields.contains(&name.into()),"{fields:?}");}
 let help=session.hover("main.tsrx",source,at+2).unwrap_or_default();assert!(help.contains("Required")&&help.contains("f32"),"{help}");
 let at=source.find("aspect-video").unwrap()+8;let values=session.completion("main.tsrx",source,at).unwrap();assert!(values.contains(&"aspect-video".into()),"{values:?}");assert!(session.hover("main.tsrx",source,at).unwrap_or_default().contains("width/height"));
 for broken in [source.replace("ratio={1.5}",""),source.replace("ratio={1.5}","ratio=\"wide\""),source.replace("aspect-video","aspect-[0]")] {
  session.change("main.tsrx",&broken);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(!diagnostics.as_array().unwrap().is_empty(),"{broken}: {diagnostics}");
 }
}

#[test]
fn consuming_throw_assertion_has_signature_and_ownership_help(){
 let source="import {throws} from 'std:testing';function main(){throws(()=>{throw 'expected';});}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.rfind("throws(").unwrap()+2;let help=session.hover("main.tsr",source,at).unwrap_or_default();assert!(help.contains("single-use")&&help.contains("AssertionError"),"{help}");
 let at=source.rfind("throws(").unwrap()+7;let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":at}}));assert!(signature.to_string().contains("Once<Function<void>>"),"{signature}");
}
