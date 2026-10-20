mod support;
#[test]
fn vector_props_paint_literals_and_live_paths_have_checked_editor_help(){
 let source="import {Svg,Path,Node,signal} from 'std:ui';function Page():Node{const visible=signal(false);return <Svg aria-label=\"Chart\" strokeLinecap=\"round\" viewBox=\"0 0 24 24\"><Path d={visible.get()?'M0 0L24 24':''} strokeWidth={2}/></Svg>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for (needle,expected) in [("<Svg aria","viewport"),("<Path d","owned"),("strokeLinecap=","End-cap"),("viewBox=","Four finite"),("strokeWidth=","nonnegative")]{let at=source.find(needle).unwrap()+2;assert!(session.hover("main.tsrx",source,at).unwrap_or_default().contains(expected),"{needle}");}
 let at=source.find("round").unwrap()+2;assert!(session.completion("main.tsrx",source,at).unwrap().contains(&"round".into()));
 let at=source.find("strokeWidth=").unwrap();let props=session.completion("main.tsrx",source,at).unwrap();assert!(props.contains(&"strokeLinecap".into())&&props.contains(&"strokeWidth".into())&&!props.contains(&"children".into())&&!props.contains(&"d".into()),"{props:?}");
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
