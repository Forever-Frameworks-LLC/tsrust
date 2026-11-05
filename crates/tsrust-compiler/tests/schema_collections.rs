//! Collection section witnesses; run after checker/native/editor integration lands.
mod support;

#[test]
fn collection_schemas_preserve_typed_output_paths_and_unique_values() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const dictionary=s.record(s.int().min(0));
      {const outcome0=dictionary.safeParseJson('{"alice":10,"bob":7}');if(outcome0.success){const values=outcome0.data;if(values.size!==2||values.get('alice')!==10){throw 'Wrong dictionary';}}else{const error=outcome0.error;throw 'Dictionary rejected';}}
      {const outcome1=dictionary.safeParseJson('{"alice":-1}');if(outcome1.success){const values=outcome1.data;throw 'Negative dictionary entry accepted';}else{const failure=outcome1.error;if(failure.kind==='validation'){
          if(failure.error.issues[0].path[0].field!=='alice'){throw 'Wrong dictionary path';}
        }else{throw 'Wrong dictionary error';}}}
      const input=new Map<int,string>();input.set(7,'seven');
      const map=s.map(s.int().min(0),s.string().min(1));
      {const outcome2=map.safeParse(input);if(outcome2.success){const values=outcome2.data;if(values.get(7)!=='seven'){throw 'Wrong map';}}else{const error=outcome2.error;throw 'Map rejected';}}
      const tags=new Set<string>();tags.add('admin');tags.add('editor');
      const set=s.set(s.string().min(1));
      {const outcome3=set.safeParse(tags);if(outcome3.success){const values=outcome3.data;if(values.size!==2||!values.has('admin')){throw 'Wrong set';}}else{const error=outcome3.error;throw 'Set rejected';}}
      const collisions=new Map<string,int>();collisions.set('A',1);collisions.set('a',2);
      const lowered=s.map(s.string().transform(value=>value.toLowerCase()),s.int());
      {const outcome4=lowered.safeParse(collisions);if(outcome4.success){const values=outcome4.data;throw 'Colliding map keys accepted';}else{const error=outcome4.error;if(error.issues.length===0){throw 'Missing collision issue';}}}
      const elements=new Set<string>();elements.add('A');elements.add('a');
      const loweredSet=s.set(s.string().transform(value=>value.toLowerCase()));
      {const outcome5=loweredSet.safeParse(elements);if(outcome5.success){const values=outcome5.data;throw 'Colliding set elements accepted';}else{const error=outcome5.error;if(error.issues.length===0){throw 'Missing set collision issue';}}}
    }
    "#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
}

#[test]
fn collection_catalog_examples_execute_together() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut source=String::from("import {s} from 'std:schema';\n");
    let mut expected=String::new();
    for (index,name) in ["record","map","set"].into_iter().enumerate(){
        let entry=catalog["schemaBuilder"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();
        let example=&entry["examples"][0];
        source.push_str(&example["source"].as_str().unwrap().replace("import {s} from 'std:schema';","").replace("function main()",&format!("function example{index}()")));
        expected.push_str(example["output"].as_str().unwrap());
    }
    source.push_str("function main(){example0();example1();example2();}");
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(&source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),expected);
}

