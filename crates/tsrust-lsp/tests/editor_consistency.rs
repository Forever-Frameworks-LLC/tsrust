//! Hover, completion and the catalog must not contradict one another, and
//! the server must report no misleading answer while a document is typed.
mod support;
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Duration;
use support::session::Session;

const GREETING: &str = "export function greeting(): string { return \"hi\"; }\n";
const HEAD: &str = "import { greeting } from './greeting.tsr';\nimport { s } from 'std:schema';\n\ntype Item = { id: int; name: string; };\ntype AliasValues<T=int> = T[];\nconst schema = s.string();\n\nfunction main() {\n  const values: AliasValues = [1, 2, 3];\n  const units: void[] = [];\n  const item: Item = { id: 1, name: \"a\" };\n  const text = \"abc\";\n  const lookup = new Map<string, int>();\n  const seen = new Set<int>();\n  const parsed = schema.safeParse(greeting());\n  const loaded: Result<string, int> = Ok(greeting());\n  ";
const TAIL: &str = "\n}\n";

#[test]
fn path_environment_os_help_agrees_across_all_server_phases() {
    let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    for module in ["std:path","std:env","std:os"] {
        let before="import { ";let source=format!("{before} }} from '{module}';function main(){{}}");
        let mut session=Session::start(&[]);session.open("main.tsr",&source);session.settle();
        let names=session.completion("main.tsr",&source,before.len()).unwrap();
        assert!(!names.is_empty(),"{module}");
        for row in catalog[module].as_array().unwrap(){let name=row["name"].as_str().unwrap();assert!(names.iter().any(|item|item==name),"{module}.{name}: {names:?}");}
    }
    let mut cases=Vec::new();
    for (module,name,arguments,result) in [
        ("std:path","parse","'file.txt'","PathParts"),("std:path","format","{name:'file',ext:'txt'}","ReadOnly<PathFormatOptions>"),
        ("std:path","resolve","'file.txt'","PathError"),("std:path","separator","","string"),("std:path","delimiter","","string"),
        ("std:env","vars","","EnvVar"),("std:env","currentDir","","EnvError"),("std:env","currentExe","","EnvError"),("std:env","tempDir","","EnvError"),("std:env","homeDir","","undefined"),
        ("std:os","availableParallelism","","OsError"),("std:os","hostname","","OsError"),
    ] {cases.push((format!("import {{{name}}} from '{module}';function main(){{"),name,format!("({arguments});}}"),result,true));}
    for (module,ty,name,result) in [("std:path","PathParts","ext","string"),("std:path","PathFormatOptions","name","undefined"),("std:path","PathError","code","string"),("std:env","EnvVar","value","string"),("std:os","OsError","message","string")] {
        cases.push((format!("import {{{ty}}} from '{module}';function inspect(value:ReadOnly<{ty}>){{console.log(value."),name,");}function main(){}".into(),result,false));
    }
    for (before,typed,after,expected,callable) in cases {
        let mut baseline=None;let mut signature_baseline:Option<Value>=None;
        for (mode,delay,broken) in [("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)] {
            let after=if broken{format!("{after}\nfunction broken(){{const = ;}}") }else{after.clone()};let source=format!("{before}{typed}{after}");
            let mut session=Session::start(&[]);
            if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",&before,typed,&after,delay);}else{session.open("main.tsr",&source);session.settle();}
            let response=session.completion_items("main.tsr",&source,before.len()+typed.len());let items=if response.is_array(){response}else{response["items"].clone()};
            let items=items.as_array().unwrap();assert!(!items.is_empty(),"{typed} [{mode}]");
            let item=items.iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("{typed} [{mode}]: {items:?}"));
            let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&source,before.len()+1).unwrap_or_default();let text=format!("{detail}\n{hover}");
            assert!(text.contains(expected)&&text.contains("Ownership:"),"{typed} [{mode}]: {text}");
            if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{typed} [{mode}]");}else{baseline=Some((detail.clone(),hover.clone()));}
            if callable {
                let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));
                assert!(help.to_string().contains(expected),"{typed} [{mode}]: {help}");
                if let Some(previous)=&signature_baseline{assert_eq!(&help,previous,"{typed} [{mode}] signature");}else{signature_baseline=Some(help);}
            }
            let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{typed} [{mode}]: {counters}");}
        }
    }
}

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
    Receiver { name: "units", hover: "void[]", silent: None },
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
  ("Interval","import {Interval} from 'std:futures';async function main():Task<void>{const x=new Interval(1);await x.","tick();}"),
  ("CancellationToken","import {CancellationToken} from 'std:futures';async function main():Task<void>{const x=new CancellationToken();x.","cancel();}"),
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
 for(family,ty,member)in[("HttpRouter","Router","get"),("HttpApp","App","get"),("HttpClient","Client","send"),("HttpRequest","Request","method"),("HttpResponse","Response","status"),("HttpBody","Body","next"),("HttpNext","Next","run"),("HttpServer","Server","address")]{
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
  for member in ["code","message"]{let offset=text.rfind(&format!("error.{member}")).unwrap()+7;let hover=session.hover("main.tsr",&text,offset).unwrap_or_default();assert!(hover.contains("```tsrust")&&hover.contains(&format!("error.{member}"))&&hover.contains("Ownership"),"{ty}.{member}: {hover}");}
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

#[test]
fn disclosure_help_matches_completion_and_diagnostics(){
 let source="import {Button,signal} from 'std:ui';function main(){const open=signal(false);const node=<Button aria-expanded={open.get()}/>;}";
 let mut session=Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();let at=source.find("aria-expanded=").unwrap();let hover=session.hover("main.tsrx",source,at+3).unwrap_or_default();assert!(hover.contains("live")&&hover.contains("undefined"),"{hover}");let offered=session.completion("main.tsrx",source,at+3).unwrap();for field in ["aria-expanded","aria-controls"]{assert!(offered.contains(&field.into()),"{offered:?}");}
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn empty_heading_field_help_matches_completion_and_diagnostics(){
 let source="import {ThemeTypography} from 'std:ui';function main(){const typography:ThemeTypography={base:16,xl:24,headingFamily:'mono'};console.log(typography.headingFamily??'sans');}";
 let mut session=Session::start(&[]);session.open("main.tsr",source);let diagnostics=session.wait_for_diagnostics("main.tsr");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();let at=source.rfind("headingFamily").unwrap()+2;let hover=session.hover("main.tsr",source,at).unwrap_or_default();assert!(hover.contains("sans")&&hover.contains("Ownership:"),"{hover}");let labels=session.completion("main.tsr",source,at).unwrap();assert!(labels.contains(&"headingFamily".into()),"{labels:?}");let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn url_exports_and_documented_members_are_reachable() {
    let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let before="import { ";let text=format!("{before} }} from 'std:url';function main(){{}}");
    let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
    let imports=session.completion("main.tsr",&text,before.len()).unwrap();
    for entry in catalog["std:url"].as_array().unwrap() {
        let name=entry["name"].as_str().unwrap();assert!(imports.iter().any(|label|label==name),"std:url.{name}: {imports:?}");
    }
    for hidden in ["__href","__setHref","__get","url_intrinsic"]{assert!(!imports.iter().any(|label|label==hidden),"{imports:?}");}
    for (family,ty,member) in [("Url","Url","href"),("SearchParams","SearchParams","get('x')")] {
        let before=format!("import {{{ty}}} from 'std:url';function inspect(value:Mutable<{ty}>){{value.");
        let text=format!("{before}{member};}}function main(){{}}");
        session.change("main.tsr",&text);session.settle();
        let offered=session.completion("main.tsr",&text,before.len()).unwrap();
        let expected=catalog[family].as_array().unwrap().iter().map(|entry|entry["name"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
        assert_eq!(offered.into_iter().collect::<BTreeSet<_>>(),expected,"{family}");
    }
    let counters=session.request("tsrust/counters",Value::Null);
    for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn url_read_write_help_agrees_at_typing_speed_and_during_syntax_recovery() {
    let cases=[
        ("read pathname","import {Url} from 'std:url';function inspect(value:ReadOnly<Url>){console.log(value.","pathname",
            ");}function main(){}",&["StringView","assign string","throws UrlError","Ownership:"][..]),
        ("write pathname","import {Url} from 'std:url';function inspect(value:Mutable<Url>){value.","pathname",
            "='/next';}function main(){}",&["StringView","assign string","throws UrlError","Ownership:"][..]),
        ("write optional port","import {Url} from 'std:url';function inspect(value:Mutable<Url>){value.","port",
            "=8443;}function main(){}",&["int","undefined","assign (int | undefined)","throws UrlError","Ownership:"][..]),
        ("write optional hostname","import {Url} from 'std:url';function inspect(value:Mutable<Url>){value.","hostname",
            "='other.example';}function main(){}",&["StringView","undefined","assign (string | undefined)","throws UrlError","Ownership:"][..]),
        ("readonly origin","import {Url} from 'std:url';function inspect(value:ReadOnly<Url>){console.log(value.","origin",
            ");}function main(){}",&["string","read-only","Ownership:"][..]),
        ("query borrowed get","import {SearchParams} from 'std:url';function inspect(params:SearchParams){params.","get",
            "('x');}function main(){}",&["StringView","undefined","Ownership:"][..]),
    ];
    for (name,before,typed,after,expected) in cases {
        let mut baseline=None;
        for (mode,delay,broken) in [("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)] {
            let after=if broken{format!("{after}\nfunction broken(){{const = ;}}") }else{after.to_owned()};
            let text=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
            if let Some(delay)=delay {
                session.open("main.tsr",&format!("{before}{after}"));session.settle();
                session.type_text("main.tsr",before,typed,&after,delay);
            }else{session.open("main.tsr",&text);session.settle();}
            let reply=session.completion_items("main.tsr",&text,before.len()+typed.len());
            let items=if reply.is_array(){reply}else{reply["items"].clone()};
            let item=items.as_array().unwrap().iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("{name} [{mode}]: {items}"));
            let detail=item["detail"].as_str().unwrap().to_owned();
            let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();
            let combined=format!("{detail}\n{hover}");
            for fragment in expected{assert!(combined.contains(fragment),"{name} [{mode}]: missing {fragment}: {combined}");}
            for hidden in ["ResultUnwrap","__set","__tsrust_private_"]{assert!(!combined.contains(hidden),"{name} [{mode}]: {combined}");}
            if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{name} [{mode}] disagrees with settled help");}
            else{baseline=Some((detail,hover));}
            if typed=="get" {
                let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));
                let signature=help.to_string();assert!(signature.contains("StringView")&&signature.contains("undefined"),"{name} [{mode}]: {help}");
            }
            let counters=session.request("tsrust/counters",Value::Null);
            for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{name} [{mode}]: {counters}");}
        }
    }
}

#[test]
fn filesystem_exports_and_documented_record_help_agree_across_server_phases() {
    let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let before="import { ";let text=format!("{before} }} from 'std:fs';function main(){{}}");
    let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
    let imports=session.completion("main.tsr",&text,before.len()).unwrap();
    for entry in catalog["std:fs"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(imports.iter().any(|label|label==name),"std:fs.{name}: {imports:?}");}
    for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(session.request("tsrust/counters",Value::Null)[key],0);}
    let cases=[
        ("seek callable","import {File,seek} from 'std:fs';function inspect(file:Mutable<File>){","seek",
            "(file,0,'start');}function main(){}",&["Mutable<File>","int","FsError","Ownership:"][..]),
        ("position callable","import {File,position} from 'std:fs';function inspect(file:Mutable<File>){","position",
            "(file);}function main(){}",&["Mutable<File>","int","FsError","Ownership:"][..]),
        ("cursor alias","import { ","SeekOrigin",
            " } from 'std:fs';function inspect(origin:SeekOrigin){}function main(){}",&["SeekOrigin","start","current","end","Ownership:"][..]),
        ("file option","import {OpenOptions} from 'std:fs';function inspect(value:ReadOnly<OpenOptions>){console.log(value.","read",
            ");}function main(){}",&["boolean","undefined","defaults to false","Ownership:"][..]),
        ("exact timestamp","import {FileTime} from 'std:fs';function inspect(value:ReadOnly<FileTime>){console.log(value.","unixSeconds",
            ");}function main(){}",&["int","rounded down","Ownership:"][..]),
        ("optional metadata timestamp","import {Metadata} from 'std:fs';function inspect(value:ReadOnly<Metadata>){console.log(value.","created",
            ");}function main(){}",&["FileTime","undefined","not the Unix metadata-change time","Ownership:"][..]),
        ("directory path","import {DirEntry} from 'std:fs';function inspect(value:ReadOnly<DirEntry>){console.log(value.","path",
            ");}function main(){}",&["string","Relative input produces a relative path","Ownership:"][..]),
    ];
    for(name,before,typed,after,expected)in cases{
        let mut baseline=None;
        let mut help_baseline:Option<Value>=None;
        for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
            let after=if broken{format!("{after}\nfunction broken(){{const = ;}}") }else{after.to_owned()};
            let text=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
            if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&text);session.settle();}
            let reply=session.completion_items("main.tsr",&text,before.len()+typed.len());let items=if reply.is_array(){reply}else{reply["items"].clone()};
            let item=items.as_array().unwrap().iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("{name} [{mode}]: {items}"));
            let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();let combined=format!("{detail}\n{hover}");
            for fragment in expected{assert!(combined.contains(fragment),"{name} [{mode}]: missing {fragment}: {combined}");}
            assert!(!combined.contains("__tsrust_private_"),"{name} [{mode}]: {combined}");
            if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{name} [{mode}] disagrees with settled help");}else{baseline=Some((detail,hover));}
            if matches!(typed,"seek"|"position") {
                let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));
                let text=help.to_string();assert!(text.contains("Mutable<File>")&&text.contains("FsError"),"{name} [{mode}]: {help}");
                if typed=="seek" {assert!(text.contains("offset: int")&&text.contains("origin: SeekOrigin"),"{name} [{mode}]: {help}");}
                if let Some(prior)=&help_baseline {assert_eq!(&help,prior,"{name} [{mode}] signature help disagrees with settled");}else{help_baseline=Some(help);}
            }
            let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{name} [{mode}]: {counters}");}
        }
    }
}

