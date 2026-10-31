//! Public pipeline witnesses; execute after the complete pipeline section lands.
mod support;

#[test]
fn chained_json_refinements_restore_input_before_consuming_fallback() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const checked=s.array(s.int()).refine(values=>values.length>0).refine(values=>values[0]===9);
      const schema=s.union([
        checked.transform(values=>0),
        s.array(s.int()).transform(values=>{values.push(3);return values.length;})
      ]);
      const first=schema.safeParse(JSON.parse('[1,2]'));
      if(first.success){console.log(first.data);}else{throw 'Fallback lost input';}
      const second=schema.safeParse(JSON.parse('[9]'));
      if(second.success){console.log(second.data);}else{throw 'Refinement lost input';}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "3\n0\n");
}

#[test]
fn coercion_exceptions_propagate_and_skip_later_stages() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.int().coerce((values:int[]):int=>{
        if(values.length===0){throw 'conversion failed';}return values.length;
      }).transform((count:int):int=>{console.log('transformed');return count;});
      try {
        schema.safeParse([]);
        throw 'Expected callback exception';
      } catch(error) {
        if(error!=='conversion failed'){throw 'Wrong callback exception';}
      }
      {const outcome0=schema.safeParse([4]);if(outcome0.success){const value=outcome0.data;if(value!==1){throw 'Wrong reusable output';}}else{const error=outcome0.error;throw 'Reusable callback rejected';}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "transformed\n");
}

#[test]
fn coercion_consumes_raw_input_before_validation_and_preserves_chain_types() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const converted=s.int().min(3).coerce((values:int[]):int=>{
        values.push(9);return values.length;
      });
      const checked=converted.refine(count=>count===3);
      const output=checked.transform(count=>{
        if(count===3){return 'three';}return 'unexpected';
      });
      {const outcome1=output.safeParse([1,2]);if(outcome1.success){const value=outcome1.data;const text:string=value;if(text!=='three'){throw 'Wrong output';}}else{const error=outcome1.error;throw 'Valid conversion rejected';}}
      {const outcome2=output.safeParse([]);if(outcome2.success){const value=outcome2.data;throw 'Converted value bypassed minimum';}else{const error=outcome2.error;if(error.issues.length===0){throw 'Missing validation issue';}}}
      {const outcome3=output.safeParse([4,5]);if(outcome3.success){const value=outcome3.data;if(value!=='three'){throw 'Reusable plan changed';}}else{const error=outcome3.error;throw 'Reusable plan rejected';}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn consuming_coercion_commits_union_before_converted_value_validation() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.int().min(10).coerce((values:int[]):int=>{
          values.push(7);console.log('converted');return values.length;
        }),
        s.int().coerce((values:int[]):int=>{
          console.log('wrong retry');return values.length;
        })
      ]);
      {const outcome4=schema.safeParse([1]);if(outcome4.success){const value=outcome4.data;throw 'Invalid converted value accepted';}else{const error=outcome4.error;if(error.issues.length===0){throw 'Missing issue';}}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "converted\n");
}

#[test]
fn schema_pipeline_tracks_changed_output_and_mutates_only_owned_input() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const input=s.array(s.int()).refine((values:int[]):boolean=>values.length>0);
      const transformed=input.transform((values:int[]):int=>{values.push(7);return values.length;});
      const checked=transformed.refine((count:int):boolean=>count===3);
      {const outcome5=checked.safeParse([1,2]);if(outcome5.success){const value=outcome5.data;const count:int=value;console.log(count);}else{const error=outcome5.error;throw 'Owned pipeline rejected';}}
      {const outcome6=checked.safeParse(JSON.parse('[3,4]'));if(outcome6.success){const value=outcome6.data;console.log(value);}else{const error=outcome6.error;throw 'JSON pipeline rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"3\n3\n");
}

#[test]
fn union_pipeline_does_not_retry_after_a_consuming_transform() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.array(s.int()).transform((values:int[]):string=>{values.push(7);console.log('committed');return 'first';}).refine((value:string):boolean=>false),
        s.array(s.int()).transform((values:int[]):string=>{console.log('wrong retry');return 'second';})
      ]);
      {const outcome7=schema.safeParse([1,2]);if(outcome7.success){const value=outcome7.data;throw 'Failed refinement accepted';}else{const error=outcome7.error;console.log(error.issues.length>0);}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"committed\ntrue\n");
}

