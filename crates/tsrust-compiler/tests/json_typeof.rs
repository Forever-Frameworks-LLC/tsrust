mod support;

#[test]
fn typeof_json_observes_contents_without_consuming_or_repeating_evaluation() {
    let source = r#"
import {JsonValue} from 'std:json';
import {strictEqual} from 'std:testing';
function inspect(value: ReadOnly<JsonValue | undefined>): string { return typeof value; }
function guarded(value: ReadOnly<JsonValue | string>): string {
  if(typeof value==='string'){return typeof value;}
  return typeof value;
}
function main() {
  const text=JSON.parse('"hello"');
  strictEqual(typeof text,'string');
  strictEqual(JSON.stringify(text),'"hello"');
  strictEqual(typeof JSON.parse('42'),'number');
  strictEqual(typeof JSON.parse('1.5'),'number');
  strictEqual(typeof JSON.parse('true'),'boolean');
  strictEqual(typeof JSON.parse('null'),'object');
  strictEqual(typeof JSON.parse('[]'),'object');
  strictEqual(typeof JSON.parse('{}'),'object');
  const optional:JsonValue|undefined=JSON.parse('"union"');
  strictEqual(inspect(optional),'string');
  strictEqual(inspect(optional),'string');
  strictEqual(inspect(undefined),'undefined');
  const unionText:JsonValue|string=JSON.parse('"inside"');
  const unionNumber:JsonValue|string=JSON.parse('123');
  strictEqual(guarded(unionText),'string');
  strictEqual(guarded(unionNumber),'number');
  strictEqual(guarded('ordinary'),'string');
  let calls=0;
  {const make=():JsonValue=>{calls++;return JSON.parse('false');};
  strictEqual(typeof make(),'boolean');}
  strictEqual(calls,1);
  if(typeof text==='string'){strictEqual(JSON.stringify(text),'"hello"');}else{throw 'wrong branch';}
}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn json_typeof_catalog_example_executes() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let entry=catalog.as_object().unwrap().values().filter_map(|value|value.as_array()).flatten().find(|entry|entry["name"]=="typeof").unwrap();
    let example=entry["examples"].as_array().unwrap().iter().find(|example|example["source"].as_str().unwrap().contains("JSON.parse")).unwrap();
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
    let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}
