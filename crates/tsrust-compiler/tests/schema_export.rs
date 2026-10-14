//! JSON Schema export must reflect reusable runtime plans, not source-text guesses.
mod support;

fn exported(source: &str) -> serde_json::Value {
    let directory = support::TempDirectory::new();
    let file = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(file), 60);
    support::success(&output);
    serde_json::from_slice(&output.stdout).expect("one exported JSON document")
}

#[test]
fn schema_export_preserves_runtime_bounds_presence_and_strict_fields() {
    let value = exported(r#"
import {s} from 'std:schema';
function build(limit:int){
  const schema=s.object({name:s.string().min(limit),age:s.int().min(0),note:s.string().optional()});
  console.log(JSON.stringify(schema.toJsonSchema()));
  {const outcome0=schema.safeParseJson('{"name":"Ada","age":4}');if(outcome0.success){const value=outcome0.data;}else{const error=outcome0.error;throw 'Export consumed schema';}}
}
function main(){build(2);}
"#);
    assert_eq!(value["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(value["type"], "object");
    assert_eq!(value["additionalProperties"], false);
    assert_eq!(value["properties"]["name"]["minLength"], 2);
    assert_eq!(value["properties"]["age"]["minimum"], 0);
    let required=value["required"].as_array().expect("required fields");
    assert!(required.contains(&serde_json::json!("name")));
    assert!(required.contains(&serde_json::json!("age")));
    assert!(!required.contains(&serde_json::json!("note")));
}

#[test]
fn schema_export_preserves_numeric_bounds_without_relaxing_native_conversion() {
    let value = exported(r#"
import {s} from 'std:schema';
function main(){
  const schema=s.object({decimal:s.number().min(0.5).max(12.25),exact:s.int()});
  const number=s.number();
  {const outcome1=number.safeParseJson('9007199254740993');if(outcome1.success){const value=outcome1.data;throw 'Export must not relax native precision checks';}else{const error=outcome1.error;}}
  {const outcome2=number.safeParseJson('9007199254740992');if(outcome2.success){const value=outcome2.data;}else{const error=outcome2.error;throw 'Exactly representable integer rejected';}}
  console.log(JSON.stringify(schema.toJsonSchema()));
}
"#);
    let decimal = &value["properties"]["decimal"];
    assert_eq!(decimal["type"], "number");
    assert_eq!(decimal["minimum"], 0.5);
    assert_eq!(decimal["maximum"], 12.25);
    let integer = &value["properties"]["exact"];
    assert_eq!(integer["minimum"].as_i64(), Some(i64::MIN));
    assert_eq!(integer["maximum"].as_i64(), Some(i64::MAX));
}

#[test]
fn schema_export_encodes_finite_recursive_references() {
    let value=exported(r#"
import {s} from 'std:schema';
function main(){
  const schema=s.recursive(self=>s.object({value:s.int(),children:s.array(self)}));
  console.log(JSON.stringify(schema.toJsonSchema()));
}

"#);
    let reference=value["$ref"].as_str().expect("recursive root reference");
    let pointer=reference.strip_prefix('#').expect("local reference");
    let definition=value.pointer(pointer).expect("resolved root definition");
    assert_eq!(definition["type"],"object");
    assert_eq!(definition["properties"]["children"]["items"]["$ref"],reference);
    assert!(serde_json::to_string(&value).unwrap().len()<4096);
}

#[test]
fn schema_export_uses_any_of_for_overlapping_union_branches() {
    let value = exported(r#"
import {s} from 'std:schema';
function main(){
  const schema=s.union([s.string().min(1),s.string().max(10)]);
  console.log(JSON.stringify(schema.toJsonSchema()));
}
"#);
    assert!(value.get("oneOf").is_none());
    // The scalar plan also retains its outer bounds. Those intersect the
    // alternatives rather than changing anyOf into exclusive oneOf.
    let branches = value["allOf"][0]["anyOf"].as_array().expect("overlapping alternatives");
    assert_eq!(branches.len(), 2);
    assert_eq!(branches[0]["minLength"], 1);
    assert_eq!(branches[1]["maxLength"], 10);
}

#[test]
fn schema_export_preserves_message_wrapped_constraint_intersections() {
    let value = exported(r#"
import {s} from 'std:schema';
function main(){
  const schema=s.string().min(4).min(2,{message:'Too short'});
  console.log(JSON.stringify(schema.toJsonSchema()));
}
"#);
    let branches=value["allOf"].as_array().expect("intersected constraints");
    assert_eq!(branches[0]["minLength"],4);
    // Construction inherits the tighter existing bound into the message plan.
    assert_eq!(branches[1]["minLength"],4);
}

#[test]
fn schema_export_rejects_unrepresentable_rules_without_invoking_callbacks() {
    for plan in [
        "s.string().refine(value=>{throw 'Callback ran';})",
        "s.string().transform((value:string):string=>{throw 'Callback ran';})",
        "s.string().default(()=>{throw 'Factory ran';})",
        "s.string().superRefine((value,ctx)=>{throw 'Callback ran';})",
        "s.string().coerce((value:int):string=>{throw 'Callback ran';})",
        "s.map(s.string(),s.int())",
        "s.set(s.string())",
    ] {
        let source=format!(r#"
import {{s,SchemaExportError}} from 'std:schema';
function report(error:SchemaExportError){{console.log(JSON.stringify(error));}}
function main(){{
  const schema={plan};
  try {{schema.toJsonSchema();}}
  catch(error){{report(error);}}
}}
"#);
        let value=exported(&source);
        assert_eq!(value["code"],"unrepresentable");
        assert_eq!(value["path"],"");
        assert!(!value["reason"].as_str().expect("reason").is_empty());
    }
}

#[test]
fn schema_export_reports_nested_rule_paths_without_partial_output() {
    let value=exported(r#"
import {s,SchemaExportError} from 'std:schema';
function report(error:SchemaExportError){console.log(JSON.stringify(error));}
function main(){
  const schema=s.object({items:s.array(s.string().refine(value=>true))});
  try {schema.toJsonSchema();}
  catch(error){report(error);}
}
"#);
    assert_eq!(value["code"],"unrepresentable");
    assert_eq!(value["path"],"/properties/items/items");
    assert!(value.get("$schema").is_none());
}

#[test]
fn schema_export_catalog_example_executes() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let entry=catalog["SchemaPipeline"].as_array().unwrap().iter().find(|entry|entry["name"]=="toJsonSchema").unwrap();
    let example=&entry["examples"][0];
    let directory=support::TempDirectory::new();
    let file=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
    let output=support::run(support::cli().arg("run").arg(file),60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
}
