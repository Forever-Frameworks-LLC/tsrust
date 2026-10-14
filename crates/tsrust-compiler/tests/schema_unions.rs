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
      {const outcome0=schema.safeParse(JSON.parse('{}'));if(outcome0.success){const value=outcome0.data;throw 'Invalid default escaped outer constraint';}else{const error=outcome0.error;for(const issue of error.issues){console.log(issue.code);}}}
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
      {const outcome1=mixed.safeParse(JSON.parse('{"kind":7,"count":11}'));if(outcome1.success){const value=outcome1.data;if(value.kind===7){console.log(value.count);}else{console.log(value.text);}}else{const error=outcome1.error;throw 'Mixed discriminator rejected';}}
      const enums=s.discriminatedUnion('kind',[
        s.object({kind:s.enum(First),first:s.int()}),
        s.object({kind:s.enum(Second),second:s.string()})
      ]);
      {const outcome2=enums.safeParse(JSON.parse('{"kind":"done","second":"nominal"}'));if(outcome2.success){const value=outcome2.data;if(value.kind===Second.Done){console.log(value.second);}else{console.log(value.first);}}else{const error=outcome2.error;throw 'Enum discriminator rejected';}}
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
      {const outcome3=schema.safeParse(JSON.parse('{"kind":9007199254740993,"second":"exact"}'));if(outcome3.success){const value=outcome3.data;if(value.kind===9007199254740993){console.log(value.second);}else{console.log(value.first);}}else{const error=outcome3.error;throw 'Exact discriminator rejected';}}
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
      {const outcome4=schema.safeParse(JSON.parse('{"kind":"b","details":{"count":7},"items":[11]}'));if(outcome4.success){const value=outcome4.data;console.log(value.details.count,value.items[0],value.items.length);if(value.kind==='b'){console.log(value.details.count);}}else{const error=outcome4.error;throw 'Common field input rejected';}}
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
      {const outcome5=strict.safeParse(JSON.parse('{"value":2,"extra":"kept until selected"}'));if(outcome5.success){const value=outcome5.data;throw 'unknown field accepted';}else{const error=outcome5.error;console.log('rejected');}}
      const local=s.union([s.object({value:s.int().min(5)}),s.object({value:s.int()}).strip()]);
      {const outcome6=local.safeParse(JSON.parse('{"value":2,"extra":"discarded"}'));if(outcome6.success){const value=outcome6.data;console.log(value.value);}else{const error=outcome6.error;throw 'branch policy lost';}}
      const outer=s.union([s.object({value:s.int().min(5)}),s.object({value:s.int()})]).strip();
      {const outcome7=outer.safeParse(JSON.parse('{"value":7,"extra":"discarded"}'));if(outcome7.success){const value=outcome7.data;console.log(value.value);}else{const error=outcome7.error;throw 'outer policy lost';}}
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
      {const outcome8=choice.safeParse(JSON.parse('"ready"'));if(outcome8.success){const value=outcome8.data;console.log(scalar(value));}else{const error=outcome8.error;throw 'string rejected';}}
      {const outcome9=choice.safeParse(JSON.parse('7'));if(outcome9.success){const value=outcome9.data;console.log(scalar(value));}else{const error=outcome9.error;throw 'integer rejected';}}
      const owned='owned'.repeat(2);
      {const outcome10=choice.safeParse(owned);if(outcome10.success){const value=outcome10.data;console.log(scalar(value));}else{const error=outcome10.error;throw 'owned rejected';}}
      const repeated=stringChoice(false);
      {const outcome11=repeated.safeParse('aa');if(outcome11.success){const value=outcome11.data;console.log(value);}else{const error=outcome11.error;throw 'second branch rejected';}}
      {const outcome12=repeated.safeParse('abc');if(outcome12.success){const value=outcome12.data;throw 'invalid accepted';}else{const error=outcome12.error;console.log(error.issues.length>0);}}
      {const outcome13=stringChoice(true).safeParse('abcdef');if(outcome13.success){const value=outcome13.data;console.log(value);}else{const error=outcome13.error;throw 'reversed rejected';}}
      const nested=s.array(s.union([s.literal('ready'),s.literal(7)]).optional());
      {const outcome14=nested.safeParse(JSON.parse('["ready",7]'));if(outcome14.success){const values=outcome14.data;console.log(values.length);}else{const error=outcome14.error;throw 'nested rejected';}}
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
      {const outcome15=schema.safeParse(JSON.parse('{"value":2}'));if(outcome15.success){const value=outcome15.data;if(value.tag==='second'){console.log(value.value);}else{throw 'wrong branch';}}else{const error=outcome15.error;throw 'fallback lost input';}}
      {const outcome16=schema.safeParse(JSON.parse('{"value":7}'));if(outcome16.success){const value=outcome16.data;if(value.tag==='first'){console.log(value.value);}else{throw 'wrong order';}}else{const error=outcome16.error;throw 'first branch rejected';}}
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
      {const outcome17=schema.safeParse(JSON.parse('{"kind":"count","count":7}'));if(outcome17.success){const value=outcome17.data;if(value.kind==='count'){const count:int=value.count;console.log(count);}else{console.log(value.text);}}else{const error=outcome17.error;throw 'count rejected';}}
      {const outcome18=schema.safeParse(JSON.parse('{"kind":"text"}'));if(outcome18.success){const value=outcome18.data;if(value.kind==='text'){const selected=value;const text:string=selected.text;console.log(text);}else{throw 'wrong tag';}}else{const error=outcome18.error;throw 'text rejected';}}
      {const outcome19=schema.safeParse(JSON.parse('{"kind":"unknown"}'));if(outcome19.success){const value=outcome19.data;throw 'unknown accepted';}else{const error=outcome19.error;console.log(error.issues.length>0);}}
      {const outcome20=schema.safeParse(JSON.parse('{}'));if(outcome20.success){const value=outcome20.data;throw 'missing accepted';}else{const error=outcome20.error;console.log(error.issues.length>0);}}
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

