mod support;
#[test]
fn owned_json_containers_build_nested_values_and_validate_limits() {
 let source=r#"
import {fromArray,fromEntries,fromInt,fromString,nullValue,JsonValue,JsonEntry,JsonBuildLimits} from 'std:json';
import {strictEqual} from 'std:testing';
function main(){
 const values:JsonValue[]=[fromInt(42),nullValue()];
 match(fromArray(values)){Ok(array)=>{const entries:JsonEntry[]=[{key:'items',value:array},{key:'z',value:nullValue()},{key:'z',value:fromString('last')}];match(fromEntries(entries)){Ok(object)=>{strictEqual(JSON.stringify(object),'{"items":[42,null],"z":"last"}');},Err(error)=>{throw error.message;}}},Err(error)=>{throw error.message;}}
 const empty:JsonValue[]=[];const zero:JsonBuildLimits={maxDepth:1,maxEntries:0,maxNodes:1};match(fromArray(empty,zero)){Ok(value)=>{strictEqual(JSON.stringify(value),'[]');},Err(error)=>{throw error.message;}}
 const limited:JsonBuildLimits={maxDepth:1,maxEntries:1,maxNodes:2};const over:JsonValue[]=[nullValue(),nullValue()];match(fromArray(over,limited)){Ok(value)=>{throw 'limit ignored';},Err(error)=>{strictEqual(error.code,'entry-limit');strictEqual(error.offset,0);}}
 const duplicate:JsonEntry[]=[{key:'x',value:nullValue()},{key:'x',value:nullValue()}];match(fromEntries(duplicate,limited)){Ok(value)=>{throw 'duplicate limit ignored';},Err(error)=>{strictEqual(error.code,'entry-limit');}}
 const invalid:JsonBuildLimits={maxDepth:-1,maxEntries:1,maxNodes:2};const input:JsonValue[]=[];match(fromArray(input,invalid)){Ok(value)=>{throw 'negative limit ignored';},Err(error)=>{strictEqual(error.code,'invalid-limits');}}
}
"#;
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}
#[test]
fn json_containers_consume_input_arrays_even_on_error() {
 for (ty,call) in [("JsonValue","fromArray(values)"),("JsonValue","fromArray(values,{maxDepth:0,maxEntries:0,maxNodes:0})"),("JsonEntry","fromEntries(values)"),("JsonEntry","fromEntries(values,{maxDepth:0,maxEntries:0,maxNodes:0})")] {
 let source=format!("import {{fromArray,fromEntries,JsonValue,JsonEntry}} from 'std:json';function main(){{const values:{ty}[]=[];{call};console.log(values.length);}}");let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("error[transferred]"));
 }
}

#[test]
fn json_container_catalog_examples_execute() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let mut sources=std::collections::HashSet::new();let mut count=0;
 for entry in catalog["std:json"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("fromArray"|"fromEntries"|"JsonEntry"|"JsonBuildLimits"))){count+=1;for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();if !sources.insert(source){continue;}let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());}}
 assert_eq!(count,4);
}

#[test]
fn json_container_children_require_owned_json_values() {
 let source="import {get,fromArray,JsonValue} from 'std:json';function main(){const owner=JSON.parse('{\"x\":42}');match(get(owner,'x')){Ok(child)=>{if(child!==undefined){const values:JsonValue[]=[child];fromArray(values);}},Err(error)=>{throw error.message;}}}";
 let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success());let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains("Expected JsonValue, found JsonView"),"{error}");
}
