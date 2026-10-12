mod support;
#[test]
fn typeof_json_boolean_refinement_executes_without_consuming_owner() {
  let source = r#"
import {strictEqual} from 'std:testing';
import {JsonValue,get} from 'std:json';
function read(value:ReadOnly<JsonValue>):boolean {if(typeof value==='boolean'){return value;}return false;}
function main(){const value=JSON.parse('true');strictEqual(read(value),true);if(typeof value==='boolean'){const result:boolean=value;strictEqual(result,true);const inferred=value;strictEqual(!inferred,false);}strictEqual(JSON.stringify(value),'true');const owner=JSON.parse('{"flag":false}');match(get(owner,'flag')){Ok(child)=>{if(typeof child==='boolean'){const result:boolean=child;strictEqual(result,false);}},Err(error)=>{throw error.message;}}}
"#;
  let directory = support::TempDirectory::new();
  let path = directory.source(&support::format_round_trip(source));
  support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn json_boolean_proofs_do_not_survive_mutation_or_unproven_joins() {
  for body in [
    "if(typeof value==='boolean'){value=JSON.parse('42');const result:boolean=value;}",
    "if(typeof value!=='boolean'){const result:boolean=value;}",
    "if(typeof value==='boolean'){}const result:boolean=value;",
    "if(typeof value==='boolean'){while(true){const result:boolean=value;value=JSON.parse('42');}}",
  ] {
    let source = format!("function main(){{let value=JSON.parse('true');{body}}}");
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(&source));
    let output = support::run(support::cli().arg("check").arg(path), 60);
    assert!(!output.status.success(), "unsound proof accepted: {body}");
  }
}
