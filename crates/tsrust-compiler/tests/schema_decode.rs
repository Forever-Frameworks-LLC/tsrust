//! Public fused decoding witnesses; run after the complete decoder section lands.
mod support;
fn execute(source:&str)->String {
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn fused_objects_defaults_exact_numbers_retention_and_error_discriminators(){
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.object({name:s.string().min(2),values:s.array(s.int()),count:s.int().default(()=>7)});
      const text='{"name":"bad","name":"good","values":[9007199254740993,-7]}';
      {const outcome0=schema.safeParseJson(text);if(outcome0.success){const value=outcome0.data;if(value.name!=='good'||value.values[0]!==9007199254740993||value.count!==7){throw 'Wrong decoded value';}}else{const error=outcome0.error;throw 'Valid document rejected';}}
      console.log(text.length>0);
      {const outcome1=schema.safeParseJson('{"name":"ok","values":[1,"secret"]}');if(outcome1.success){const value=outcome1.data;throw 'Invalid value accepted';}else{const error=outcome1.error;if(error.kind==='validation'){if(error.error.issues.length!==1){throw 'Wrong issue count';}console.log(error.error.issues[0].code);}else{throw 'Wrong failure kind';}}}
      {const outcome2=schema.safeParseJson('{"name":');if(outcome2.success){const value=outcome2.data;throw 'Malformed document accepted';}else{const error=outcome2.error;if(error.kind==='json'){console.log(error.error.offset>0);}else{throw 'Wrong syntax failure kind';}}}
      const kept=s.object({count:s.int()}).retain();
      {const outcome3=kept.safeParseJson('{"count":2,"extra":[1,2]}');if(outcome3.success){const value=outcome3.data;if(value.value.count!==2||value.extra.size!==1){throw 'Wrong retained result';}}else{const error=outcome3.error;throw 'Retention rejected';}}
      const exact=s.number();
      {const outcome4=exact.safeParseJson('9007199254740993');if(outcome4.success){const value=outcome4.data;throw 'Rounded integer';}else{const error=outcome4.error;if(error.kind==='validation'){console.log(error.error.issues[0].code);}else{throw 'Wrong numeric failure';}}}
    }
    "#;
    assert_eq!(execute(source),"true\ntype-mismatch\ntrue\nnumber-precision\n");
}

#[test]
fn fused_callbacks_prevalidate_syntax_commit_consumption_and_reuse_plans(){
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.object({items:s.array(s.int()).transform(values=>{values.push(9);console.log('transform');return values.length;})});
      {const outcome5=schema.safeParseJson('{"items":[1,2]} trailing');if(outcome5.success){const value=outcome5.data;throw 'Trailing data accepted';}else{const error=outcome5.error;if(error.kind!=='json'){throw 'Wrong trailing error';}}}
      {const outcome6=schema.safeParseJson('{"items":[1,2]}');if(outcome6.success){const value=outcome6.data;console.log(value.items);}else{const error=outcome6.error;throw 'Pipeline rejected';}}
      const choices=s.union([
        s.int().coerce((values:int[]):int=>{console.log('coerce');values.push(1);return values.length;}).refine(value=>value>9),
        s.int().coerce((values:int[]):int=>{console.log('wrong retry');return values.length;})
      ]);
      {const outcome7=choices.safeParseJson('[1]');if(outcome7.success){const value=outcome7.data;throw 'Invalid conversion accepted';}else{const error=outcome7.error;if(error.kind!=='validation'){throw 'Wrong conversion error';}}}
      const fallback=s.union([s.string().refine(value=>value==='no'),s.string().transform(value=>value.length)]);
      {const outcome8=fallback.safeParseJson('"yes"');if(outcome8.success){const value=outcome8.data;console.log(typeof value);}else{const error=outcome8.error;throw 'Readonly fallback rejected';}}
      const throws=s.int().transform(value=>{if(value===1){throw 'callback';}return value;});
      try{throws.safeParseJson('1');throw 'No exception';}catch(error){console.log(error);}
    }
    "#;
    assert_eq!(execute(source),"transform\n3\ncoerce\nnumber\ncallback\n");
}

#[test]
fn fused_catalog_example_executes(){
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let entry=catalog["SchemaPipeline"].as_array().unwrap().iter().find(|entry|entry["name"]=="safeParseJson").unwrap();
    for example in entry["examples"].as_array().unwrap(){assert_eq!(execute(example["source"].as_str().unwrap()),example["output"].as_str().unwrap());}
}

