mod support;
#[test]
fn uuids_preserve_bytes_and_explicit_ownership(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/uuid/identifiers.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,b"67e55044-10b1-426f-9247-bb680e5fe0c8\n");
}
#[test]
fn uuid_documentation_examples_execute_as_one_native_program(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut imports=std::collections::BTreeMap::<String,std::collections::BTreeSet<String>>::new();let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
 for entry in catalog["std:uuid"].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
 let source=example["source"].as_str().unwrap();let (prefix,body)=source.split_once("function main()").unwrap();
 for import in prefix.split(';').filter(|part|!part.trim().is_empty()){
  let (names,module)=import.rsplit_once(" from ").unwrap();let start=names.find('{').unwrap();let end=names.find('}').unwrap();
  for name in names[start+1..end].split(','){imports.entry(module.trim().to_owned()).or_default().insert(name.trim().to_owned());}
 }
 functions.push_str(&format!("function example{count}(){body}\n"));calls.push_str(&format!("example{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;
 }}
 assert_eq!(count,9);let imports=imports.into_iter().map(|(module,names)|format!("import {{{}}} from {module};",names.into_iter().collect::<Vec<_>>().join(","))).collect::<String>();
 let source=format!("{imports}{functions}function main(){{{calls}}}");let directory=support::TempDirectory::new();let source=support::format_round_trip(&source);let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}
#[test]
fn uuid_storage_cannot_be_forged_or_implicitly_serialized(){
 for body in ["const value:Uuid={};","const value=unwrap(parseUuid('67e55044-10b1-426f-9247-bb680e5fe0c8'));console.log(value.storage);","const value=unwrap(parseUuid('67e55044-10b1-426f-9247-bb680e5fe0c8'));JSON.stringify(value);"]{
 let dir=support::TempDirectory::new();let source=format!("import {{Uuid,parseUuid}} from 'std:uuid';function main(){{{body}}}");let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);assert!(!output.status.success());
 }
}
#[test]
fn uuid_package_exports_with_its_locked_offline_backend(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip("import {randomUuid,version} from 'std:uuid';function main(){console.log(version(unwrap(randomUuid())));}"));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let home=dir.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked","--jobs","1"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
 support::success(&support::run(&mut command,180));let output=support::run(&mut std::process::Command::new(target.join("debug/program")),10);support::success(&output);assert_eq!(output.stdout,b"4\n");
 assert!(relocated.join("vendor/uuid-1.28.0/LICENSE-MIT").exists());
}
