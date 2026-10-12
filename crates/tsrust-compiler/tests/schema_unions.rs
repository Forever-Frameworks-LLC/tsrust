//! Section witnesses: run after ordered/discriminated union integration is complete.
mod support;

#[test]
fn union_outer_constraints_validate_selected_defaults_without_rerunning_factories() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const alternatives=s.union([
        s.string().default(()=>{console.log('first');return 'a';}),
        s.string().default(()=>{console.log('second');return 'long';})
      ]);
      const schema=s.object({value:alternatives.min(3)});
      match(schema.safeParse(JSON.parse('{}'))){
        Ok(value)=>{throw 'Invalid default escaped outer constraint';},
        Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"first\ntoo-small\n");
}

#[test]
fn discriminator_access_preserves_mixed_scalar_and_nominal_enum_alternatives() {
    let source=r#"
    import {s} from 'std:schema';
    enum First{Ready='ready'}enum Second{Done='done'}
    function main(){
      const mixed=s.discriminatedUnion('kind',[
        s.object({kind:s.literal('text'),text:s.string()}),
        s.object({kind:s.literal(7),count:s.int()})
      ]);
      match(mixed.safeParse(JSON.parse('{"kind":7,"count":11}'))){
        Ok(value)=>{if(value.kind===7){console.log(value.count);}else{console.log(value.text);}},
        Err(error)=>{throw 'Mixed discriminator rejected';}
      }
      const enums=s.discriminatedUnion('kind',[
        s.object({kind:s.enum(First),first:s.int()}),
        s.object({kind:s.enum(Second),second:s.string()})
      ]);
      match(enums.safeParse(JSON.parse('{"kind":"done","second":"nominal"}'))){
        Ok(value)=>{if(value.kind===Second.Done){console.log(value.second);}else{console.log(value.first);}},
        Err(error)=>{throw 'Enum discriminator rejected';}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"11\nnominal\n");
}

#[test]
fn union_schema_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut seen=std::collections::HashSet::new();
    for entry in catalog["schemaBuilder"].as_array().unwrap().iter().filter(|entry|matches!(entry["name"].as_str(),Some("union"|"discriminatedUnion"))).chain(catalog["SchemaUnion"].as_array().unwrap().iter()){
        for example in entry["examples"].as_array().into_iter().flatten(){
            let source=example["source"].as_str().unwrap();if !seen.insert(source){continue;}
            let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
            let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
            assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
        }
    }
    assert!(seen.len()>=2);
}

#[test]
fn discriminated_integer_tags_keep_exact_values_above_float_precision() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.discriminatedUnion('kind',[
        s.object({kind:s.literal(9007199254740992),first:s.int()}),
        s.object({kind:s.literal(9007199254740993),second:s.string()})
      ]);
      match(schema.safeParse(JSON.parse('{"kind":9007199254740993,"second":"exact"}'))){
        Ok(value)=>{if(value.kind===9007199254740993){console.log(value.second);}else{console.log(value.first);}},
        Err(error)=>{throw 'Exact discriminator rejected';}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"exact\n");
}

#[test]
fn record_discriminator_fact_is_invalidated_by_replacing_the_union() {
    let source=r#"
    type A={kind:'a';first:int;};type B={kind:'b';second:int;};
    function choose(first:boolean):A|B{if(first){return {kind:'a',first:1};}return {kind:'b',second:2};}
    function main(){let value=choose(true);if(value.kind==='a'){value=choose(false);console.log(value.first);}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(source);
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let error=String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("has no field")&&error.contains("first"),"{error}");
}

#[test]
fn common_record_union_fields_borrow_nested_records_and_arrays() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const schema=s.discriminatedUnion('kind',[
        s.object({kind:s.literal('a'),details:s.object({count:s.int()}),items:s.array(s.int())}),
        s.object({kind:s.literal('b'),details:s.object({count:s.int()}),items:s.array(s.int())})
      ]);
      match(schema.safeParse(JSON.parse('{"kind":"b","details":{"count":7},"items":[11]}'))){
        Ok(value)=>{console.log(value.details.count,value.items[0],value.items.length);if(value.kind==='b'){console.log(value.details.count);}},
        Err(error)=>{throw 'Common field input rejected';}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"7 11 1\n7\n");
}

#[test]
fn same_output_union_preserves_branch_policies_and_outer_strip_override() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const strict=s.union([s.object({value:s.int().min(5)}),s.object({value:s.int()})]);
      match(strict.safeParse(JSON.parse('{"value":2,"extra":"kept until selected"}'))){
        Ok(value)=>{throw 'unknown field accepted';},Err(error)=>{console.log('rejected');}
      }
      const local=s.union([s.object({value:s.int().min(5)}),s.object({value:s.int()}).strip()]);
      match(local.safeParse(JSON.parse('{"value":2,"extra":"discarded"}'))){
        Ok(value)=>{console.log(value.value);},Err(error)=>{throw 'branch policy lost';}
      }
      const outer=s.union([s.object({value:s.int().min(5)}),s.object({value:s.int()})]).strip();
      match(outer.safeParse(JSON.parse('{"value":7,"extra":"discarded"}'))){
        Ok(value)=>{console.log(value.value);},Err(error)=>{throw 'outer policy lost';}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"rejected\n2\n7\n");
}