#[test]
fn failed_pretransform_refinement_skips_callbacks_and_allows_next_union_branch() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.array(s.int()).refine((values:int[]):boolean=>values.length>3).transform((values:int[]):int=>{console.log('wrong transform');return values.length;}),
        s.array(s.int()).transform((values:int[]):int=>{console.log('selected');return values.length;})
      ]);
      {const outcome8=schema.safeParse([1,2]);if(outcome8.success){const value=outcome8.data;console.log(value);}else{const error=outcome8.error;throw 'Fallback rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"selected\n2\n");
}

#[test]
fn schema_pipeline_rejects_stale_types_input_reuse_and_mutating_captures() {
    for source in [
        "import {s} from 'std:schema';function main(){const schema=s.string().transform((value:string):int=>value.length);schema.refine((value:string):boolean=>value.length>0);}",
        "import {s} from 'std:schema';function main(){const schema=s.array(s.int()).transform((value:int[]):int=>value.length);schema.safeParse('wrong input');}",
        "import {s} from 'std:schema';function main(){const schema=s.array(s.int()).transform((value:int[]):int=>value.length);const input=[1,2];schema.safeParse(input);console.log(input.length);}",
        "import {s} from 'std:schema';function main(){let calls=0;const schema=s.int().transform((value:int):int=>{calls++;return value;});schema.safeParse(1);}",
        "import {s} from 'std:schema';function main(){const schema=s.array(s.int()).refine((value:int[]):boolean=>{value.push(7);return true;});schema.safeParse([1]);}",
    ]{
        let directory=support::TempDirectory::new();let path=directory.source(source);
        let output=support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success(),"accepted {source}");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn array_schema_retains_each_child_pipeline_input_and_output() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.array(s.string().transform(value=>value.length));
      {const outcome9=schema.safeParse(['a','three']);if(outcome9.success){const value=outcome9.data;const sizes:int[]=value;if(sizes.length!==2||sizes[0]!==1||sizes[1]!==5){throw 'Wrong owned array output';}}else{const error=outcome9.error;throw 'Owned array rejected';}}
      {const outcome10=schema.safeParse(JSON.parse('["hi","four"]'));if(outcome10.success){const value=outcome10.data;if(value[0]!==2||value[1]!==4){throw 'Wrong JSON array output';}}else{const error=outcome10.error;throw 'JSON array rejected';}}
      const nested=s.array(s.array(s.int().transform(value=>value>0)));
      {const outcome11=nested.safeParse([[1,0]]);if(outcome11.success){const value=outcome11.data;const flags:boolean[][]=value;if(flags.length!==1){throw 'Wrong nested output';}}else{const error=outcome11.error;throw 'Nested array rejected';}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn object_schema_keeps_transformed_fields_defaults_and_unknown_field_rejection() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.object({
        size:s.string().default(()=>'ready').transform(value=>value.length),
        flags:s.array(s.int().transform(value=>value>0))
      });
      {const outcome12=schema.safeParse({size:'abc',flags:[1,0]});if(outcome12.success){const value=outcome12.data;const size:int=value.size;if(size!==3||value.flags.length!==2){throw 'Wrong owned fields';}}else{const error=outcome12.error;throw 'Owned object rejected';}}
      {const outcome13=schema.safeParse(JSON.parse('{"flags":[1,0]}'));if(outcome13.success){const value=outcome13.data;if(value.size!==5){throw 'Default not transformed';}}else{const error=outcome13.error;throw 'Missing default rejected';}}
      {const outcome14=schema.safeParse(JSON.parse('{"size":"abc","flags":[],"extra":true}'));if(outcome14.success){const value=outcome14.data;throw 'Unknown field accepted';}else{const error=outcome14.error;if(error.issues.length===0){throw 'Missing unknown field issue';}}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn transformed_object_policies_strip_or_retain_unknown_json_data() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const stripped=s.object({size:s.string().transform(text=>text.length)}).strip();
      {const outcome15=stripped.safeParse(JSON.parse('{"size":"abc","other":7}'));if(outcome15.success){const value=outcome15.data;if(value.size!==3){throw 'Wrong stripped value';}}else{const error=outcome15.error;throw 'Strip rejected extra data';}}
      const retained=s.object({size:s.string().transform(text=>text.length)}).retain();
      {const outcome16=retained.safeParse(JSON.parse('{"size":"four","other":7}'));if(outcome16.success){const value=outcome16.data;if(value.value.size!==4||!value.extra.has('other')||value.extra.has('size')){throw 'Wrong retained fields';}}else{const error=outcome16.error;throw 'Retain rejected extra data';}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn rejected_json_projection_preserves_unknown_fields_missing_fields_and_exact_integers() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.object({label:s.string().default(()=>{console.log('default once');return 'generated';}),exact:s.int()})
          .strip().refine(value=>false).transform(value=>0),
        s.object({exact:s.int()}).retain().transform(value=>{
          if(value.value.exact!==9223372036854775807){throw 'Exact integer changed';}
          if(value.extra.has('label')||!value.extra.has('other')){throw 'Original fields changed';}
          const extra=value.extra.get('other');
          if(extra===undefined){throw 'Unknown value lost';}
          if(JSON.stringify(extra)!=='["preserved"]'){throw 'Unknown payload changed';}
          return 1;
        })
      ]);
      {const outcome17=schema.safeParse(JSON.parse('{"exact":9223372036854775807,"other":["preserved"]}'));if(outcome17.success){const value=outcome17.data;if(value!==1){throw 'Wrong branch';}}else{const error=outcome17.error;throw 'Restored alternative rejected';}}
    }
    "#;
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "default once\n");
}

