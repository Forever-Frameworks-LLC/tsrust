mod support;
#[test]
fn json_scalar_construction_preserves_values_and_rejects_nonfinite_numbers() {
 let source=r#"
import {nullValue,fromBoolean,fromInt,fromNumber,fromString,asInt,asNumber,takeString} from 'std:json';
import {strictEqual} from 'std:testing';
function main(){
 strictEqual(JSON.stringify(fromInt(-9223372036854775808)),'-9223372036854775808');
 strictEqual(JSON.stringify(nullValue()),'null');strictEqual(JSON.stringify(fromBoolean(true)),'true');
 const integer=fromInt(9223372036854775807);strictEqual(JSON.stringify(integer),'9223372036854775807');match(asInt(integer)){Ok(value)=>{strictEqual(value,9223372036854775807);},Err(error)=>{throw error.message;}}
 match(fromNumber(-0.0)){Ok(value)=>{match(asNumber(value)){Ok(number)=>{strictEqual(Object.is(number,-0.0),true);},Err(error)=>{throw error.message;}}},Err(error)=>{throw error.message;}}
 match(fromNumber(1.25)){Ok(value)=>{strictEqual(JSON.stringify(value),'1.25');},Err(error)=>{throw error.message;}}
 match(fromNumber(Number.NaN)){Ok(value)=>{throw 'NaN accepted';},Err(error)=>{strictEqual(error.code,'number-nonfinite');strictEqual(error.offset,0);}}
 match(fromNumber(Number.POSITIVE_INFINITY)){Ok(value)=>{throw 'infinity accepted';},Err(error)=>{strictEqual(error.code,'number-nonfinite');}}
 match(fromNumber(Number.NEGATIVE_INFINITY)){Ok(value)=>{throw 'negative infinity accepted';},Err(error)=>{strictEqual(error.code,'number-nonfinite');}}
 const text='hello é';const value=fromString(text);match(takeString(value)){Ok(result)=>{strictEqual(result,'hello é');},Err(error)=>{throw error.message;}}
}
"#;
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}
#[test]
fn json_string_construction_transfers_ownership() {
 let source="import {fromString} from 'std:json';function main(){const text='owned';const value=fromString(text);console.log(text);}";
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("error[transferred]"));
}

#[test]
fn json_construction_catalog_examples_execute() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 for entry in catalog["std:json"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("nullValue"|"fromBoolean"|"fromInt"|"fromNumber"|"fromString"))) {
  for example in entry["examples"].as_array().unwrap(){let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(example["source"].as_str().unwrap()));let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());}
 }
}
