mod support;

#[test]
fn reactive_semantic_props_keep_exact_hover_and_current_diagnostics(){
 let source="import {Button,Input,View,Text,Node,signal} from 'std:ui';function Page():Node{const changed=signal(false);return <View><Input aria-label={changed.get()?'Changed':'Draft'} aria-invalid={changed.get()?true:undefined} aria-describedby={changed.get()?'error':'hint'}/><Button onPress={()=>{changed.set(!changed.get());}}>Change</Button><Text aria-hidden={changed.get()}>Decoration</Text></View>;}function main(){const node=Page();}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for field in ["aria-label","aria-invalid","aria-describedby","aria-hidden"]{let at=source.find(&format!("{field}=")).unwrap()+2;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("live")&&hover.contains("Ownership:"),"{field}: {hover}");}
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
