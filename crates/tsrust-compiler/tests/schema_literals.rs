mod support;

#[test]
fn schema_literal_validates_owned_and_json_without_widening_output() {
    let source=r#"
    import {s} from 'std:schema';
    function ready(value:'ready'):string{return value;}
    function main(){const schema=s.literal('ready');const input='ready'.repeat(1);match(schema.safeParse(input)){Ok(value)=>{console.log(ready(value));},Err(error)=>{throw 'owned rejected';}}match(schema.safeParse(JSON.parse('"ready"'))){Ok(value)=>{console.log(ready(value));},Err(error)=>{throw 'json rejected';}}match(schema.safeParse('wrong')){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{console.log(error.issues[0].code);}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\nready\nliteral-mismatch\n");
}

#[test]
fn schema_literals_preserve_exact_numeric_and_boolean_outputs() {
    let source=r#"
    import {s} from 'std:schema';
    function exact(value:9007199254740993):int{return value;}
    function main(){const integer=s.literal(9007199254740993);match(integer.safeParse(JSON.parse('9007199254740993'))){Ok(value)=>{console.log(exact(value));},Err(error)=>{throw 'exact rejected';}}match(integer.safeParse(9007199254740992)){Ok(value)=>{throw 'rounded accepted';},Err(error)=>{console.log(error.issues[0].code);}}const flag=s.literal(false);match(flag.safeParse(false)){Ok(value)=>{const typed:false=value;console.log(typed);},Err(error)=>{throw 'false rejected';}}const decimal=s.literal(2.5);match(decimal.safeParse(JSON.parse('2.5'))){Ok(value)=>{const typed:2.5=value;console.log(typed);},Err(error)=>{throw 'decimal rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"9007199254740993\nliteral-mismatch\nfalse\n2.5\n");
}

#[test]
fn literal_schemas_compose_with_objects_arrays_and_defaults() {
    let source=r#"
    import {s} from 'std:schema';
    function ready(value:'ready'):string{return value;}
    function main(){const schema=s.object({tag:s.literal('ready').default(()=>'ready'),values:s.array(s.literal(7))});match(schema.safeParse(JSON.parse('{"values":[7,7]}'))){Ok(value)=>{console.log(ready(value.tag),value.values.length);},Err(error)=>{throw 'nested rejected';}}match(schema.safeParse({tag:'ready',values:[7]})){Ok(value)=>{console.log(ready(value.tag),value.values.length);},Err(error)=>{throw 'owned rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready 2\nready 1\n");
}

#[test]
fn literal_schema_owned_composites_validate_broad_inputs_and_keep_paths() {
    let source=r#"
    import {s} from 'std:schema';
    type Input={tag:string;values:int[];};
    function main(){const schema=s.object({tag:s.literal('ready'),values:s.array(s.literal(7))});const input:Input={tag:'wrong',values:[8]};match(schema.safeParse(input)){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{console.log(error.issues.length,error.issues[0].code,error.issues[1].code);}}const amount=s.literal(2.0);const integer:int=2;match(amount.safeParse(integer)){Ok(value)=>{const typed:2.0=value;console.log(typed);},Err(error)=>{throw 'exact integer rejected';}}const nothing=s.literal(null);match(nothing.safeParse(null)){Ok(value)=>{console.log(value);},Err(error)=>{throw 'null rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2 literal-mismatch literal-mismatch\n2\nnull\n");
}

#[test]
fn literal_presence_chains_validate_owned_scalar_inputs() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){const schema=s.literal('ready').optional();match(schema.safeParse('wrong')){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{console.log(error.issues[0].code);}}match(schema.safeParse(undefined)){Ok(value)=>{console.log(typeof value);},Err(error)=>{throw 'missing rejected';}}const nullable=s.literal(7).nullable();match(nullable.safeParse(JSON.parse('null'))){Ok(value)=>{console.log(value);},Err(error)=>{throw 'null rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"literal-mismatch\nundefined\nnull\n");
}

#[test]
fn literal_schema_requires_known_values_and_consumes_inputs() {
    for (source,code) in [
      ("import {s} from 'std:schema';function build(value:string){s.literal(value);}function main(){}","schema-literal"),
      ("import {s} from 'std:schema';function main(){const schema=s.literal('ready');const value='wrong';schema.safeParse(value);console.log(value);}","transferred"),
      ("import {s} from 'std:schema';function main(){s.literal('ready').min(1);}","unsupported"),
    ] {
        let directory=support::TempDirectory::new();let path=directory.source(source);
        let output=support::run(support::cli().arg("check").arg(path),60);
        assert!(!output.status.success(),"accepted: {source}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn literal_schema_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut seen=std::collections::HashSet::new();
    for entry in catalog["schemaBuilder"].as_array().unwrap().iter().filter(|entry|entry["name"]=="literal").chain(catalog["SchemaLiteral"].as_array().unwrap().iter()) {
        for example in entry["examples"].as_array().unwrap(){
            let source=example["source"].as_str().unwrap();if !seen.insert(source){continue;}
            let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
            let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
            assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
        }
    }
}

#[test]
fn literal_schema_accepts_typed_literal_inputs_and_evaluates_null_once() {
    let source=r#"
    import {s} from 'std:schema';
    function nothing():null{console.log('factory');return null;}
    function main(){const tag:'ready'='ready';const schema=s.literal(tag);match(schema.safeParse('ready')){Ok(value)=>{console.log(value);},Err(error)=>{throw 'literal binding failed';}}match(s.literal(nothing()).safeParse(null)){Ok(value)=>{console.log(value);},Err(error)=>{throw 'null failed';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\nfactory\nnull\n");
}

#[test]
fn number_literal_presence_keeps_checked_integer_conversion() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){const schema=s.literal(2.0).optional();const exact:int=2;match(schema.safeParse(exact)){Ok(value)=>{console.log(value);},Err(error)=>{throw 'exact rejected';}}const imprecise:int=9007199254740993;match(schema.safeParse(imprecise)){Ok(value)=>{throw 'rounded accepted';},Err(error)=>{console.log(error.issues[0].code);}}const array=s.array(s.literal(2.0).optional());const values:int[]=[2];match(array.safeParse(values)){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'array rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2\nnumber-precision\n1\n");
}
