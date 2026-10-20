mod support;
#[test]
fn badge_local_props_and_shared_flow_utilities_have_help(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-compiler/src/ui-catalog/badge-1.0.0.json")).unwrap();let component=catalog[0]["files"][0]["content"].as_str().unwrap();
 let source="import {Badge} from './badge';import {View,Node,signal} from 'std:ui';function Page():Node{const active=signal(false);return <View><Badge variant={active.get()?'outline':null} className=\"whitespace-nowrap underline-offset-4 rounded-4xl [a]:hover:bg-primary/80\">Ready</Badge></View>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[("badge.tsrx",component)]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for (needle,expected) in [("variant=","Live six-way"),("whitespace-nowrap","collapses"),("underline-offset-4","below the baseline"),("rounded-4xl","1.25/1.5/2"),("[a]:hover:bg-primary/80","current Link")]{let at=source.find(needle).unwrap()+2;assert!(session.hover("main.tsrx",source,at).unwrap_or_default().contains(expected),"{needle}");}
 let at=source.find("whitespace-nowrap").unwrap()+"whitespace-no".len();assert!(session.completion("main.tsrx",source,at).unwrap().contains(&"whitespace-nowrap".into()));
}