#[test]
fn native_collection_schemas_reject_implicit_json_and_input_reuse() {
    for (body,code) in [
        ("const schema=s.map(s.string(),s.int());schema.safeParseJson('{}');","schema-json-input"),
        ("const schema=s.set(s.int());schema.safeParseJson('[]');","schema-json-input"),
        ("const schema=s.array(s.set(s.int()));schema.safeParseJson('[[]]');","schema-json-input"),
        ("const schema=s.map(s.string(),s.int());const input=new Map<string,int>();schema.safeParse(input);console.log(input.size);","transferred"),
        ("const child=s.int();const schema=s.record(child);child.safeParse(1);","transferred"),
        ("const schema=s.set(s.int());const input=new Set<int>();schema.parse(input).size=1;","unsupported"),
        ("const schema=s.map(s.string(),s.int());const input=new Map<string,int>();schema.parse(input).size=1;","unsupported"),
    ] {
        let source=format!("import {{s}} from 'std:schema';function main(){{{body}}}");
        let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(&source));
        let output=support::run(support::cli().arg("check").arg(path),30);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn collections_compose_restore_readonly_rejections_and_commit_collisions(){
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const rows=s.array(s.record(s.array(s.int())));
      {const outcome6=rows.safeParseJson('[{"a":[1,2]},{"b":[]}]');if(outcome6.success){const value=outcome6.data;if(value.length!==2||!value[0].has('a')){throw 'Wrong nested record';}}else{const error=outcome6.error;throw 'Nested records rejected';}}
      const alternatives=s.union([
        s.record(s.string().refine(value=>value.length>9)),
        s.record(s.string())
      ]);
      {const outcome7=alternatives.safeParse(JSON.parse('{"first":"short","second":"ok"}'));if(outcome7.success){const value=outcome7.data;if(value.get('first')!=='short'||value.get('second')!=='ok'){throw 'Rollback changed dictionary';}}else{const error=outcome7.error;throw 'Readonly fallback rejected';}}
      const choices=s.union([
        s.set(s.string().transform(value=>value.toLowerCase())),
        s.set(s.string()).transform(value=>{if(value.size>0){throw 'Retried after consuming collision';}return value;})
      ]);
      const input=new Set<string>();input.add('A');input.add('a');
      {const outcome8=choices.safeParse(input);if(outcome8.success){const value=outcome8.data;throw 'Collision accepted';}else{const error=outcome8.error;if(error.issues[0].code!=='collection-collision'){throw 'Wrong collision code';}}}
      const throwing=s.map(s.string().transform(value=>{if(value==='stop'){throw 'key failure';}return value;}),s.int().transform(value=>{if(value===1){throw 'Value ran after failed key';}return value;}));
      const entries=new Map<string,int>();entries.set('stop',1);
      try{throwing.safeParse(entries);throw 'Expected key exception';}catch(error){if(error!=='key failure'){throw 'Wrong exception';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn identity_collections_preserve_bounds_paths_order_and_union_fallback(){
    let source=r#"
import {s} from 'std:schema';
import {strictEqual} from 'std:testing';
function main(){
 const input=new Map<string,int>();input.set('gone',9);input.set('first',2);input.delete('gone');input.set('last',3);
 const schema=s.map(s.string().min(2),s.int().min(0));
 const value=schema.parse(input);strictEqual(value.keys().join(','),'first,last');strictEqual(value.get('last'),3);
 const tags=new Set<string>();tags.add('gone');tags.add('kept');tags.delete('gone');tags.add('last');
 const result=s.set(s.string().min(2)).parse(tags);strictEqual(result.values().join(','),'kept,last');
 const choices=s.union([s.map(s.string().min(9),s.int().min(10)),s.map(s.string(),s.int())]);
 const fallback=new Map<string,int>();fallback.set('short',1);
 const accepted=choices.parse(fallback);strictEqual(accepted.get('short'),1);
 const badKey=new Map<string,int>();badKey.set('x',1);
 const keyOutcome=s.map(s.string().min(2),s.int()).safeParse(badKey);
 if(keyOutcome.success){throw 'short key accepted';}else{const error=keyOutcome.error;strictEqual(error.issues.length,1);strictEqual(error.issues[0].path[0].index,0);strictEqual(error.issues[0].path[1].field,'key');}
 const badValue=new Map<string,int>();badValue.set('ok',-1);
 const valueOutcome=s.map(s.string(),s.int().min(0)).safeParse(badValue);
 if(valueOutcome.success){throw 'negative value accepted';}else{const error=valueOutcome.error;strictEqual(error.issues[0].path[1].field,'value');}
 const badSet=new Set<string>();badSet.add('x');
 const setOutcome=s.set(s.string().min(2)).safeParse(badSet);
 if(setOutcome.success){throw 'short set entry accepted';}else{const error=setOutcome.error;strictEqual(error.issues[0].path[0].index,0);strictEqual(error.issues[0].path.length,1);}
 const numbers=new Map<string,number>();numbers.set('nan',NaN);
 const numberOutcome=s.map(s.string(),s.number()).safeParse(numbers);
 if(numberOutcome.success){throw 'nonfinite value accepted';}else{const error=numberOutcome.error;strictEqual(error.issues[0].code,'number-nonfinite');}
 const flags=new Set<boolean>();flags.add(false);flags.add(true);strictEqual(s.set(s.boolean()).parse(flags).size,2);
 const empty=new Map<string,int>();strictEqual(s.map(s.string().min(9),s.int().min(9)).parse(empty).size,0);
}
"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}
