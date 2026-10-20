//! Hover, completion and the catalog must not contradict one another, and
//! the server must report no misleading answer while a document is typed.
mod support;
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Duration;
use support::session::Session;

const GREETING: &str = "export function greeting(): string { return \"hi\"; }\n";
const HEAD: &str = "import { greeting } from './greeting.tsr';\nimport { s } from 'std:schema';\n\ntype Item = { id: int; name: string; };\ntype AliasValues<T=int> = T[];\nconst schema = s.string();\n\nfunction main() {\n  const values: AliasValues = [1, 2, 3];\n  const item: Item = { id: 1, name: \"a\" };\n  const text = \"abc\";\n  const lookup = new Map<string, int>();\n  const seen = new Set<int>();\n  const parsed = schema.safeParse(greeting());\n  const loaded: Result<string, int> = Ok(greeting());\n  ";
const TAIL: &str = "\n}\n";

#[test]
fn nested_native_request_keeps_hover_and_completion() {
    let text=tsrust_compiler::format("import {Context} from 'std:http';function probe(ctx:Mutable<Context>){const tags=ctx.req.queries('tag');}function main(){}").unwrap();
    let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
    let offset=text.find("queries(").unwrap();
    let hover=session.hover("main.tsr",&text,offset).unwrap_or_default();
    assert!(hover.contains("string[]")&&hover.contains("Ownership:"),"{hover}");
    let labels=session.completion("main.tsr",&text,offset).unwrap();
    for member in ["query","queries","header","text"] {assert!(labels.iter().any(|label|label==member),"{labels:?}");}
    let counters=session.request("tsrust/counters",Value::Null);
    for name in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"] {assert_eq!(counters[name],0,"{counters}");}
}

/// A receiver whose type hover reports. `silent` is the reviewed reason why
/// completion offers nothing; hover must then say so too.
struct Receiver {
    name: &'static str,
    hover: &'static str,
    silent: Option<&'static str>,
}

const RECEIVERS: &[Receiver] = &[
    Receiver { name: "values", hover: "int[]", silent: None },
    Receiver { name: "item", hover: "Item", silent: None },
    Receiver { name: "text", hover: "string", silent: None },
    Receiver { name: "lookup", hover: "Map<string, int>", silent: None },
    Receiver { name: "seen", hover: "Set<int>", silent: None },
    Receiver { name: "schema", hover: "Schema<string>", silent: None },
    Receiver { name: "parsed", hover: "SafeParseSuccess<string> | SafeParseError<ValidationError>", silent: None },
    Receiver { name: "loaded", hover: "Result<string, int>", silent: None },
];