#[test]
fn byte_exports_and_callable_help_agree_at_typing_speed_and_during_recovery() {
    let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let before="import { ";let text=format!("{before} }} from 'std:bytes';function main(){{}}");
    let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
    let imports=session.completion("main.tsr",&text,before.len()).unwrap();
    for entry in catalog["std:bytes"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();assert!(imports.iter().any(|label|label==name),"std:bytes.{name}: {imports:?}");}
    let cases=[
        ("UTF16 codec","import {fromUtf16} from 'std:bytes';function inspect(text:string){","fromUtf16","(text,'big');}function main(){}",&["ByteError","little","big","Ownership:"][..]),
        ("integer read","import {Bytes,readInt} from 'std:bytes';function inspect(bytes:ReadOnly<Bytes>){","readInt","(bytes,0,1,'little');}function main(){}",&["ByteError","width","Ownership:"][..]),
        ("distinct-owner copy","import {Bytes,copyInto} from 'std:bytes';function inspect(target:Mutable<Bytes>,source:ReadOnly<Bytes>){","copyInto","(target,0,source);}function main(){}",&["ByteError","distinct","Ownership:"][..]),
        ("rounded float write","import {Bytes,writeFloat32} from 'std:bytes';function inspect(bytes:Mutable<Bytes>){","writeFloat32","(bytes,0,1.5,'big');}function main(){}",&["ByteError","ties to even","Ownership:"][..]),
    ];
    for(name,before,typed,after,expected)in cases{
        let mut baseline=None;let mut help_baseline:Option<Value>=None;
        for(mode,delay,broken)in[("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
            let after=if broken{format!("{after}\nfunction broken(){{const = ;}}") }else{after.to_owned()};let text=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
            if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",before,typed,&after,delay);}else{session.open("main.tsr",&text);session.settle();}
            let reply=session.completion_items("main.tsr",&text,before.len()+typed.len());let items=if reply.is_array(){reply}else{reply["items"].clone()};let item=items.as_array().unwrap().iter().find(|item|item["label"]==typed).unwrap_or_else(||panic!("{name} [{mode}]: {items}"));let detail=item["detail"].as_str().unwrap().to_owned();let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();let combined=format!("{detail}\n{hover}");
            for fragment in expected{assert!(combined.contains(fragment),"{name} [{mode}]: missing {fragment}: {combined}");}assert!(!combined.contains("__tsrust_private_"),"{combined}");if let Some((prior_detail,prior_hover))=&baseline{assert_eq!((&detail,&hover),(prior_detail,prior_hover),"{name} [{mode}]");}else{baseline=Some((detail,hover));}
            let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));assert!(help.to_string().contains("ByteError"),"{name} [{mode}]: {help}");if let Some(prior)=&help_baseline{assert_eq!(&help,prior,"{name} [{mode}] parameter help");}else{help_baseline=Some(help);}
            let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{name} [{mode}]: {counters}");}
        }
    }
}

