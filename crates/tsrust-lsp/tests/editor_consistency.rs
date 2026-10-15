//! Hover, completion and the catalog must not contradict one another, and
//! the server must report no misleading answer while a document is typed.
mod support;
use serde_json::Value;
use std::collections::BTreeSet;
use std::time::Duration;
use support::session::Session;

const GREETING: &str = "export function greeting(): string { return \"hi\"; }\n";
const HEAD: &str = "import { greeting } from './greeting.tsr';\nimport { s } from 'std:schema';\n\ntype Item = { id: int; name: string; };\nconst schema = s.string();\n\nfunction main() {\n  const values: int[] = [1, 2, 3];\n  const item: Item = { id: 1, name: \"a\" };\n  const text = \"abc\";\n  const lookup = new Map<string, int>();\n  const seen = new Set<int>();\n  const parsed = schema.safeParse(greeting());\n  const loaded: Result<string, int> = Ok(greeting());\n  ";
const TAIL: &str = "\n}\n";

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
