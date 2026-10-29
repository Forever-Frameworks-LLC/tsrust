mod support;
#[test]
fn json_child_views_preserve_nested_storage_and_inspection() {
 let source=r#"
import {get,at,asInt,JsonView,JsonError} from 'std:json';
import {strictEqual} from 'std:testing';
function first(value:ReadOnly<JsonView>):Result<JsonView|undefined,JsonError>{return at(value,0);}
function main(){
 const owner=JSON.parse('{"items":[42,null]}');
 match(get(owner,'items')){Ok(items)=>{if(items!==undefined){
  strictEqual(Array.isArray(items),true);
  match(first(items)){Ok(child)=>{if(child!==undefined){match(asInt(child)){Ok(value)=>{strictEqual(value,42);},Err(error)=>{throw error.message;}}strictEqual(typeof child,'number');strictEqual(JSON.stringify(child),'42');}},Err(error)=>{throw error.message;}}
  match(at(items,1)){Ok(child)=>{if(child!==undefined){strictEqual(child===null,true);}else{throw 'null lost';}},Err(error)=>{throw error.message;}}
  match(at(items,2)){Ok(child)=>{strictEqual(child===undefined,true);},Err(error)=>{throw error.message;}}
 }else{throw 'items missing';}},Err(error)=>{throw error.message;}}
 strictEqual(JSON.stringify(owner),'{"items":[42,null]}');
 match(get(owner,'missing')){Ok(child)=>{strictEqual(child===undefined,true);},Err(error)=>{throw error.message;}}
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn json_views_reject_owner_invalidation_clone_and_escape() {
 for (label,source,code) in [
  ("replace", "import {get} from 'std:json';function main(){let owner=JSON.parse('{}');const child=get(owner,'x');owner=JSON.parse('{}');match(child){Ok(value)=>{console.log(typeof value);},Err(error)=>{throw error.message;}}}", "native-view-conflict"),
  ("null remains borrowed", "import {get} from 'std:json';function main(){let owner=JSON.parse('{\"x\":null}');match(get(owner,'x')){Ok(child)=>{if(child===null){owner=JSON.parse('{}');console.log(typeof child);}},Err(error)=>{throw error.message;}}}", "native-view-conflict"),
  ("escape", "import {get} from 'std:json';function bad(){const owner=JSON.parse('{}');return get(owner,'x');}function main(){}", "native-view-escape"),
  ("clone", "import {get} from 'std:json';function main(){const owner=JSON.parse('{}');match(get(owner,'x')){Ok(child)=>{if(child!==undefined){clone(child);}},Err(error)=>{throw error.message;}}}", "clone-type"),
 ] {
  let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
  let output=support::run(support::cli().arg("check").arg(path),60);
  let stderr=String::from_utf8_lossy(&output.stderr);
  assert!(!output.status.success()&&stderr.contains(&format!("error[{code}]")),"{label}: {stderr}");
 }
}

#[test]
fn json_absent_and_error_branches_release_the_source_loan() {
 let source=r#"
import {get,JsonValue,JsonView,JsonError} from 'std:json';
import {strictEqual} from 'std:testing';
function child(value:ReadOnly<JsonValue>):Result<JsonView|undefined,JsonError>{return get(value,'x');}
function main(){
 let owner=JSON.parse('{}');
 match(child(owner)){Ok(value)=>{if(value===undefined){owner=JSON.parse('null');}else{throw 'unexpected child';}},Err(error)=>{throw error.message;}}
 strictEqual(owner===null,true);
 match(child(owner)){Ok(value)=>{throw 'unexpected object';},Err(error)=>{strictEqual(error.code,'expected-object');owner=JSON.parse('{}');}}
 strictEqual(JSON.stringify(owner),'{}');
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 support::success(&support::run(support::cli().arg("run").arg(path),90));
}

#[test]
fn json_view_catalog_examples_execute() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 for entry in catalog["std:json"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("JsonView"|"get"|"at"))) {
  for example in entry["examples"].as_array().unwrap(){
   let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
   let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);
   assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
  }
 }
}

// A native argument with typed alternatives must see through the global
// unwrap and a non-null assertion to select the JsonView alternative.
#[test]
fn global_unwrap_selects_the_json_view_alternative_inline() {
    let source = r#"
import {parse,get,asInt} from 'std:json';
function main(){
 const owner=unwrap(parse('{"a":7}'));
 console.log(unwrap(asInt(unwrap(get(owner,'a'))!)));
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 90);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "7\n");
}