#[test]
fn fused_choices_presence_and_nested_commitment_preserve_plan_identity(){
    let source=r#"
    import {s} from 'std:schema';
    enum Tag{Ready='ready',Done='done'}
    function main(){
      const alternatives=s.union([s.string(),s.string()]);const bounded=alternatives.min(3);
      {const outcome9=bounded.safeParseJson('"x"');if(outcome9.success){const value=outcome9.data;throw 'Outer bound bypassed';}else{const error=outcome9.error;if(error.kind!=='validation'){throw 'Wrong bound error';}}}
      const shape=s.object({tag:s.enum(Tag),mode:s.enum(['a','b']),flag:s.boolean(),empty:s.null(),optional:s.int().optional(),nullable:s.int().nullable()});
      {const outcome10=shape.safeParseJson('{"tag":"ready","mode":"b","flag":true,"empty":null,"nullable":null}');if(outcome10.success){const value=outcome10.data;if(value.tag!==Tag.Ready||value.mode!=='b'||!value.flag||value.empty!==null||value.optional!==undefined||value.nullable!==null){throw 'Wrong presence';}}else{const error=outcome10.error;throw 'Presence rejected';}}
      const choices=s.discriminatedUnion('kind',[s.object({kind:s.literal(9007199254740992),text:s.string()}),s.object({kind:s.literal(9007199254740993),count:s.int()})]);
      {const outcome11=choices.safeParseJson('{"kind":9007199254740993,"count":4}');if(outcome11.success){const value=outcome11.data;if(value.kind===9007199254740993){console.log(value.count);}else{throw 'Wrong branch';}}else{const error=outcome11.error;throw 'Discriminator rejected';}}
      const nested=s.object({first:s.int().transform(value=>value+1),second:s.union([s.literal('no'),s.literal('yes')])});
      {const outcome12=nested.safeParseJson('{"first":1,"second":"yes"}');if(outcome12.success){const value=outcome12.data;console.log(value.first,value.second);}else{const error=outcome12.error;throw 'Earlier field blocked later union selection';}}
      const defaults=s.object({items:s.array(s.int()).transform(values=>values.length).default(()=>{console.log('default');return [1,2];})});
      {const outcome13=defaults.safeParseJson('{}');if(outcome13.success){const value=outcome13.data;console.log(value.items);}else{const error=outcome13.error;throw 'Pipeline default rejected';}}
    }
    "#;
    assert_eq!(execute(source),"4\n2 yes\ndefault\n2\n");
}

#[test]
fn fused_emission_uses_source_cursors_for_typed_fields(){
    let directory=support::TempDirectory::new();let source=directory.source("import {s} from 'std:schema';function main(){const schema=s.object({items:s.array(s.int())});schema.safeParseJson('{\"items\":[1,2]}');}");
    let output=support::run(support::cli().arg("emit").arg(source),60);support::success(&output);
    let native=String::from_utf8(output.stdout).unwrap();
    let body=native.rsplit("fn tsr_function0").next().unwrap();
    assert!(body.contains("source::document")&&body.contains("decode_schema"),"Missing direct decode");
    assert!(!body.contains("tsrust_std_json_parse(")&&!body.contains("validate_json")&&!body.contains("value.retain()"),"Typed decode constructed an intermediate tree");
}

#[test]
fn fused_coercion_rejects_non_json_inputs_during_checking(){
    let directory=support::TempDirectory::new();let source=directory.source("import {s} from 'std:schema';function main(){const schema=s.int().coerce((input:Set<int>):int=>input.size);schema.safeParseJson('[]');}");
    let output=support::run(support::cli().arg("check").arg(source),60);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("schema-json-input"),"{}",String::from_utf8_lossy(&output.stderr));
}

#[test]
fn fused_rust_parser_defaults_precede_callbacks_and_validation_issues_stay_bounded() {
    let source=r#"
    import {s} from 'std:schema';
    function forbiddenCallback():int{throw 'Callback ran before document validation';}
    function rejectedJson(text:string,code:string){
      const schema=s.int().transform(value=>forbiddenCallback());
      {const outcome14=schema.safeParseJson(text);if(outcome14.success){const value=outcome14.data;throw 'Resource violation accepted';}else{const failure=outcome14.error;
          if(failure.kind!=='json'){throw 'Expected JSON boundary error';}
          if(failure.error.code!==code){throw 'Wrong JSON limit code';}
        }}
    }
    function main(){
      rejectedJson('['.repeat(128)+'0'+']'.repeat(128),'invalid-json');
      rejectedJson('0 trailing','trailing-data');
      const scalar=s.int();
      {const large=scalar.safeParseJson(' '.repeat(16777216)+'7');if(!large.success){throw 'Unrequested byte limit';}}
      const ignored=s.object({count:s.int().default(()=>7)}).strip();
      {const deep=ignored.safeParseJson('{"ignored":'+'['.repeat(65)+'0'+']'.repeat(65)+'}');if(!deep.success){throw 'Unrequested depth limit';}}
      {const many=ignored.safeParseJson('{"ignored":['+'0,'.repeat(1000000)+'0]}');if(!many.success){throw 'Unrequested entry or node limit';}}
      const schema=s.array(s.int());
      {const outcome15=schema.safeParseJson('['+'null,'.repeat(80)+'null]');if(outcome15.success){const value=outcome15.data;throw 'Invalid array accepted';}else{const failure=outcome15.error;
          if(failure.kind!=='validation'){throw 'Wrong error domain';}
          if(!failure.error.truncated||failure.error.issues.length!==64){throw 'Issue budget bypassed';}
          if(failure.error.issues[63].path[0].index!==63){throw 'Issue order changed';}
          if(failure.error.issues[0].observed!=='null'){throw 'Issue contains wrong type';}
        }}
      const defaults=s.object({count:s.int().default(()=>forbiddenCallback())}).strip();
      {const outcome16=defaults.safeParseJson('{"ignored":"\\ud800"}');if(outcome16.success){const value=outcome16.data;throw 'Invalid ignored Unicode accepted';}else{const failure=outcome16.error;if(failure.kind!=='json'){throw 'Wrong ignored error domain';}}}
    }
    "#;
    assert_eq!(execute(source), "");
}
