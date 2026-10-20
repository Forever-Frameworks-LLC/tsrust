mod support;

#[test]
fn computed_receiver_is_readonly_and_preserves_payload_help() {
 for (payload,value) in [("int","41"),("string","'Ready'"),("boolean","true"),("'default' | 'destructive'","'default'"),("int | undefined","undefined")] {
  let source=format!("import {{computed}} from 'std:ui';function main(){{const value=computed<{payload}>(()=>{value});value.get();}}");
  let mut session=support::session::Session::start(&[]);session.open("main.tsr",&source);let diagnostics=session.wait_for_diagnostics("main.tsr");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
  let at=source.find("value.get").unwrap()+6;assert_eq!(session.completion("main.tsr",&source,at).unwrap(),vec!["get"]);
  let hover=session.hover("main.tsr",&source,at).unwrap_or_default();assert!(hover.contains("get():")&&hover.contains("Evaluates")&&hover.contains("Ownership:"),"{hover}");
  if !payload.contains('|'){assert!(hover.contains(&format!("get(): {payload}")),"{hover}");}
  let call=source.find("value.get(").unwrap()+10;let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":call}}));assert!(signature.to_string().contains("get()"),"{signature}");
  for part in payload.split(" | "){assert!(hover.contains(part.trim_matches('\'')),"{hover}");}
 }
}

#[test]
fn reactive_prop_inheritance_and_replacement_parameters_have_editor_types() {
 let source="import {View,ViewProps,ReactiveValue,useProp,Node,Button} from 'std:ui';interface Props extends ViewProps<ReactiveValue<string>,ReactiveValue<string>>{tone?:ReactiveValue<'default'|'destructive'>;}function Notice(props:Props):Node{const tone=useProp<'default'|'destructive'>(props.tone??'default');props.tone=undefined;return (<View {...props} data-variant={tone.get()}/>);}function main(){const node=<Button render={(props)=><Button {...props}/>} onPress={()=>{console.log('Pressed');}}>Action</Button>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");session.settle();assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 let hover=session.hover("main.tsrx",source,source.find("useProp<'default'").unwrap()+2).unwrap_or_default();assert!(hover.contains("Adapts")&&hover.contains("ReactiveValue"),"{hover}");
 let hover=session.hover("main.tsrx",source,source.find("render=").unwrap()+2).unwrap_or_default();assert!(hover.contains("replacement")&&hover.contains("once"),"{hover}");
}