#[test]
fn ordered_unions_preserve_runtime_branches_output_types_and_owned_inputs() {
    let source=r#"
    import {s,Schema} from 'std:schema';
    function stringChoice(reverse:boolean):Schema<string>{
      if(reverse){return s.union([s.string().max(2),s.string().min(5)]);}
      return s.union([s.string().min(5),s.string().max(2)]);
    }
    function scalar(value:string|int):string{return String(value);}
    function main(){
      const choice=s.union([s.string().min(3),s.int().min(2)]);
      match(choice.safeParse(JSON.parse('"ready"'))){Ok(value)=>{console.log(scalar(value));},Err(error)=>{throw 'string rejected';}}
      match(choice.safeParse(JSON.parse('7'))){Ok(value)=>{console.log(scalar(value));},Err(error)=>{throw 'integer rejected';}}
      const owned='owned'.repeat(2);
      match(choice.safeParse(owned)){Ok(value)=>{console.log(scalar(value));},Err(error)=>{throw 'owned rejected';}}
      const repeated=stringChoice(false);
      match(repeated.safeParse('aa')){Ok(value)=>{console.log(value);},Err(error)=>{throw 'second branch rejected';}}
      match(repeated.safeParse('abc')){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{console.log(error.issues.length>0);}}
      match(stringChoice(true).safeParse('abcdef')){Ok(value)=>{console.log(value);},Err(error)=>{throw 'reversed rejected';}}
      const nested=s.array(s.union([s.literal('ready'),s.literal(7)]).optional());
      match(nested.safeParse(JSON.parse('["ready",7]'))){Ok(values)=>{console.log(values.length);},Err(error)=>{throw 'nested rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\n7\nownedowned\naa\ntrue\nabcdef\n2\n");
}

#[test]
fn ordered_object_branches_prepare_defaults_once_and_keep_original_fields() {
    let source=r#"
    import {s} from 'std:schema';
    function first():'first'{console.log('first factory');return 'first';}
    function second():'second'{console.log('second factory');return 'second';}
    function main(){
      const schema=s.union([
        s.object({value:s.int().min(5),tag:s.literal('first').default(first)}),
        s.object({value:s.int(),tag:s.literal('second').default(second)})
      ]);
      match(schema.safeParse(JSON.parse('{"value":2}'))){
        Ok(value)=>{if(value.tag==='second'){console.log(value.value);}else{throw 'wrong branch';}},
        Err(error)=>{throw 'fallback lost input';}
      }
      match(schema.safeParse(JSON.parse('{"value":7}'))){
        Ok(value)=>{if(value.tag==='first'){console.log(value.value);}else{throw 'wrong order';}},
        Err(error)=>{throw 'first branch rejected';}
      }
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"first factory\nsecond factory\n2\nfirst factory\n7\n");
}

#[test]
fn discriminated_unions_select_one_branch_and_narrow_its_typed_fields() {
    let source=r#"
    import {s} from 'std:schema';
    function fallback():string{console.log('selected factory');return 'generated';}
    function main(){
      const schema=s.discriminatedUnion('kind',[
        s.object({kind:s.literal('text'),text:s.string().default(fallback)}),
        s.object({kind:s.literal('count'),count:s.int()})
      ]);
      match(schema.safeParse(JSON.parse('{"kind":"count","count":7}'))){
        Ok(value)=>{if(value.kind==='count'){const count:int=value.count;console.log(count);}else{console.log(value.text);}},
        Err(error)=>{throw 'count rejected';}
      }
      match(schema.safeParse(JSON.parse('{"kind":"text"}'))){
        Ok(value)=>{if(value.kind==='text'){const selected=value;const text:string=selected.text;console.log(text);}else{throw 'wrong tag';}},
        Err(error)=>{throw 'text rejected';}
      }
      match(schema.safeParse(JSON.parse('{"kind":"unknown"}'))){Ok(value)=>{throw 'unknown accepted';},Err(error)=>{console.log(error.issues.length>0);}}
      match(schema.safeParse(JSON.parse('{}'))){Ok(value)=>{throw 'missing accepted';},Err(error)=>{console.log(error.issues.length>0);}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"7\nselected factory\ngenerated\ntrue\ntrue\n");
}

#[test]
fn union_schemas_reject_empty_branches_reused_plans_and_input_reuse() {
    for source in [
      "import {s} from 'std:schema';function main(){s.union([]);}",
      "import {s} from 'std:schema';function main(){const child=s.string();s.union([child,child]);}",
      "import {s} from 'std:schema';function main(){const schema=s.union([s.string(),s.int()]);const input='owned';schema.safeParse(input);console.log(input);}",
      "import {s} from 'std:schema';function main(){s.discriminatedUnion('tag',[s.object({tag:s.literal('same')}),s.object({tag:s.literal('same')})]);}",
    ] {
      let directory=support::TempDirectory::new();let path=directory.source(source);
      let output=support::run(support::cli().arg("check").arg(path),60);
      assert!(!output.status.success(),"accepted: {source}");
      assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"),"{}",String::from_utf8_lossy(&output.stderr));
    }
}
