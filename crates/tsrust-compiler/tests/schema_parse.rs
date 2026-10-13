//! Throwing validation retains the safeParse ownership and error contracts.
mod support;

#[test]
fn parse_returns_typed_values_and_throws_validation_issues() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
const greeting=s.string({message:'Expected greeting'}).min(1);
function main(){
  strictEqual(greeting.parse('Hello'),'Hello');
  let failed=false;
  try{greeting.parse('');}
  catch(error){failed=true;strictEqual(error.issues.length,1);}
  strictEqual(failed,true);
  const person=s.object({name:s.string(),age:s.int().min(0)});
  const value=person.parse({name:'Pat',age:7});
  strictEqual(value.name,'Pat');strictEqual(value.age,7);
  const size=s.string().transform(value=>value.length);
  strictEqual(size.parse('abc'),3);
  strictEqual(s.int().min(1).parse(2),2);
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn parse_preserves_input_consumption() {
    let source="import {s} from 'std:schema';function main(){const schema=s.string();const value=String('hello');schema.parse(value);console.log(value);}";
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("check").arg(file),30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("transferred"),"{}",String::from_utf8_lossy(&output.stderr));
}

#[test]
fn parse_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let entry=catalog["Schema"].as_array().unwrap().iter().find(|entry|entry["name"]=="parse").unwrap();
    let example=&entry["examples"][0];
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
    let output=support::run(support::cli().arg("run").arg(file),60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
}

#[test]
fn parse_evaluates_input_once_and_preserves_callback_exceptions() {
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
let calls:int=0;
function input():string{calls+=1;return 'hello';}
function main(){
  const schema=s.string();
  strictEqual(schema.parse(input()),'hello');
  strictEqual(calls,1);
  const callback=s.string().transform(value=>{if(value==='hello'){throw 'callback failure';}return value;});
  let caught=false;
  try{callback.parse('hello');}
  catch(error){if(typeof error==='string'){strictEqual(error,'callback failure');caught=true;}}
  strictEqual(caught,true);
}
"#;
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}

#[test]
fn parse_accepts_inline_object_schema_and_input() {
    let source="import {s} from 'std:schema';import {strictEqual} from 'std:testing';function main(){const value=s.object({name:s.string()}).parse({name:'Pat'});strictEqual(value.name,'Pat');}";
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(file),60));
}
