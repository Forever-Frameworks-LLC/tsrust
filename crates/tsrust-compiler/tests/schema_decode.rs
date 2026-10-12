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
      match(schema.safeParseJson(text)){
        Ok(value)=>{if(value.name!=='good'||value.values[0]!==9007199254740993||value.count!==7){throw 'Wrong decoded value';}},
        Err(error)=>{throw 'Valid document rejected';}
      }
      console.log(text.length>0);
      match(schema.safeParseJson('{"name":"ok","values":[1,"secret"]}')){
        Ok(value)=>{throw 'Invalid value accepted';},
        Err(error)=>{if(error.kind==='validation'){if(error.error.issues.length!==1){throw 'Wrong issue count';}console.log(error.error.issues[0].code);}else{throw 'Wrong failure kind';}}
      }
      match(schema.safeParseJson('{"name":')){
        Ok(value)=>{throw 'Malformed document accepted';},
        Err(error)=>{if(error.kind==='json'){console.log(error.error.offset>0);}else{throw 'Wrong syntax failure kind';}}
      }
      const kept=s.object({count:s.int()}).retain();
      match(kept.safeParseJson('{"count":2,"extra":[1,2]}')){
        Ok(value)=>{if(value.value.count!==2||value.extra.size!==1){throw 'Wrong retained result';}},
        Err(error)=>{throw 'Retention rejected';}
      }
      const exact=s.number();
      match(exact.safeParseJson('9007199254740993')){
        Ok(value)=>{throw 'Rounded integer';},
        Err(error)=>{if(error.kind==='validation'){console.log(error.error.issues[0].code);}else{throw 'Wrong numeric failure';}}
      }
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
      match(schema.safeParseJson('{"items":[1,2]} trailing')){
        Ok(value)=>{throw 'Trailing data accepted';},Err(error)=>{if(error.kind!=='json'){throw 'Wrong trailing error';}}
      }
      match(schema.safeParseJson('{"items":[1,2]}')){Ok(value)=>{console.log(value.items);},Err(error)=>{throw 'Pipeline rejected';}}
      const choices=s.union([
        s.int().coerce((values:int[]):int=>{console.log('coerce');values.push(1);return values.length;}).refine(value=>value>9),
        s.int().coerce((values:int[]):int=>{console.log('wrong retry');return values.length;})
      ]);
      match(choices.safeParseJson('[1]')){Ok(value)=>{throw 'Invalid conversion accepted';},Err(error)=>{if(error.kind!=='validation'){throw 'Wrong conversion error';}}}
      const fallback=s.union([s.string().refine(value=>value==='no'),s.string().transform(value=>value.length)]);
      match(fallback.safeParseJson('"yes"')){Ok(value)=>{console.log(typeof value);},Err(error)=>{throw 'Readonly fallback rejected';}}
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
      match(bounded.safeParseJson('"x"')){Ok(value)=>{throw 'Outer bound bypassed';},Err(error)=>{if(error.kind!=='validation'){throw 'Wrong bound error';}}}
      const shape=s.object({tag:s.enum(Tag),mode:s.enum(['a','b']),flag:s.boolean(),empty:s.null(),optional:s.int().optional(),nullable:s.int().nullable()});
      match(shape.safeParseJson('{"tag":"ready","mode":"b","flag":true,"empty":null,"nullable":null}')){
        Ok(value)=>{if(value.tag!==Tag.Ready||value.mode!=='b'||!value.flag||value.empty!==null||value.optional!==undefined||value.nullable!==null){throw 'Wrong presence';}},Err(error)=>{throw 'Presence rejected';}
      }
      const choices=s.discriminatedUnion('kind',[s.object({kind:s.literal(9007199254740992),text:s.string()}),s.object({kind:s.literal(9007199254740993),count:s.int()})]);
      match(choices.safeParseJson('{"kind":9007199254740993,"count":4}')){
        Ok(value)=>{if(value.kind===9007199254740993){console.log(value.count);}else{throw 'Wrong branch';}},Err(error)=>{throw 'Discriminator rejected';}
      }
      const nested=s.object({first:s.int().transform(value=>value+1),second:s.union([s.literal('no'),s.literal('yes')])});
      match(nested.safeParseJson('{"first":1,"second":"yes"}')){Ok(value)=>{console.log(value.first,value.second);},Err(error)=>{throw 'Earlier field blocked later union selection';}}
      const defaults=s.object({items:s.array(s.int()).transform(values=>values.length).default(()=>{console.log('default');return [1,2];})});
      match(defaults.safeParseJson('{}')){Ok(value)=>{console.log(value.items);},Err(error)=>{throw 'Pipeline default rejected';}}
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
fn fused_limits_precede_callbacks_and_validation_issues_stay_bounded() {
    let source=r#"
    import {s} from 'std:schema';
    function forbiddenCallback():int{throw 'Callback ran before document validation';}
    function rejectedJson(text:string,code:string){
      const schema=s.int().transform(value=>forbiddenCallback());
      match(schema.safeParseJson(text)){
        Ok(value)=>{throw 'Resource violation accepted';},
        Err(failure)=>{
          if(failure.kind!=='json'){throw 'Expected JSON boundary error';}
          if(failure.error.code!==code){throw 'Wrong JSON limit code';}
        }
      }
    }
    function main(){
      rejectedJson(' '.repeat(16777216)+'0','byte-limit');
      rejectedJson('['.repeat(65)+'0'+']'.repeat(65),'depth-limit');
      rejectedJson('['+'0,'.repeat(999999)+'0]','node-limit');
      const schema=s.array(s.int());
      match(schema.safeParseJson('['+'null,'.repeat(80)+'null]')){
        Ok(value)=>{throw 'Invalid array accepted';},
        Err(failure)=>{
          if(failure.kind!=='validation'){throw 'Wrong error domain';}
          if(!failure.error.truncated||failure.error.issues.length!==64){throw 'Issue budget bypassed';}
          if(failure.error.issues[63].path[0].index!==63){throw 'Issue order changed';}
          if(failure.error.issues[0].observed!=='null'){throw 'Issue contains wrong type';}
        }
      }
      const defaults=s.object({count:s.int().default(()=>forbiddenCallback())}).strip();
      match(defaults.safeParseJson('{"ignored":"\\ud800"}')){
        Ok(value)=>{throw 'Invalid ignored Unicode accepted';},
        Err(failure)=>{if(failure.kind!=='json'){throw 'Wrong ignored error domain';}}
      }
    }
    "#;
    assert_eq!(execute(source), "");
}
