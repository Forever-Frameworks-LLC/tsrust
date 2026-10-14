mod support;
#[test]
fn enum_schema_infers_literal_union_and_preserves_owned_strings() {
    let source=r#"
    import {s} from 'std:schema';
    function accept(value:'ready'|'done'):string{return String(value);}
    function main(){const schema=s.enum(['ready','done']);const input='done'.repeat(1);{const outcome0=schema.safeParse(input);if(outcome0.success){const value=outcome0.data;console.log(accept(value));}else{const error=outcome0.error;throw 'owned rejected';}}{const outcome1=schema.safeParse(JSON.parse('"ready"'));if(outcome1.success){const value=outcome1.data;console.log(accept(value));}else{const error=outcome1.error;throw 'json rejected';}}{const outcome2=schema.safeParse('waiting');if(outcome2.success){const value=outcome2.data;throw 'invalid accepted';}else{const error=outcome2.error;console.log(error.issues[0].code);}}}
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
    function main(){const optional=s.enum(['ready','done']).default(()=>'ready').optional();const schema=s.object({state:optional,values:s.array(s.enum(['a','b']))});{const outcome3=schema.safeParse(JSON.parse('{"values":["b","a"]}'));if(outcome3.success){const value=outcome3.data;console.log(accept(value.state),value.values.length);}else{const error=outcome3.error;throw 'default rejected';}}const nullable=s.enum(['ready','done']).nullable().optional();{const outcome4=nullable.safeParse(null);if(outcome4.success){const value=outcome4.data;console.log(value);}else{const error=outcome4.error;throw 'null rejected';}}{const outcome5=nullable.safeParse('done');if(outcome5.success){const value=outcome5.data;console.log(value);}else{const error=outcome5.error;throw 'owned rejected';}}}
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
    function main(){const singleton=s.enum(['only']);{const outcome6=singleton.safeParse('only');if(outcome6.success){const value=outcome6.data;const exact:'only'=value;console.log(exact);}else{const error=outcome6.error;throw 'single rejected';}}const schema=s.array(s.enum(['done','ready']));const input:string[]=['ready','done'];{const outcome7=schema.safeParse(input);if(outcome7.success){const values=outcome7.data;for(const value of values){console.log(value);}}else{const error=outcome7.error;throw 'array rejected';}}{const outcome8=s.enum(['ready','done']).safeParse(JSON.parse('false'));if(outcome8.success){const value=outcome8.data;throw 'wrong kind accepted';}else{const error=outcome8.error;console.log(error.issues[0].code);}}}
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
    function main(){const first=s.enum(['ready','done']);const second=s.enum([done(),ready()]);const schema=s.object({state:second});const input:Input={state:'done'};{const outcome9=schema.safeParse(input);if(outcome9.success){const value=outcome9.data;console.log(value.state);}else{const error=outcome9.error;throw 'owned record rejected';}}{const outcome10=first.safeParse('ready');if(outcome10.success){const value=outcome10.data;console.log(value);}else{const error=outcome10.error;throw 'first rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"done factory\nready factory\ndone\nready\n");
}
#[test]
fn enum_schema_catalog_examples_execute() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    let mut seen=std::collections::HashSet::new();
    for entry in catalog["schemaBuilder"].as_array().unwrap().iter().filter(|entry|entry["name"]=="enum").chain(catalog["SchemaEnum"].as_array().unwrap().iter()).chain(catalog["SchemaDeclaredEnum"].as_array().unwrap().iter()) {
        for example in entry["examples"].as_array().unwrap(){
            let source=example["source"].as_str().unwrap();if !seen.insert(source){continue;}
            let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
            let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
            assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
        }
    }
}

#[test]
fn declared_enum_schemas_keep_nominal_output_for_strings_and_numbers() {
    let source=r#"
    import {s} from 'std:schema';
    enum State{Ready='ready',Done='done'}
    enum Code{Low=1,High=2.5,Alias=1}
    function state(value:State):string{return String(value);}
    function code(value:Code):number{return Number(value);}
    function main(){const states=s.enum(State);{const outcome11=states.safeParse('done');if(outcome11.success){const value=outcome11.data;console.log(state(value));}else{const error=outcome11.error;throw 'string rejected';}}{const outcome12=states.safeParse(JSON.parse('"ready"'));if(outcome12.success){const value=outcome12.data;console.log(state(value));}else{const error=outcome12.error;throw 'json string rejected';}}const codes=s.enum(Code);{const outcome13=codes.safeParse(1);if(outcome13.success){const value=outcome13.data;console.log(code(value));}else{const error=outcome13.error;throw 'integer rejected';}}{const outcome14=codes.safeParse(JSON.parse('2.5'));if(outcome14.success){const value=outcome14.data;console.log(code(value));}else{const error=outcome14.error;throw 'json number rejected';}}{const outcome15=codes.safeParse(3.0);if(outcome15.success){const value=outcome15.data;throw 'invalid accepted';}else{const error=outcome15.error;console.log(error.issues[0].code);}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"done\nready\n1\n2.5\nliteral-mismatch\n");
}

#[test]
fn declared_enum_schema_chains_validate_records_arrays_defaults_and_exact_numbers() {
    let source=r#"
    import {s} from 'std:schema';
    enum State{Ready='ready',Done='done'}enum Code{Low=1,High=2.5}
    type Input={state:string;codes:int[];};
    function main(){const schema=s.object({state:s.enum(State).default(()=>State.Ready),codes:s.array(s.enum(Code))});const input:Input={state:'done',codes:[1]};{const outcome16=schema.safeParse(input);if(outcome16.success){const value=outcome16.data;console.log(value.state,value.codes[0]);}else{const error=outcome16.error;throw 'record rejected';}}{const outcome17=schema.safeParse(JSON.parse('{"codes":[2.5]}'));if(outcome17.success){const value=outcome17.data;console.log(value.state,value.codes[0]);}else{const error=outcome17.error;throw 'default rejected';}}const optional=s.enum(Code).nullable().optional();{const outcome18=optional.safeParse(1);if(outcome18.success){const value=outcome18.data;console.log(value);}else{const error=outcome18.error;throw 'optional rejected';}}{const outcome19=optional.safeParse(undefined);if(outcome19.success){const value=outcome19.data;console.log(value);}else{const error=outcome19.error;throw 'missing rejected';}}const codes=s.enum(Code);{const outcome20=codes.safeParse(9007199254740993);if(outcome20.success){const value=outcome20.data;throw 'lossy accepted';}else{const error=outcome20.error;console.log(error.issues[0].code);}}{const outcome21=codes.safeParse(JSON.parse('9007199254740993'));if(outcome21.success){const value=outcome21.data;throw 'lossy json accepted';}else{const error=outcome21.error;console.log(error.issues[0].code);}}{const outcome22=codes.safeParse(Infinity);if(outcome22.success){const value=outcome22.data;throw 'infinity accepted';}else{const error=outcome22.error;console.log(error.issues[0].code);}}{const outcome23=codes.safeParse(Code.Low);if(outcome23.success){const value=outcome23.data;console.log(value);}else{const error=outcome23.error;throw 'nominal rejected';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"done 1\nready 2.5\n1\nundefined\nnumber-precision\nnumber-precision\nnumber-nonfinite\n1\n");
}

#[test]
fn declared_enum_schemas_reject_empty_nonfinite_shadowed_and_wrong_nominal_inputs() {
    for source in [
        "import {s} from 'std:schema';enum E{}function main(){s.enum(E);}",
        "import {s} from 'std:schema';enum E{A=Infinity}function main(){s.enum(E);}",
        "import {s} from 'std:schema';enum E{A='a'}function main(){const E='a';s.enum(E);}",
        "import {s} from 'std:schema';enum E{A='a'}enum F{A='a'}function main(){s.enum(E).safeParse(F.A);}",
    ] {let directory=support::TempDirectory::new();let path=directory.source(source);let output=support::run(support::cli().arg("check").arg(path),60);assert!(!output.status.success(),"accepted {source}");}
}

#[test]
fn declared_enum_schemas_resolve_import_aliases_and_mixed_owned_arrays() {
    let directory=support::TempDirectory::new();
    std::fs::write(directory.0.join("enum.tsr"),support::format_round_trip("export enum Code{Low=1,High=2.5}")).unwrap();
    let source="import {Code as Status} from './enum.tsr';import {s} from 'std:schema';function main(){const schema=s.array(s.enum(Status));const input:(Status|number|int)[]=[Status.Low,2.5,1];{const outcome24=schema.safeParse(input);if(outcome24.success){const values=outcome24.data;console.log(values.length,values[1]===Status.High);}else{const error=outcome24.error;throw 'mixed rejected';}}}";
    let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"3 true\n");
}
