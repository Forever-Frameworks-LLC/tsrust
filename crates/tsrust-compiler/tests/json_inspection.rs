mod support;
#[test]
fn json_array_and_null_inspection_preserves_values_and_optional_branches() {
    let source=r#"
import {JsonValue} from 'std:json';
import {strictEqual} from 'std:testing';
function isNull(value:ReadOnly<JsonValue|null|undefined>):boolean{return value===null;}
function isArray(value:ReadOnly<JsonValue|undefined>):boolean{return Array.isArray(value);}
function guarded(value:ReadOnly<JsonValue|null>):string{if(value===null){return typeof value;}return typeof value;}
function main(){
  const array=JSON.parse('[null]');const nil=JSON.parse('null');const object=JSON.parse('{}');
  strictEqual(Array.isArray(array),true);strictEqual(Array.isArray(nil),false);strictEqual(Array.isArray(object),false);
  strictEqual(Array.isArray(JSON.parse('"[]"')),false);strictEqual(Array.isArray(JSON.parse('0')),false);
  strictEqual(nil===null,true);strictEqual(null===nil,true);strictEqual(nil!==null,false);strictEqual(null!==array,true);
  strictEqual(nil===undefined,false);strictEqual(array===null,false);strictEqual(object===null,false);
  strictEqual(JSON.stringify(array),'[null]');strictEqual(JSON.stringify(nil),'null');
  const optional:JsonValue|undefined=JSON.parse('[]');strictEqual(isArray(optional),true);strictEqual(isArray(undefined),false);
  const union:JsonValue|null|undefined=JSON.parse('null');strictEqual(isNull(union),true);strictEqual(isNull(null),true);strictEqual(isNull(undefined),false);
  const number:JsonValue|null=JSON.parse('1');strictEqual(guarded(number),'number');strictEqual(guarded(null),'object');
  const jsonNull:JsonValue|null=JSON.parse('null');strictEqual(guarded(jsonNull),'object');
  let calls=0;{const make=():JsonValue=>{calls++;return JSON.parse('[]');};strictEqual(Array.isArray(make()),true);strictEqual(make()===null,false);}strictEqual(calls,2);
}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn json_inspection_catalog_example_executes() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let entry=catalog["ArrayConstructor"].as_array().unwrap().iter().find(|entry|entry["name"]=="isArray").unwrap();
    let example=entry["examples"].as_array().unwrap().iter().find(|example|example["source"].as_str().unwrap().contains("JSON.parse")).unwrap();
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
    let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}