#[test]
fn literal_union_string_catalog_and_identity_agree_across_server_phases(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let expected=catalog["string"].as_array().unwrap().iter().map(|entry|entry["name"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
 for(prelude,annotation)in [("import {SeekOrigin} from 'std:fs';","SeekOrigin"),("type Origin=('start'|'current'|'end');","Origin")]{
  let before=format!("{prelude}function inspect(value:ReadOnly<{annotation}>){{value.");let typed="toUpperCase";let after="();}function main(){}";
  let mut baseline=None;
  for(mode,delay,broken)in [("settled",None,false),("typed-fast",Some(Duration::ZERO),false),("80ms",Some(Duration::from_millis(80)),false),("syntax-error",None,true)]{
   let after=if broken{format!("{after}function broken(){{const = ;}} ")}else{after.into()};let text=format!("{before}{typed}{after}");let mut session=Session::start(&[]);
   if let Some(delay)=delay{session.open("main.tsr",&format!("{before}{after}"));session.settle();session.type_text("main.tsr",&before,typed,&after,delay);}else{session.open("main.tsr",&text);session.settle();}
   let labels=session.completion("main.tsr",&text,before.len()).unwrap().into_iter().collect::<BTreeSet<_>>();assert_eq!(labels,expected,"{annotation} [{mode}]: full shared catalog");
   let hover=session.hover("main.tsr",&text,before.len()+2).unwrap_or_default();assert!(hover.contains("toUpperCase")&&hover.contains("string"),"{annotation} [{mode}]: {hover}");
   let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+typed.len()+1}}));assert!(help.to_string().contains("toUpperCase"),"{annotation} [{mode}]: {help}");
   let receiver_at=before.rfind("value.").unwrap()+2;let receiver=session.hover("main.tsr",&text,receiver_at).unwrap_or_default();assert!(receiver.contains(&format!("value: ReadOnly<{annotation}>")),"written alias/access lost for {annotation} [{mode}]: {receiver}");
   if let Some(prior)=&baseline{assert_eq!(prior,&(hover.clone(),help.clone(),receiver.clone()),"{annotation} [{mode}] phase disagreement");}else{baseline=Some((hover,help,receiver));}
   for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(session.request("tsrust/counters",Value::Null)[key],0,"{annotation} [{mode}]");}
  }
 }
}

