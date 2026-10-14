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
