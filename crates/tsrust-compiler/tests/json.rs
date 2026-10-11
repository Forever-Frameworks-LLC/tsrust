mod support;
#[test]
fn json_dynamic_round_trip_limits_and_owned_values() {
 let dir=support::TempDirectory::new();
 let source=r#"import {parse, stringify, kind, JsonValue, JsonError, JsonLimits} from 'std:json';
 import {strictEqual} from 'std:testing';
 function saved():JsonValue {const text='{"z":1,"a":"é","z":2}';const decode=parse;match(decode(text)){Ok(value)=>{return value;},Err(error)=>{throw error.message;}}}
 function main(){
  const value=saved();const copied=clone(value);strictEqual(kind(value),'object');
  match(stringify(copied)){Ok(text)=>{strictEqual(text,'{"a":"é","z":2}');},Err(error)=>{throw error.message;}}
  match(parse('null secret')){Ok(_)=>{throw 'accepted trailing data';},Err(error)=>{const typed:JsonError=error;strictEqual(typed.code,'trailing-data');strictEqual(typed.offset,5);}}
  const limits:JsonLimits={maxBytes:20,maxDepth:2,maxEntries:1,maxNodes:10};
  match(parse('[1,2]',limits)){Ok(_)=>{throw 'accepted over limit';},Err(error)=>{strictEqual(error.code,'entry-limit');}}
  match(stringify(value,1)){Ok(_)=>{throw 'accepted output over limit';},Err(error)=>{strictEqual(error.code,'output-limit');}}
  console.log('JSON passed');
 }"#;
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(source)).unwrap();
 for release in [false,true] {let mut command=support::cli();command.arg("run").arg(&path);if release{command.arg("--release");}let output=support::run(&mut command,90);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"JSON passed\n");}
}

#[test]
fn json_rejects_invalid_calls_and_forged_values() {
 let dir=support::TempDirectory::new();let path=dir.0.join("bad.tsr");
 for statement in ["parse()", "parse(1)", "kind('null')", "stringify('null')", "const value:JsonValue={}"] {
  let source=format!("import {{parse,kind,stringify,JsonValue}} from 'std:json'; function main(){{{statement};}}");
  std::fs::write(&path,support::format_round_trip(&source)).unwrap();
  let output=support::run(support::cli().arg("check").arg(&path),20);assert!(!output.status.success(),"accepted {statement}");
 }
}
#[test]
fn json_catalog_examples_execute_through_cli() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let dir=support::TempDirectory::new();let path=dir.0.join("example.tsr");
 let entries=catalog["std:json"].as_array().unwrap();assert_eq!(entries.len(),6);
 for entry in entries {for example in entry["examples"].as_array().unwrap() {
  std::fs::write(&path,support::format_round_trip(example["source"].as_str().unwrap())).unwrap();
  let output=support::run(support::cli().arg("run").arg(&path),90);support::success(&output);
  assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap(),"{}",entry["name"]);
 }}
}
