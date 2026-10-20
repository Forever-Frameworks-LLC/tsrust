mod support;

#[test]
fn disclosure_props_and_instance_ids_have_current_help_and_diagnostics(){
 let source="import {Button,View,Node,signal,useId} from 'std:ui';function Page():Node{const open=signal(false);const id=useId();const controls=clone(id);return <View><Button aria-expanded={open.get()} aria-controls={open.get()?clone(controls):undefined} onPress={()=>{open.set(!open.get());}}>Details</Button><View id={id} hidden={!open.get()}/></View>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for field in ["aria-expanded","aria-controls"]{let at=source.find(&format!("{field}=")).unwrap()+2;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("live")&&hover.contains("Ownership:"),"{field}: {hover}");}
 let at=source.find("useId()").unwrap()+2;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("runtime-local")&&hover.contains("no arguments"),"{hover}");
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
