mod support;
#[test]
fn enum_schema_infers_literal_union_and_preserves_owned_strings() {
    let source=r#"
    import {s} from 'std:schema';
    function accept(value:'ready'|'done'):string{return String(value);}
    function main(){const schema=s.enum(['ready','done']);const input='done'.repeat(1);match(schema.safeParse(input)){Ok(value)=>{console.log(accept(value));},Err(error)=>{throw 'owned rejected';}}match(schema.safeParse(JSON.parse('"ready"'))){Ok(value)=>{console.log(accept(value));},Err(error)=>{throw 'json rejected';}}match(schema.safeParse('waiting')){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{console.log(error.issues[0].code);}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"done\nready\nliteral-mismatch\n");
}

#[test]
fn enum_schema_composes_with_defaults_presence_arrays_and_objects() {
    let source=r#"
    import {s} from 'std:schema';
    function accept(value:'ready'|'done'|undefined):string{return String(value);}
    function main(){const optional=s.enum(['ready','done']).default(()=>'ready').optional();const schema=s.object({state:optional,values:s.array(s.enum(['a','b']))});match(schema.safeParse(JSON.parse('{"values":["b","a"]}'))){Ok(value)=>{console.log(accept(value.state),value.values.length);},Err(error)=>{throw 'default rejected';}}const nullable=s.enum(['ready','done']).nullable().optional();match(nullable.safeParse(null)){Ok(value)=>{console.log(value);},Err(error)=>{throw 'null rejected';}}match(nullable.safeParse('done')){Ok(value)=>{console.log(value);},Err(error)=>{throw 'owned rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready 2\nnull\ndone\n");
}

#[test]
fn enum_schema_rejects_invalid_lists_and_preserves_ownership() {
    for (source,code) in [
        ("import {s} from 'std:schema';function main(){s.enum([]);}","schema-enum"),
        ("import {s} from 'std:schema';function main(){s.enum(['a','a']);}","schema-enum"),
        ("import {s} from 'std:schema';function main(){s.enum([1,2]);}","schema-enum"),
        ("import {s} from 'std:schema';function main(){const values=['a','b'];s.enum(values);}","schema-enum"),
        ("import {s} from 'std:schema';function main(){const schema=s.enum(['a','b']);const input='a';schema.safeParse(input);console.log(input);}","transferred"),
    ] {
        let directory=support::TempDirectory::new();let path=directory.source(source);
        let output=support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success(),"accepted: {source}");assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn enum_schema_handles_singletons_reordered_lists_and_owned_arrays() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){const singleton=s.enum(['only']);match(singleton.safeParse('only')){Ok(value)=>{const exact:'only'=value;console.log(exact);},Err(error)=>{throw 'single rejected';}}const schema=s.array(s.enum(['done','ready']));const input:string[]=['ready','done'];match(schema.safeParse(input)){Ok(values)=>{for(const value of values){console.log(value);}},Err(error)=>{throw 'array rejected';}}match(s.enum(['ready','done']).safeParse(JSON.parse('false'))){Ok(value)=>{throw 'wrong kind accepted';},Err(error)=>{console.log(error.issues[0].code);}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"only\nready\ndone\ntype-mismatch\n");
}

#[test]
fn enum_schema_evaluates_values_once_in_source_order_and_validates_owned_records() {
    let source=r#"
    import {s} from 'std:schema';
    type Input={state:string;};
    function ready():'ready'{console.log('ready factory');return 'ready';}
    function done():'done'{console.log('done factory');return 'done';}
    function main(){const first=s.enum(['ready','done']);const second=s.enum([done(),ready()]);const schema=s.object({state:second});const input:Input={state:'done'};match(schema.safeParse(input)){Ok(value)=>{console.log(value.state);},Err(error)=>{throw 'owned record rejected';}}match(first.safeParse('ready')){Ok(value)=>{console.log(value);},Err(error)=>{throw 'first rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"done factory\nready factory\ndone\nready\n");
}
#[test]
fn enum_schema_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut seen=std::collections::HashSet::new();
    for entry in catalog["schemaBuilder"].as_array().unwrap().iter().filter(|entry|entry["name"]=="enum").chain(catalog["SchemaEnum"].as_array().unwrap().iter()) {
        for example in entry["examples"].as_array().unwrap(){
            let source=example["source"].as_str().unwrap();if !seen.insert(source){continue;}
            let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
            let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
            assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
        }
    }
}
