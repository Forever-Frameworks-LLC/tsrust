mod support;

#[test]
fn checked_json_scalars_preserve_values_and_report_conversion_failures() {
    let source=r#"
import {asBoolean, asInt, asNumber, takeString} from 'std:json';
import {strictEqual} from 'std:testing';
function main() {
  const integer=JSON.parse('9007199254740993');
  strictEqual(typeof integer,'number');
  if(typeof integer==='number'){
  match(asInt(integer)){Ok(value)=>{strictEqual(value,9007199254740993);},Err(error)=>{throw error;}}
  match(asNumber(integer)){Ok(_)=>{throw 'lost precision';},Err(error)=>{strictEqual(error.code,'number-precision');}}
  }
  strictEqual(JSON.stringify(integer),'9007199254740993');
  match(asBoolean(JSON.parse('false'))){Ok(value)=>{strictEqual(value,false);},Err(error)=>{throw error;}}
  match(asInt(JSON.parse('1.5'))){Ok(_)=>{throw 'rounded fraction';},Err(error)=>{strictEqual(error.code,'integer-fractional');}}
  match(asNumber(JSON.parse('-0.0'))){Ok(value)=>{strictEqual(Object.is(value,-0.0),true);},Err(error)=>{throw error;}}
  match(takeString(JSON.parse('"hello"'))){Ok(value)=>{strictEqual(value,'hello');},Err(error)=>{throw error;}}
  match(asInt(JSON.parse('null'))){Ok(_)=>{throw 'coerced null';},Err(error)=>{strictEqual(error.code,'expected-number');strictEqual(error.offset,0);}}
}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn consuming_string_extraction_rejects_later_owner_use() {
    let source="import {takeString} from 'std:json';function main(){const value=JSON.parse('\"text\"');takeString(value);JSON.stringify(value);}";
    let directory=support::TempDirectory::new();let path=directory.source(source);
    let error=tsrust_compiler::project_cli::check_input(&path,source).err().expect("moved owner must reject");
    let rendered=error.render("input.tsr",source,false);
    assert!(rendered.contains("Value transferred here"),"{rendered}");
}

#[test]
fn json_scalar_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    for entry in catalog["std:json"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("asBoolean"|"asInt"|"asNumber"|"takeString"))) {
        for example in entry["examples"].as_array().unwrap() {
            let directory=support::TempDirectory::new();
            let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
            let output=support::run(support::cli().arg("run").arg(path),90);
            support::success(&output);
            assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
        }
    }
}

#[test]
fn nested_match_shadow_restores_outer_binding() {
    tsrust_compiler::check("function main(){const value=42;match(Ok<int,string>(1)){Ok(value)=>{console.log(value);},Err(error)=>{throw error;}}console.log(value);}").unwrap();
}

#[test]
fn json_number_guard_does_not_implicitly_round_into_number() {
 let source="function main(){const value=JSON.parse('9007199254740993');if(typeof value==='number'){const rounded:number=value;}}";
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("error[type]"));
}
