mod support;
#[test]
fn borrowed_json_strings_cross_locals_and_function_returns() {
 let source=r#"
import {asString,JsonValue,JsonError,StringView,get} from 'std:json';
import {strictEqual} from 'std:testing';
function text(value:ReadOnly<JsonValue>):Result<StringView,JsonError>{return asString(value);}
function copyText(value:ReadOnly<JsonValue>):string{match(asString(value)){Ok(view)=>{return String(view);},Err(error)=>{throw error.message;}}}
function main(){
 const owner=JSON.parse('"hello"');
 match(text(owner)){Ok(view)=>{strictEqual(view.length,5);strictEqual(view.includes('ell'),true);strictEqual(typeof view,'string');const owned=String(view);strictEqual(owned,'hello');strictEqual('hello'.includes(view),true);strictEqual(view.startsWith('he'),true);strictEqual(view.endsWith('lo'),true);strictEqual(view.indexOf('l'),2);strictEqual(view.lastIndexOf('l'),3);console.log(view);const maybe:StringView|undefined=view;strictEqual(typeof maybe,'string');strictEqual(String(maybe),'hello');if(typeof maybe==='string'){strictEqual(maybe.length,5);}},Err(error)=>{throw error.message;}}
 const object=JSON.parse('{"text":"nested"}');
 match(get(object,'text')){Ok(child)=>{if(child!==undefined){match(asString(child)){Ok(view)=>{strictEqual(view.length,6);},Err(error)=>{throw error.message;}}}},Err(error)=>{throw error.message;}}
 strictEqual(JSON.stringify(owner),'"hello"');
 let replaceable=JSON.parse('42');match(asString(replaceable)){Ok(view)=>{throw 'wrong';},Err(error)=>{replaceable=JSON.parse('null');}}
 let copiedOwner=JSON.parse('\"copy\"');const copied=copyText(copiedOwner);copiedOwner=JSON.parse('null');strictEqual(copied,'copy');
 const empty=JSON.parse('""');match(asString(empty)){Ok(view)=>{strictEqual(Boolean(view),false);const maybe:StringView|undefined=view;strictEqual(Boolean(maybe),false);},Err(error)=>{throw error.message;}}
 const wrong=JSON.parse('42');match(asString(wrong)){Ok(view)=>{throw 'wrong kind';},Err(error)=>{strictEqual(error.code,'expected-string');}}
}
"#;
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);assert_eq!(output.stdout,b"hello\n");
}

#[test]
fn borrowed_json_strings_reject_escape_clone_owner_replacement_and_array_search() {
 for (body,code) in [
  ("function bad(){const owner=JSON.parse('\"text\"');return asString(owner);}function main(){}","native-view-escape"),
  ("function main(){const owner=JSON.parse('\"text\"');match(asString(owner)){Ok(view)=>{clone(view);},Err(error)=>{throw error.message;}}}","clone-type"),
  ("function main(){let owner=JSON.parse('\"text\"');match(asString(owner)){Ok(view)=>{owner=JSON.parse('null');console.log(view.length);},Err(error)=>{throw error.message;}}}","native-view-conflict"),
  ("function main(){const owner=JSON.parse('\"text\"');match(asString(owner)){Ok(view)=>{const values=[1];values.includes(view);},Err(error)=>{throw error.message;}}}","type"),
  ("function main(){const owner=JSON.parse('\"text\"');if(typeof owner==='string'){const values=['text'];values.includes(owner);}}","type"),
 ] {
  let source=format!("import {{asString}} from 'std:json';{body}");let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),60);let diagnostics=String::from_utf8_lossy(&output.stderr);assert!(!output.status.success()&&diagnostics.contains(&format!("error[{code}]")),"{body}: {diagnostics}");
 }
}

#[test]
fn borrowed_json_string_catalog_example_executes() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let entries=catalog["std:json"].as_array().unwrap();let entry=entries.iter().find(|entry|entry["name"]=="asString").unwrap();let alias=entries.iter().find(|entry|entry["name"]=="StringView").unwrap();assert_eq!(entry["examples"],alias["examples"]);
 let example=&entry["examples"][0];let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));let output=support::run(support::cli().arg("run").arg(path),90);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}