fn compact(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn completion_is_not_empty_where_hover_reports_a_type() {
    for receiver in RECEIVERS {
        let mut session = Session::start(&[("greeting.tsr", GREETING)]);
        let read = format!("{HEAD}console.log({});{TAIL}", receiver.name);
        session.open("main.tsr", &read);
        session.settle();
        let hover = session.hover("main.tsr", &read, HEAD.len() + "console.log(".len() + 1).unwrap_or_default();
        assert!(compact(&hover).contains(&compact(receiver.hover)), "{}: hover must report {}: {hover}", receiver.name, receiver.hover);
        let before = format!("{HEAD}{}.", receiver.name);
        let text = format!("{before}{TAIL}");
        session.change("main.tsr", &text);
        let labels = session.completion("main.tsr", &text, before.len()).unwrap();
        match receiver.silent {
            None => assert!(!labels.is_empty(), "{}: hover reports {} but completion offers nothing", receiver.name, receiver.hover),
            Some(reason) => assert!(labels.is_empty() && hover.contains(reason), "{}: a silent receiver must explain itself in hover ({reason}): {hover}", receiver.name),
        }
    }
}

#[test]
fn general_result_hover_and_completion_state_the_payload_field_and_the_match() {
    let mut session = Session::start(&[("greeting.tsr", GREETING)]);
    let text = format!("{HEAD}console.log(loaded.payload);{TAIL}");
    session.open("main.tsr", &text);
    session.settle();
    let value = session.hover("main.tsr", &text, HEAD.len() + "console.log(pa".len()).unwrap();
    assert!(value.contains("loaded.payload") && value.contains("match (loaded)"), "{value}");
    assert!(!value.contains("Match before accessing either payload"), "the hover must not deny the field: {value}");
    let field = session.hover("main.tsr", &text, HEAD.len() + "console.log(loaded.pay".len()).unwrap();
    let field = compact(&field);
    assert!(field.contains("payload:") && field.contains("string") && field.contains("int"), "{field}");
    let before = format!("{HEAD}loaded.");
    let typed = format!("{before}{TAIL}");
    session.change("main.tsr", &typed);
    let reply = session.completion_items("main.tsr", &typed, before.len());
    let items = if reply.is_array() { reply } else { reply["items"].clone() };
    let items = items.as_array().unwrap();
    let payload = items.iter().find(|item| item["label"] == "payload").expect("payload field");
    assert!(payload["detail"].as_str().unwrap().contains("int"));
    let narrowing = items.iter().find(|item| item["label"] == "match").expect("match skeleton");
    let edit = narrowing["textEdit"]["newText"].as_str().unwrap();
    assert!(edit.starts_with("match (loaded) {") && edit.contains("Ok(value)") && edit.contains("Err(error)"), "{edit}");
    // The skeleton replaces the whole access, from the receiver to the cursor.
    assert_eq!(narrowing["textEdit"]["range"]["start"]["character"], 2);
    assert_eq!(narrowing["filterText"], "loaded.match");
}

#[test]
fn every_documented_member_is_reachable_through_completion() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    // Catalog family and a receiver of that family.
    for (family, declaration) in [
        ("string", "const x = \"abc\";"),
        ("Array", "const x: int[] = [1, 2];"),
        ("int", "const x = 1;"),
        ("number", "const x = 1.5;"),
        ("boolean", "const x = true;"),
        ("Map", "const x = new Map<string, int>();"),
        ("Set", "const x = new Set<int>();"),
    ] {
        let documented = catalog[family].as_array().unwrap().iter().filter_map(|entry| entry["name"].as_str()).collect::<BTreeSet<_>>();
        let before = format!("function main() {{\n  {declaration}\n  x.");
        let text = format!("{before}\n}}\n");
        let mut session = Session::start(&[]);
        session.open("main.tsr", &text);
        session.settle();
        let offered = session.completion("main.tsr", &text, before.len()).unwrap().into_iter().collect::<BTreeSet<_>>();
        let unreachable = documented.iter().filter(|name| !offered.contains(**name)).collect::<Vec<_>>();
        assert!(unreachable.is_empty(), "{family}: documented but not offered on `{declaration}`: {unreachable:?}");
    }
}

#[test]
fn typing_a_project_document_produces_no_misleading_answer() {
    let mut session = Session::start(&[("greeting.tsr", GREETING)]);
    session.open("main.tsr", &format!("{HEAD}{TAIL}"));
    session.settle();
    let before = session.request("tsrust/counters", Value::Null);
    for typed in ["parsed.", "const next = greeting();\n  next.", "console.log(parsed.su", "values.ma", "match (loaded) { Ok(value) => { value.", "if (parsed.success) { parsed.", "item."] {
        session.type_text("main.tsr", HEAD, typed, TAIL, Duration::ZERO);
        let text = format!("{HEAD}{typed}{TAIL}");
        let labels = session.completion("main.tsr", &text, HEAD.len() + typed.len()).unwrap();
        assert!(!labels.is_empty(), "{typed}: no completion");
        assert!(!labels.iter().any(|label| label == "function" || label == "main"), "{typed}: scope names after a member separator");
    }
    let after = session.request("tsrust/counters", Value::Null);
    for counter in ["staleAnswers", "emptyKnownReceiver", "nonMemberAfterSeparator"] {
        assert_eq!(after[counter], before[counter], "{counter} must stay at its start value: {after}");
        assert_eq!(after[counter], 0, "{counter}: {after}");
    }
}
#[test]
fn every_async_lifecycle_member_is_reachable(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 for(family,before,after)in[
  ("TaskMethods","async function one():Task<int>{return 1;}async function main():Task<void>{const x=one();x.","timeout(10);}"),
  ("Job","async function one():Task<int>{return 1;}async function main():Task<void>{const x=one().start();x.","cancel();await x;}"),
  ("AsyncChannel","import {Channel} from 'std:futures';async function main():Task<void>{const x=new Channel<int>(1);x.","close();}"),
  ("AsyncSender","import {Channel} from 'std:futures';async function main():Task<void>{const queue=new Channel<int>(1);const x=queue.sender();await x.","send(1);}"),
 ]{
  let text=format!("{before}{after}");let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
  let offered=session.completion("main.tsr",&text,before.len()).unwrap();
  for entry in catalog[family].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(offered.iter().any(|label|label==name),"{family}.{name}: {offered:?}");}
 }
}
#[test]
fn every_http_member_is_reachable(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 for(family,ty,member)in[("HttpApp","App","get"),("HttpClient","Client","send"),("HttpRequest","Request","method"),("HttpResponse","Response","status"),("HttpBody","Body","next"),("HttpNext","Next","run"),("HttpServer","Server","address")]{
  let before=format!("import {{{ty}}} from 'std:http';function inspect(x:{ty}):void{{x.");let text=format!("{before}{member};}}function main(){{}}");let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
  let offered=session.completion("main.tsr",&text,before.len()).unwrap();
  for entry in catalog[family].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(offered.iter().any(|label|label==name),"{family}.{name}: {offered:?}");}
 }
}