#[test]
fn direct_union_constraint_chains_preserve_equal_scalar_outputs_and_single_evaluation(){
    let source=r#"
    import {s,Schema} from 'std:schema';
    function branch():Schema<string>{console.log('branch');return s.string();}
    function limit():int{console.log('limit');return 2;}
    function main(){
      const text=s.union([branch(),s.string()]).min(limit()).max(4);
      {const outcome21=text.safeParseJson('"yes"');if(outcome21.success){const value=outcome21.data;console.log(value);}else{const error=outcome21.error;throw 'Valid string rejected';}}
      {const outcome22=text.safeParse('x');if(outcome22.success){const value=outcome22.data;throw 'Minimum ignored';}else{const error=outcome22.error;console.log(error.issues.length);}}
      const integer=s.union([s.int(),s.int()]).gt(1).lt(4);
      {const outcome23=integer.safeParse(2);if(outcome23.success){const value=outcome23.data;console.log(value);}else{const error=outcome23.error;throw 'Integer rejected';}}
      const number=s.union([s.number(),s.number()]).min(1.5).max(2.5);
      {const outcome24=number.safeParse(2.0);if(outcome24.success){const value=outcome24.data;console.log(value);}else{const error=outcome24.error;throw 'Number rejected';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"branch\nlimit\nyes\n1\n2\n2\n");
}

#[test]
fn direct_union_constraint_chain_still_consumes_owned_branch(){
    let source="import {s} from 'std:schema';function main(){const branch=s.string();const schema=s.union([branch,s.string()]).min(2);branch.safeParse('again');}";
    let directory=support::TempDirectory::new();let path=directory.source(source);
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("error[transferred]"),"{}",String::from_utf8_lossy(&output.stderr));
}
