mod support;
#[test]
fn decimals_preserve_money_precision_and_explicit_ownership(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/decimal/money.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);
 assert_eq!(output.stdout,b"59.97\ndecimal boundaries exact\n");
}
#[test]
fn decimal_documentation_examples(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut imports=std::collections::BTreeSet::new();let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
 for entry in catalog["std:decimal"].as_array().unwrap().iter(){for example in entry["examples"].as_array().unwrap(){
  let source=example["source"].as_str().unwrap();let (prefix,body)=source.split_once("function main()").unwrap();let start=prefix.find('{').unwrap();let end=prefix.find('}').unwrap();for name in prefix[start+1..end].split(','){imports.insert(name.trim().to_owned());}
  functions.push_str(&format!("// {}\nfunction example{count}(){body}\n",entry["name"].as_str().unwrap()));calls.push_str(&format!("example{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;
 }}
 assert_eq!(count,13);let source=format!("import {{{}}} from 'std:decimal';{functions}function main(){{{calls}}}",imports.into_iter().collect::<Vec<_>>().join(","));
 let directory=support::TempDirectory::new();let source=support::format_round_trip(&source);let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}
#[test]
fn decimal_storage_cannot_be_forged_or_implicitly_serialized(){
 for body in ["const value:Decimal={};","const value=unwrap(parseDecimal('1'));console.log(value.storage);","const value=unwrap(parseDecimal('1'));JSON.stringify(value);"]{
 let dir=support::TempDirectory::new();let source=format!("import {{Decimal,parseDecimal}} from 'std:decimal';function main(){{{body}}}");
 let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);assert!(!output.status.success(),"{source}");
 }
}
#[test]
fn decimal_package_exports_with_its_locked_offline_backend(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip("import {parseDecimal,format,divideExact,fromInt} from 'std:decimal';function main(){console.log(format(unwrap(divideExact(unwrap(parseDecimal('1')),fromInt(8)))));}"));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let home=dir.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked","--jobs","1"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
 support::success(&support::run(&mut command,180));let output=support::run(&mut std::process::Command::new(target.join("debug/program")),10);support::success(&output);assert_eq!(output.stdout,b"0.125\n");
 assert!(relocated.join("vendor/bigdecimal-0.4.11/LICENSE-MIT").exists());
}