#[test]
fn regex_exports_members_and_parameter_help_agree(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let before="import { ";let text=format!("{before} }} from 'std:regex';function main(){{}}");
 let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
 let imports=session.completion("main.tsr",&text,before.len()).unwrap();
 for entry in catalog["std:regex"].as_array().unwrap(){assert!(imports.iter().any(|name|name==entry["name"].as_str().unwrap()),"{imports:?}");}
 for hidden in ["__find","__stringMatch","regex_intrinsic"]{assert!(!imports.iter().any(|name|name==hidden));}
 let before="import {Regex as Pattern} from 'std:regex';function inspect(value:ReadOnly<Pattern>){value.";
 let text=format!("{before}isMatch('a');}}function main(){{}}");session.change("main.tsr",&text);session.settle();
 let offered=session.completion("main.tsr",&text,before.len()).unwrap().into_iter().collect::<BTreeSet<_>>();
 let expected=catalog["Regex"].as_array().unwrap().iter().map(|entry|entry["name"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();assert_eq!(offered,expected);
 for name in ["find","findAll","captures","capturesAll","replace","replaceAll","split"]{
  let arguments=if name.starts_with("replace"){"'a','$0'"}else{"'a'"};let text=format!("{before}{name}({arguments});}}function main(){{}}");session.change("main.tsr",&text);session.settle();
  let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();assert!(hover.contains("Ownership:")&&hover.contains(name),"{name}: {hover}");
  let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+name.len()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{name}: {help}");if name.starts_with("replace"){assert!(help.to_string().contains("replacementTemplate"));}
 }
 let counters=session.request("tsrust/counters",Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn locale_exports_members_and_parameter_help_agree(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();let before="import { ";let text=format!("{before} }} from 'std:locale';function main(){{}}");
 let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();let imports=session.completion("main.tsr",&text,before.len()).unwrap();
 for entry in catalog["std:locale"].as_array().unwrap(){assert!(imports.iter().any(|name|name==entry["name"].as_str().unwrap()),"{imports:?}");}
 for hidden in["__compare","__formatNumber","locale_intrinsic"]{assert!(!imports.iter().any(|name|name==hidden));}
 for (owner,method,args) in[("Collator","compare","'a','b'"),("Segmenter","segment","'é🦀'"),("NumberFormat","format","1")]{
  let before=format!("import {{{owner} as Owner}} from 'std:locale';function inspect(value:ReadOnly<Owner>){{value.");let text=format!("{before}{method}({args});}}function main(){{}}");session.change("main.tsr",&text);session.settle();
  let offered=session.completion("main.tsr",&text,before.len()).unwrap().into_iter().collect::<BTreeSet<_>>();let expected=catalog[owner].as_array().unwrap().iter().map(|entry|entry["name"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();assert_eq!(offered,expected);
  let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();assert!(hover.contains("Ownership:")&&hover.contains(method),"{hover}");
  let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()+method.len()+1}}));assert!(!help["signatures"].as_array().unwrap().is_empty(),"{help}");
 }
}

#[test]
fn io_exports_error_fields_and_parameter_help_agree(){
 let catalog:Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let before="import { ";let text=format!("{before} }} from 'std:io';function main(){{}}");
 let mut session=Session::start(&[]);session.open("main.tsr",&text);session.settle();
 let imports=session.completion("main.tsr",&text,before.len()).unwrap();
 for entry in catalog["std:io"].as_array().unwrap(){assert!(imports.iter().any(|name|name==entry["name"].as_str().unwrap()),"{imports:?}");}
 assert!(!imports.iter().any(|name|name.starts_with("__")));
 let before="import {IoError as Failure} from 'std:io';function inspect(value:ReadOnly<Failure>){value.";
 let text=format!("{before}code;}}function main(){{}}");session.change("main.tsr",&text);session.settle();
 let offered=session.completion("main.tsr",&text,before.len()).unwrap().into_iter().collect::<BTreeSet<_>>();
 assert_eq!(offered,["code","operation","message","osCode"].into_iter().map(str::to_owned).collect());
 let hover=session.hover("main.tsr",&text,before.len()+1).unwrap_or_default();assert!(hover.contains("Ownership:")&&hover.contains("Failure category"),"{hover}");
 let before="import {Reader,readInto} from 'std:io';import {Bytes} from 'std:bytes';async function inspect(input:Mutable<Reader>,output:Mutable<Bytes>):Task<void>{await readInto(";
 let text=format!("{before}input,output);}}function main(){{}}");session.change("main.tsr",&text);session.settle();
 let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":before.len()}}));
 let signatures=help["signatures"].as_array().unwrap();assert!(!signatures.is_empty(),"{help}");
 let label=signatures[0]["label"].as_str().unwrap();assert!(label.contains("destination")&&label.contains("Task<Result<int"),"{label}");
}
