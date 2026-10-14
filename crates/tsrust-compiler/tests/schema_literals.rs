mod support;

#[test]
fn schema_literal_validates_owned_and_json_without_widening_output() {
    let source=r#"
    import {s} from 'std:schema';
    function ready(value:'ready'):string{return value;}
    function main(){const schema=s.literal('ready');const input='ready'.repeat(1);{const outcome0=schema.safeParse(input);if(outcome0.success){const value=outcome0.data;console.log(ready(value));}else{const error=outcome0.error;throw 'owned rejected';}}{const outcome1=schema.safeParse(JSON.parse('"ready"'));if(outcome1.success){const value=outcome1.data;console.log(ready(value));}else{const error=outcome1.error;throw 'json rejected';}}{const outcome2=schema.safeParse('wrong');if(outcome2.success){const value=outcome2.data;throw 'invalid accepted';}else{const error=outcome2.error;console.log(error.issues[0].code);}}}
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
    function main(){const integer=s.literal(9007199254740993);{const outcome3=integer.safeParse(JSON.parse('9007199254740993'));if(outcome3.success){const value=outcome3.data;console.log(exact(value));}else{const error=outcome3.error;throw 'exact rejected';}}{const outcome4=integer.safeParse(9007199254740992);if(outcome4.success){const value=outcome4.data;throw 'rounded accepted';}else{const error=outcome4.error;console.log(error.issues[0].code);}}const flag=s.literal(false);{const outcome5=flag.safeParse(false);if(outcome5.success){const value=outcome5.data;const typed:false=value;console.log(typed);}else{const error=outcome5.error;throw 'false rejected';}}const decimal=s.literal(2.5);{const outcome6=decimal.safeParse(JSON.parse('2.5'));if(outcome6.success){const value=outcome6.data;const typed:2.5=value;console.log(typed);}else{const error=outcome6.error;throw 'decimal rejected';}}}
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
    function main(){const schema=s.object({tag:s.literal('ready').default(()=>'ready'),values:s.array(s.literal(7))});{const outcome7=schema.safeParse(JSON.parse('{"values":[7,7]}'));if(outcome7.success){const value=outcome7.data;console.log(ready(value.tag),value.values.length);}else{const error=outcome7.error;throw 'nested rejected';}}{const outcome8=schema.safeParse({tag:'ready',values:[7]});if(outcome8.success){const value=outcome8.data;console.log(ready(value.tag),value.values.length);}else{const error=outcome8.error;throw 'owned rejected';}}}
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
    function main(){const schema=s.object({tag:s.literal('ready'),values:s.array(s.literal(7))});const input:Input={tag:'wrong',values:[8]};{const outcome9=schema.safeParse(input);if(outcome9.success){const value=outcome9.data;throw 'invalid accepted';}else{const error=outcome9.error;console.log(error.issues.length,error.issues[0].code,error.issues[1].code);}}const amount=s.literal(2.0);const integer:int=2;{const outcome10=amount.safeParse(integer);if(outcome10.success){const value=outcome10.data;const typed:2.0=value;console.log(typed);}else{const error=outcome10.error;throw 'exact integer rejected';}}const nothing=s.literal(null);{const outcome11=nothing.safeParse(null);if(outcome11.success){const value=outcome11.data;console.log(value);}else{const error=outcome11.error;throw 'null rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2 literal-mismatch literal-mismatch\n2\nnull\n");
}

#[test]
fn literal_presence_chains_validate_owned_scalar_inputs() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){const schema=s.literal('ready').optional();{const outcome12=schema.safeParse('wrong');if(outcome12.success){const value=outcome12.data;throw 'invalid accepted';}else{const error=outcome12.error;console.log(error.issues[0].code);}}{const outcome13=schema.safeParse(undefined);if(outcome13.success){const value=outcome13.data;console.log(typeof value);}else{const error=outcome13.error;throw 'missing rejected';}}const nullable=s.literal(7).nullable();{const outcome14=nullable.safeParse(JSON.parse('null'));if(outcome14.success){const value=outcome14.data;console.log(value);}else{const error=outcome14.error;throw 'null rejected';}}}
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
    function main(){const tag:'ready'='ready';const schema=s.literal(tag);{const outcome15=schema.safeParse('ready');if(outcome15.success){const value=outcome15.data;console.log(value);}else{const error=outcome15.error;throw 'literal binding failed';}}{const outcome16=s.literal(nothing()).safeParse(null);if(outcome16.success){const value=outcome16.data;console.log(value);}else{const error=outcome16.error;throw 'null failed';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"ready\nfactory\nnull\n");
}

#[test]
fn number_literal_presence_keeps_checked_integer_conversion() {
    let source=r#"
    import {s} from 'std:schema';
    function main(){const schema=s.literal(2.0).optional();const exact:int=2;{const outcome17=schema.safeParse(exact);if(outcome17.success){const value=outcome17.data;console.log(value);}else{const error=outcome17.error;throw 'exact rejected';}}const imprecise:int=9007199254740993;{const outcome18=schema.safeParse(imprecise);if(outcome18.success){const value=outcome18.data;throw 'rounded accepted';}else{const error=outcome18.error;console.log(error.issues[0].code);}}const array=s.array(s.literal(2.0).optional());const values:int[]=[2];{const outcome19=array.safeParse(values);if(outcome19.success){const value=outcome19.data;console.log(value.length);}else{const error=outcome19.error;throw 'array rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2\nnumber-precision\n1\n");
}
