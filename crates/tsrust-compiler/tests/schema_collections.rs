//! Collection section witnesses; run after checker/native/editor integration lands.
mod support;

#[test]
fn collection_schemas_preserve_typed_output_paths_and_unique_values() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){
      const dictionary=s.record(s.int().min(0));
      match(dictionary.safeParseJson('{"alice":10,"bob":7}')){
        Ok(values)=>{if(values.size!==2||values.get('alice')!==10){throw 'Wrong dictionary';}},
        Err(error)=>{throw 'Dictionary rejected';}
      }
      match(dictionary.safeParseJson('{"alice":-1}')){
        Ok(values)=>{throw 'Negative dictionary entry accepted';},
        Err(failure)=>{if(failure.kind==='validation'){
          if(failure.error.issues[0].path[0].field!=='alice'){throw 'Wrong dictionary path';}
        }else{throw 'Wrong dictionary error';}}
      }
      const input=new Map<int,string>();input.set(7,'seven');
      const map=s.map(s.int().min(0),s.string().min(1));
      match(map.safeParse(input)){
        Ok(values)=>{if(values.get(7)!=='seven'){throw 'Wrong map';}},
        Err(error)=>{throw 'Map rejected';}
      }
      const tags=new Set<string>();tags.add('admin');tags.add('editor');
      const set=s.set(s.string().min(1));
      match(set.safeParse(tags)){
        Ok(values)=>{if(values.size!==2||!values.has('admin')){throw 'Wrong set';}},
        Err(error)=>{throw 'Set rejected';}
      }
      const collisions=new Map<string,int>();collisions.set('A',1);collisions.set('a',2);
      const lowered=s.map(s.string().transform(value=>value.toLowerCase()),s.int());
      match(lowered.safeParse(collisions)){
        Ok(values)=>{throw 'Colliding map keys accepted';},
        Err(error)=>{if(error.issues.length===0){throw 'Missing collision issue';}}
      }
      const elements=new Set<string>();elements.add('A');elements.add('a');
      const loweredSet=s.set(s.string().transform(value=>value.toLowerCase()));
      match(loweredSet.safeParse(elements)){
        Ok(values)=>{throw 'Colliding set elements accepted';},
        Err(error)=>{if(error.issues.length===0){throw 'Missing set collision issue';}}
      }
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
      match(rows.safeParseJson('[{"a":[1,2]},{"b":[]}]')){
        Ok(value)=>{if(value.length!==2||!value[0].has('a')){throw 'Wrong nested record';}},
        Err(error)=>{throw 'Nested records rejected';}
      }
      const alternatives=s.union([
        s.record(s.string().refine(value=>value.length>9)),
        s.record(s.string())
      ]);
      match(alternatives.safeParse(JSON.parse('{"first":"short","second":"ok"}'))){
        Ok(value)=>{if(value.get('first')!=='short'||value.get('second')!=='ok'){throw 'Rollback changed dictionary';}},
        Err(error)=>{throw 'Readonly fallback rejected';}
      }
      const choices=s.union([
        s.set(s.string().transform(value=>value.toLowerCase())),
        s.set(s.string()).transform(value=>{if(value.size>0){throw 'Retried after consuming collision';}return value;})
      ]);
      const input=new Set<string>();input.add('A');input.add('a');
      match(choices.safeParse(input)){
        Ok(value)=>{throw 'Collision accepted';},
        Err(error)=>{if(error.issues[0].code!=='collection-collision'){throw 'Wrong collision code';}}
      }
      const throwing=s.map(s.string().transform(value=>{if(value==='stop'){throw 'key failure';}return value;}),s.int().transform(value=>{if(value===1){throw 'Value ran after failed key';}return value;}));
      const entries=new Map<string,int>();entries.set('stop',1);
      try{throwing.safeParse(entries);throw 'Expected key exception';}catch(error){if(error!=='key failure'){throw 'Wrong exception';}}
    }
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}
