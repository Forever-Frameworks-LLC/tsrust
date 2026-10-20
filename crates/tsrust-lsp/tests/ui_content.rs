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

#[test]
fn current_keycaps_keep_reactive_prop_help_and_reject_unknown_props(){
 let source="import {Kbd,KbdGroup,KbdProps} from './kbd';import {Node,signal} from 'std:ui';function read(props:ReadOnly<KbdProps>){const classes=props.className;}function Page():Node{const active=signal(false);return <KbdGroup className={active.get()?'gap-2':'gap-1'}><Kbd role='group' aria-label='Save' className={active.get()?'bg-primary':'bg-muted'}>S</Kbd></KbdGroup>;}";
 let mut session=support::session::Session::start(&[("kbd.tsrx",include_str!("../../../docs/src/components/ui/kbd-v2.tsrx"))]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("props.className").unwrap()+"props.".len()+1;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("string"),"{hover}");
 let at=source.find("props.className").unwrap()+"props.".len();let completion=session.completion("main.tsrx",source,at).unwrap();for name in ["className","children","aria-label","id"]{assert!(completion.iter().any(|value|value==name),"missing {name}: {completion:?}");}
 let bad=source.replace("<Kbd role", "<Kbd invalidProp={1} role");session.change("main.tsrx",&bad);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.to_string().contains("invalidProp"),"{diagnostics}");
 let counters=session.request("tsrust/counters",serde_json::json!({}));for name in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[name],0,"{counters}");}
}

#[test]
fn table_props_keep_hover_completion_and_reject_unimplemented_row_spans(){
 let source="import {Table,TableRow,TableCell,TableHead,TableProps} from './table';import {Node,signal} from 'std:ui';function read(props:ReadOnly<TableProps>){const span=props.colSpan;}function Page():Node{const selected=signal(false);return <Table><TableRow data-state={selected.get()?'selected':undefined}><TableHead scope='row'>Total</TableHead><TableCell colSpan={2}>Value</TableCell></TableRow></Table>;}";
 let mut session=support::session::Session::start(&[("table.tsrx",include_str!("../../../docs/src/components/ui/table.tsrx"))]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let at=source.find("props.colSpan").unwrap()+"props.".len()+1;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("1000"),"{hover}");
 let completion=session.completion("main.tsrx",source,at-1).unwrap();for name in ["colSpan","scope","data-state","className"]{assert!(completion.iter().any(|value|value==name),"{completion:?}");}
 let bad=source.replace("colSpan={2}","rowSpan={2}");session.change("main.tsrx",&bad);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.to_string().contains("rowSpan"),"{diagnostics}");
 let counters=session.request("tsrust/counters",serde_json::json!({}));for name in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[name],0,"{counters}");}
}

#[test]
fn inherited_interface_docs_arrive_after_imports_without_replacing_own_docs(){
 let source="import {Leaf,Override,Middle} from './leaf';function inspect(base:ReadOnly<Leaf>,own:ReadOnly<Override>,generic:ReadOnly<Middle<string>>){const first=base.value;const second=own.value;const third=generic.value;}";
 let mut session=support::session::Session::start(&[("base.tsr","export interface Base<T>{/** Base value contract, preserved across imports. */ value:T;}"),("leaf.tsr","import {Base} from './base';export interface Middle<T> extends Base<T>{}export interface Leaf extends Middle<string>{}export interface Override extends Middle<string>{/** Local value contract. */value:string;}")]);
 session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 for (needle,expected,excluded) in [("base.value","Base value contract","Local value contract"),("own.value","Local value contract","Base value contract"),("generic.value","Base value contract","Local value contract")]{let at=source.find(needle).unwrap()+needle.find('.').unwrap()+2;let hover=session.hover("main.tsr",source,at).unwrap_or_default();assert!(hover.contains(expected)&&!hover.contains(excluded),"{hover}");let items=session.completion_items("main.tsr",source,at-1);assert!(items.to_string().contains(expected),"{items}");}
 let counters=session.request("tsrust/counters",serde_json::json!({}));for name in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[name],0,"{counters}");}
}