#[test]
fn discriminated_pipeline_selects_input_tags_before_transforming_to_scalar() {
    let source = r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.discriminatedUnion('kind',[
        s.object({kind:s.literal('text'),text:s.string()}).transform(value=>value.text.length),
        s.object({kind:s.literal('count'),count:s.int()}).transform(value=>value.count)
      ]);
      {const outcome18=schema.safeParse(JSON.parse('{"kind":"text","text":"hello"}'));if(outcome18.success){const value=outcome18.data;const count:int=value;if(count!==5){throw 'Wrong transformed text';}}else{const error=outcome18.error;throw 'Discriminated transformed branch rejected';}}
      {const outcome19=schema.safeParse(JSON.parse('{"kind":"count","count":7}'));if(outcome19.success){const value=outcome19.data;if(value!==7){throw 'Wrong transformed count';}}else{const error=outcome19.error;throw 'Discriminated count rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn nested_readonly_array_pipeline_restores_json_before_union_fallback() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.array(s.string().refine(value=>value.length>2)).transform(values=>{console.log('wrong transform');return 0;}),
        s.array(s.string()).transform(values=>{return values.length;})
      ]);
      {const outcome20=schema.safeParse(JSON.parse('["first","x","last"]'));if(outcome20.success){const value=outcome20.data;if(value!==3){throw 'Array fallback lost elements';}}else{const error=outcome20.error;throw 'Array fallback rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn nested_object_pipeline_restores_prior_fields_and_unknown_data() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.union([
        s.object({first:s.string().refine(value=>value.length>0),second:s.int().refine(value=>false)}).strip().transform(value=>{console.log('wrong transform');return 0;}),
        s.object({first:s.string(),second:s.int()}).retain().transform(value=>{
          if(value.value.first!=='original'){throw 'First field lost';}
          if(value.value.second!==9223372036854775807){throw 'Exact integer lost';}
          if(value.extra.size!==1){throw 'Unknown field lost';}
          return 1;
        })
      ]);
      {const outcome21=schema.safeParse(JSON.parse('{"first":"original","second":9223372036854775807,"extra":[1,2]}'));if(outcome21.success){const value=outcome21.data;if(value!==1){throw 'Wrong branch output';}}else{const error=outcome21.error;throw 'Object fallback rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn pipeline_defaults_validate_raw_input_preserve_omission_and_do_not_hide_failures() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.object({count:s.string().min(2).transform(value=>value.length).default(()=>{
        console.log('factory');return 'hello';
      }).optional()});
      {const outcome22=schema.safeParse({});if(outcome22.success){const value=outcome22.data;console.log(value.count);}else{const error=outcome22.error;throw 'Owned omission rejected';}}
      {const outcome23=schema.safeParse(JSON.parse('{}'));if(outcome23.success){const value=outcome23.data;console.log(value.count);}else{const error=outcome23.error;throw 'JSON omission rejected';}}
      {const outcome24=schema.safeParse(JSON.parse('{"count":"supplied"}'));if(outcome24.success){const value=outcome24.data;console.log(value.count);}else{const error=outcome24.error;throw 'Supplied input rejected';}}
      {const outcome25=schema.safeParse(JSON.parse('{"count":null}'));if(outcome25.success){const value=outcome25.data;throw 'Null invoked default';}else{const error=outcome25.error;console.log('null rejected');}}
      const invalid=s.object({count:s.string().min(2).transform(value=>value.length).default(()=> 'x').optional()});
      {const outcome26=invalid.safeParse({});if(outcome26.success){const value=outcome26.data;throw 'Optional hid invalid default';}else{const error=outcome26.error;console.log('invalid default');}}
      const absent=s.object({count:s.string().transform(value=>value.length).optional()});
      {const outcome27=absent.safeParse({});if(outcome27.success){const value=outcome27.data;console.log(value.count);}else{const error=outcome27.error;throw 'Optional omission rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"factory\n5\nfactory\n5\n8\nnull rejected\ninvalid default\nundefined\n");
}

#[test]
fn pipeline_catalog_examples_execute_exact_sources() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    for entry in catalog["SchemaPipeline"].as_array().unwrap(){
        let examples=entry["examples"].as_array().expect("every pipeline method has executable examples");
        assert!(!examples.is_empty());
        for example in examples {
            let directory=support::TempDirectory::new();
            let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
            let output=support::run(support::cli().arg("run").arg(path),60);
            support::success(&output);
            assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap(),"{}",entry["name"]);
        }
    }
}