#[test]
fn every_sqlite_member_is_reachable(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 for(family,ty,member)in[("SqliteConnection","Connection","prepare"),("SqliteStatement","Statement","execute"),("SqliteTransaction","Transaction","commit"),("SqliteRow","Row","text")]{
  let before=format!("import {{{ty}}} from 'std:sqlite';function inspect(x:{ty}):void{{x.");let text=format!("{before}{member};}}function main(){{}}");let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
  let offered=session.completion("main.tsr",&text,before.len()).unwrap();
  for entry in catalog[family].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(offered.iter().any(|label|label==name),"{family}.{name}: {offered:?}");}
 }
}

#[test]
fn tracked_resource_module_help_agrees_with_checked_ownership() {
    let text="import {trackedResource,resourceId,closeResource,releaseCount} from 'std:testing';function main(){const r=trackedResource();const key=resourceId(r);closeResource(r);releaseCount(key);}";
    let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
    for (call,expected) in [("trackedResource();","synthetic"),("resourceId(r)","identity"),("closeResource(r)","Consumes"),("releaseCount(key)","destructions")] {
        let offset=text.rfind(call).unwrap();
        let hover=session.hover("main.tsr",text,offset+1).unwrap_or_default();
        assert!(hover.contains(expected),"{call}: {hover}");
        let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));
        assert!(!signature["signatures"].as_array().unwrap().is_empty(),"{signature}");
    }
}

#[test]
fn numeric_module_help_preserves_exact_contract(){
 let text="import {toInt,toNumber,toFloat32,checkSigned,checkUnsigned,parseInt as exactInt} from 'std:numeric';function main(){toInt(1);toNumber(1);toFloat32(1);checkSigned(1,8);checkUnsigned(1,8);exactInt('1');}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 for (call,expected) in [("toInt(1)","64 bits"),("toNumber(1)","exactly"),("toFloat32(1)","binary32"),("checkSigned(1,8)","Width"),("checkUnsigned(1,8)","unsupported"),("exactInt('1')","entire string")]{
 let offset=text.rfind(call).unwrap();let hover=session.hover("main.tsr",text,offset+1).unwrap_or_default();assert!(hover.contains(expected),"{call}: {hover}");
 let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));assert!(!signature["signatures"].as_array().unwrap().is_empty(),"{signature}");
 }
}

#[test]
fn sql_parameter_help_explains_precision_and_driver_limits(){
 let text="import {connect} from 'std:sql';async function main():Task<void>{const db=await connect('sqlite::memory:',1);await db.query('SELECT ?',[1.25]);await db.close();}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 let offset=text.find("[1.25]").unwrap();
 let signature=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset}}));
 let help=signature.to_string();assert!(help.contains("precision loss")&&help.contains("Server-side implicit casts can still round"),"{signature}");
}

#[test]
fn data_error_property_examples_reach_live_hover(){
 for (module,ty) in [("std:sql","SqlError"),("std:sqlite","SqliteError")]{
  let text=format!("import {{{ty}}} from '{module}';function inspect(error:ReadOnly<{ty}>):void{{console.log(error.code);console.log(error.message);}}function main(){{}}");
  let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
  for member in ["code","message"]{let offset=text.rfind(&format!("error.{member}")).unwrap()+7;let hover=session.hover("main.tsr",&text,offset).unwrap_or_default();assert!(hover.contains("Missing column")&&hover.contains("Ownership"),"{ty}.{member}: {hover}");}
 }
}

