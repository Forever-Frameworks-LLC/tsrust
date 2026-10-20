mod support;
#[test]
fn lazy_children_keep_owned_callback_help_and_current_diagnostics(){
 let source="import {View,Text,Node,LazyChildren,signal} from 'std:ui';interface Base<T>{/** Owned child recipe. */children?:T;}interface Props extends Base<LazyChildren>{id?:string;}function Panel(props:Props):Node{const render=props.children;return <View id={props.id} children={render===undefined?[]:render()}/>;}function Page():Node{const count=signal(0);return <Panel id=\"panel\"><Text>{count.get()}</Text></Panel>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 let alias=source.find("LazyChildren").unwrap()+3;assert!(session.hover("main.tsrx",source,alias).unwrap_or_default().contains("deferred"));
 let member=source.find("props.children").unwrap()+"props.".len();let offered=session.completion("main.tsrx",source,member).unwrap();assert!(offered.contains(&"children".into())&&offered.contains(&"id".into()),"{offered:?}");assert!(session.hover("main.tsrx",source,member+2).unwrap_or_default().contains("recipe"));
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