#[test]
fn owned_record_rollback_restores_preconversion_fields_and_drops_synthetic_defaults() {
    let source=r#"
    import {s} from 'std:schema';
    import {strictEqual} from 'std:testing';
    type Input={text:string;count:int;};
    function main(){
      const schema=s.union([
        s.object({text:s.string(),count:s.number(),label:s.string().default(()=>{console.log('default');return 'generated';})})
          .refine(value=>value.label==='reject').transform(value=>0),
        s.object({text:s.string(),count:s.int()}).transform(value=>{
          strictEqual(value.text,'original');strictEqual(value.count,9007199254740992);return 1;
        })
      ]);
      const raw:Input={text:'original'.repeat(1),count:9007199254740992};
      {const outcome28=schema.safeParse(raw);if(outcome28.success){const value=outcome28.data;strictEqual(value,1);}else{const error=outcome28.error;throw 'Owned rollback rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"default\n");
}

#[test]
fn runtime_validation_error_is_public_and_distinct_from_construction_error() {
    let source = r#"
    import {s, ValidationError as InputFailure, SchemaConstructionError} from 'std:schema';
    function report(error: InputFailure): int {
      if(error.truncated){throw 'Unexpected truncation';}
      return error.issues.length;
    }
    function main(){
      const scalar=s.int().min(3);
      {const outcome29=scalar.safeParse(1);if(outcome29.success){const value=outcome29.data;throw 'Invalid value accepted';}else{const error=outcome29.error;if(report(error)!==1){throw 'Wrong issue count';}}}
      const pipeline=s.string().transform(text=>text.length).refine(length=>length>2);
      {const outcome30=pipeline.safeParse('x');if(outcome30.success){const value=outcome30.data;throw 'Invalid transformed value accepted';}else{const error=outcome30.error;if(report(error)!==1){throw 'Wrong pipeline error';}}}
      const definition:SchemaConstructionError={code:'contradictory-bounds',operation:'min',limit:3};
      if(definition.operation!=='min'){throw 'Construction error changed';}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
}