#[test]
fn decimal_help_describes_exact_arithmetic(){
 let text="import {parseDecimal,divideExact,formatFixed,toNumber,fromInt} from 'std:decimal';function main(){const value=unwrap(parseDecimal('1'));divideExact(value,fromInt(3));formatFixed(value,2);toNumber(value);}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 for (call,expected) in [("parseDecimal('1')","38 significant"),("divideExact(value","DivideByZero"),("formatFixed(value","Never rounds"),("toNumber(value","binary64")]{
 let offset=text.rfind(call).unwrap();assert!(session.hover("main.tsr",text,offset+1).unwrap_or_default().contains(expected),"{call}");
 let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{help}");
 }
}

#[test]
fn calendar_help_describes_date_fns_style_arithmetic(){
 let text="import {parseDate,addMonths,format,getMonth} from 'std:date';function main(){const date=unwrap(parseDate('2024-01-31'));addMonths(date,1);format(date,'yyyy-MM-dd');getMonth(date);}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 for (call,expected) in [("parseDate('2024","0001..9999"),("addMonths(date","clamps"),("format(date","yyyy"),("getMonth(date","zero-based")]{
 let offset=text.rfind(call).unwrap();assert!(session.hover("main.tsr",text,offset+1).unwrap_or_default().contains(expected),"{call}");
 let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{help}");
 }
}

#[test]
fn temporal_help_explains_dst_and_exact_units(){
 let text="import {parseISO,toUnixMilliseconds,resolveDateTime,parseDate,parseTime,parseTimeZone,durationFromMilliseconds} from 'std:date';function main(){const value=unwrap(parseISO('2024-01-01T00:00:00Z'));toUnixMilliseconds(value);durationFromMilliseconds(1);resolveDateTime(unwrap(parseDate('2024-11-03')),unwrap(parseTime('01:30:00')),unwrap(parseTimeZone('America/New_York')),'reject');}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 for (call,expected) in [("parseISO('2024","mandatory"),("toUnixMilliseconds(value","Precision"),("durationFromMilliseconds(1)","nanoseconds"),("resolveDateTime(unwrap","NonexistentTime")]{
 let offset=text.rfind(call).unwrap();assert!(session.hover("main.tsr",text,offset+1).unwrap_or_default().contains(expected),"{call}");
 let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{help}");
 }
}

#[test]
fn uuid_help_explains_exact_bytes_and_entropy_errors(){
 let text="import {parseUuid,toBytes,fromBytes,randomUuid} from 'std:uuid';function main(){const id=unwrap(parseUuid('67e55044-10b1-426f-9247-bb680e5fe0c8'));const bytes=toBytes(id);fromBytes(bytes);randomUuid();}";
 let mut session=Session::start(&[]);session.open("main.tsr",text);session.settle();
 for (call,expected) in [("parseUuid('67","36 ASCII"),("toBytes(id)","16 UUID bytes"),("fromBytes(bytes)","Length"),("randomUuid()","Entropy")]{
 let offset=text.rfind(call).unwrap();assert!(session.hover("main.tsr",text,offset+1).unwrap_or_default().contains(expected),"{call}");
 let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset+call.find('(').unwrap()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{help}");
 }
}

#[test]
fn sql_cursor_members_have_live_hover_and_completion(){
    let source="import {Cursor} from 'std:sql';function inspect(cursor:Mutable<Cursor>){cursor.next();}function main(){}";
    let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();
    let offset=source.find("cursor.next").unwrap()+"cursor.".len();
    let members=session.completion("main.tsr",source,offset).unwrap();
    for member in ["next","close"]{assert!(members.iter().any(|value|value==member),"{members:?}");}
    let hover=session.hover("main.tsr",source,offset+1).unwrap_or_default();
    assert!(hover.contains("Task<Row | undefined>")&&hover.contains("exclusively borrows"),"{hover}");
}

#[test]
fn ui_state_members_are_reachable_without_lifetime_helpers() {
    let before="import {signal} from 'std:ui';function main(){const count=signal(0);count.";
    let text=format!("{before}set(count.get()+1);}}");
    let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
    let offered=session.completion("main.tsr",&text,before.len()).unwrap();
    assert_eq!(offered.into_iter().collect::<BTreeSet<_>>(),BTreeSet::from(["get".into(),"set".into()]));
    for (needle,expected) in [("set(count","Replaces"),("get()","Reads")] {
        assert!(session.hover("main.tsr",&text,text.find(needle).unwrap()).unwrap().contains(expected));
    }
    let counters=session.request("tsrust/counters",Value::Null);
    for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"] {assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn ui_condition_props_and_class_help_match_checked_source() {
 let source="import {Button,Node,signal} from 'std:ui';function Page():Node{const count=signal(0);return <Button disabled={count.get() < 5} onPress={()=>{count.set(count.get()+100);}} className=\"rounded\">Disabled</Button>;}function main(){const node=Page();}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);session.settle();
 let at=source.find("disabled=").unwrap();let fields=session.completion("main.tsrx",source,at).unwrap();assert!(fields.contains(&"disabled".into()),"{fields:?}");
 assert!(session.hover("main.tsrx",source,at+2).unwrap().contains("ReactiveBoolean"));
 let class=source.find("rounded").unwrap()+3;assert!(session.hover("main.tsrx",source,class).unwrap().contains("Shared web/native"));
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn ui_semantic_attributes_keep_exact_names_and_checked_help() {
 let source="import {View,Text,Node} from 'std:ui';function Page():Node{return <View as=\"section\" role=\"region\" aria-label=\"Account\" data-variant=\"summary\"><Text as=\"h2\">Profile</Text></View>;}function main(){const node=Page();}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);session.settle();
 let variant=source.find("data-variant=").unwrap();assert!(session.hover("main.tsrx",source,variant+3).unwrap().contains("variant"));
 let at=source.find("aria-label=").unwrap();assert!(session.hover("main.tsrx",source,at+6).unwrap().contains("string"));
 let labels=session.completion("main.tsrx",source,at+5).unwrap();assert!(labels.contains(&"aria-label".into()),"{labels:?}");
 let uri=session.uri("main.tsrx");let location=session.request("textDocument/definition",serde_json::json!({"textDocument":{"uri":uri},"position":{"line":0,"character":at+6}}));assert!(!location.is_null());
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn editable_control_events_and_exact_props_keep_help_and_zero_fallbacks() {
 let source="import {Input,Node,signal} from 'std:ui';function Page():Node{const name=signal('Ada');return <Input aria-label=\"Name\" aria-describedby=\"hint\" value={name.get()} readOnly={false} onChange={(event)=>{name.set(event.target.value);}}/>;}function main(){const node=Page();}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);session.settle();
 for (needle,wanted) in [("aria-describedby=","description"),("readOnly=","selection"),("onChange=","snapshot"),("target.value","InputTarget")] {
  let at=source.find(needle).unwrap();let hover=session.hover("main.tsrx",source,at+2).unwrap_or_default();assert!(hover.to_lowercase().contains(&wanted.to_lowercase()),"{needle}: {hover}");
 }
 let at=source.find("target.value").unwrap()+"target.".len();let members=session.completion("main.tsrx",source,at).unwrap();for member in ["value","name","id"]{assert!(members.contains(&member.to_string()),"{members:?}");}
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn vector_public_exports_are_reachable_without_misleading_answers() {
 let source="import { Svg, Path, SvgProps, PathProps } from 'std:ui';function main(){}";
 let mut session=Session::start(&[]);let imports="import { } from 'std:ui';function main(){}";session.open("main.tsrx",imports);session.settle();
 let offered=session.completion("main.tsrx",imports,imports.find('}').unwrap()).unwrap();
 for name in ["Svg","Path","SvgProps","PathProps"]{assert!(offered.contains(&name.into()),"{name}: {offered:?}");}
 session.change("main.tsrx",source);session.settle();
 for name in ["Svg","Path","SvgProps","PathProps"]{assert!(!session.hover("main.tsrx",source,source.find(name).unwrap()+1).unwrap_or_default().is_empty(),"{name}");}
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn lazy_child_factories_have_invocation_help_without_resource_methods(){
 let source="import {View,Node,LazyChildren} from 'std:ui';interface Props{children:LazyChildren;}function Panel(props:Props):Node{const render=props.children;return <View children={render()}/>;}function main(){const node=<Panel><View/></Panel>;}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);session.settle();let at=source.find("render()").unwrap()+2;let hover=session.hover("main.tsrx",source,at).unwrap_or_default();assert!(hover.contains("Child")||hover.contains("Node"),"{hover}");
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn reactive_metadata_help_matches_completion_and_diagnostics(){
 let source="import {Input,signal} from 'std:ui';function main(){const changed=signal(false);const node=<Input aria-invalid={changed.get()?true:undefined}/>;}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();let at=source.find("aria-invalid=").unwrap();let hover=session.hover("main.tsrx",source,at+3).unwrap_or_default();assert!(hover.contains("live")&&hover.contains("undefined"),"{hover}");let offered=session.completion("main.tsrx",source,at+3).unwrap();assert!(offered.contains(&"aria-invalid".into()),"{offered:?}");
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
