mod support;
use std::{collections::BTreeMap, fs, process::Command};
use tsrust_compiler::api::{ApiHistory, HttpDescription, Kind, Manifest, Operation, Schema};

fn description() -> HttpDescription {
    let record = Schema::Object { properties:BTreeMap::from([
        ("-bad".into(), Schema::Boolean),
        ("count".into(), Schema::Integer),
        ("field_0".into(), Schema::Boolean),
        ("label".into(), Schema::Union { members:vec![Schema::String, Schema::Null] }),
        ("type".into(), Schema::Boolean),
    ]), required:vec!["count".into(), "type".into()] };
    let manifest = Manifest { schema_version:1, operations:vec![
        Operation { name:"tasks.echo".into(), kind:Kind::Mutation, input:record.clone(), output:record },
        Operation { name:"number".into(), kind:Kind::Query, input:Schema::Number, output:Schema::Number },
        Operation { name:"nothing".into(), kind:Kind::Query, input:Schema::Void, output:Schema::Void },
        Operation { name:"literal".into(), kind:Kind::Query, input:Schema::Literal { value:serde_json::json!("\u{0}value") }, output:Schema::Tuple { items:vec![Schema::Integer, Schema::String] } },
        Operation { name:"self".into(), kind:Kind::Query, input:Schema::Void, output:Schema::Void },
        Operation { name:"operation_4".into(), kind:Kind::Query, input:Schema::Void, output:Schema::Void },
    ] };
    HttpDescription::new(ApiHistory::new("@witness/rust").advance("@witness/rust", "0.1.0", 1, &manifest).unwrap()).unwrap()
}

#[test]
fn rust_sdk_compiles_and_validates_named_calls_models_and_failures() {
    let directory = support::TempDirectory::new();
    let files = description().rust_package().unwrap();
    syn::parse_file(&files["src/lib.rs"]).expect("Generated Rust must parse before invoking Cargo");
    for (name, source) in files {
        let output = directory.0.join(name);
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(output, source).unwrap();
    }
    fs::create_dir_all(directory.0.join("src/bin")).unwrap();
    fs::write(directory.0.join("src/bin/witness.rs"), r#"
use tsrust_witness_rust::{Client, Request, ApiError, Op0Input, Op0Output, Op3Input};
use serde_json::{Value, json};
fn main() {
    let transport = |request: Request<'_>| -> Result<Value,ApiError> {
        assert_eq!(request.api_version,1);
        assert_eq!(request.operation_contract.len(),64);
        assert_eq!(request.contract_id.len(),64);
        Ok(json!({"ok":true,"value":request.input}))
    };
    let api = Client::new(transport);
    let input:Op0Input = serde_json::from_value(json!({"count":4,"type":true})).unwrap();
    let result:Op0Output=api.api().tasks().echo(input).unwrap();
    assert_eq!(result.count,4);assert!(result.r#type);assert!(result.label.is_none());
    let explicit_null:Op0Input=serde_json::from_value(json!({"count":4,"type":true,"label":null})).unwrap();
    assert!(explicit_null.label.is_some());
    let explicit_null=serde_json::to_value(api.api().tasks().echo(explicit_null).unwrap()).unwrap();
    assert!(explicit_null.as_object().unwrap().contains_key("label"));assert_eq!(explicit_null["label"],Value::Null);
    let unsafe_integer:Op0Input=serde_json::from_value(json!({"count":9_007_199_254_740_992_i64,"type":true})).unwrap();
    assert_eq!(api.api().tasks().echo(unsafe_integer).unwrap_err().code,"api-validation");
    assert_eq!(api.api().number(f64::INFINITY).unwrap_err().code,"api-validation");
    assert_eq!(api.api().number(1e100).unwrap(),1e100);
    api.api().nothing().unwrap();
    api.api().operation_4().unwrap();
    api.api().operation_4_().unwrap();
    let collisions:Op0Input=serde_json::from_value(json!({"count":4,"type":true,"-bad":true,"field_0":false})).unwrap();
    let collisions=serde_json::to_value(api.api().tasks().echo(collisions).unwrap()).unwrap();
    assert_eq!(collisions["-bad"],true);assert_eq!(collisions["field_0"],false);
    let literal:Op3Input=serde_json::from_value(json!("\u{0}value")).unwrap();
    let literal_client=Client::new(|_:Request<'_>| Ok(json!({"ok":true,"value":[7,"tuple"]})));
    assert_eq!(literal_client.api().literal(literal).unwrap(),(7,"tuple".into()));
    let denied=Client::new(|_:Request<'_>| Ok(json!({"ok":false,"error":{"code":"api-denied","message":"Denied"}})));
    assert_eq!(denied.api().nothing().unwrap_err().code,"api-denied");
    let incompatible=Client::new(|_:Request<'_>| Ok(json!({"ok":false,"error":{"code":"api-incompatible","message":"Upgrade"}})));
    assert!(incompatible.api().nothing().unwrap_err().is_compatibility());
    let wrong=Client::new(|_:Request<'_>| Ok(json!({"ok":true,"value":{"count":1,"type":true,"extra":3}})));
    let input:Op0Input=serde_json::from_value(json!({"count":1,"type":true})).unwrap();
    assert_eq!(wrong.api().tasks().echo(input).unwrap_err().code,"api-validation");
    let fractional=Client::new(|_:Request<'_>| Ok(json!({"ok":true,"value":{"count":4.0,"type":true}})));
    let input:Op0Input=serde_json::from_value(json!({"count":1,"type":true})).unwrap();
    assert_eq!(fractional.api().tasks().echo(input).unwrap().count,4);
    println!("Rust named models and wire validation passed");
}
"#).unwrap();
    let command = || { let mut command=Command::new("rustup"); command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo"]).current_dir(&directory.0).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",std::env::var_os("CARGO_TARGET_DIR").unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").into_os_string())); command };
    support::success(&support::run(command().args(["generate-lockfile","--offline"]), 60));
    let output=support::run(command().args(["run","--offline","--locked","--no-default-features","--bin","witness"]),120);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(),"Rust named models and wire validation passed");
    support::success(&support::run(command().args(["check","--offline","--locked","--all-targets"]),120));
}

#[test]
fn rust_sdk_rejects_unsupported_tuple_before_output() {
    let mut value=description();
    let manifest=Manifest { schema_version:1, operations:vec![Operation { name:"large".into(),kind:Kind::Query,input:Schema::Void,output:Schema::Tuple { items:vec![Schema::String;17] } }] };
    value.history=ApiHistory::new("@witness/rust").advance("@witness/rust","0.1.0",1,&manifest).unwrap();
    assert!(value.rust_package().unwrap_err().contains("16"));
}
