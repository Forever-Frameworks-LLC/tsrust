mod support;
#[test]
fn calendar_dates_preserve_month_end_rules_and_explicit_ownership(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/calendar-dates/billing.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);
 assert_eq!(output.stdout,b"2024-02-29\n");
}
#[test]
fn calendar_documentation_examples(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 for entry in catalog["std:date"].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(example["source"].as_str().unwrap());
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes(),"{}",entry["name"]);
 }}
}
#[test]
fn calendar_storage_cannot_be_forged_or_implicitly_serialized(){
 for body in ["const value:Date={};","const value=unwrap(parseDate('2024-02-29'));console.log(value.storage);","const value=unwrap(parseDate('2024-02-29'));JSON.stringify(value);"]{
 let dir=support::TempDirectory::new();let source=format!("import {{Date,parseDate}} from 'std:date';function main(){{{body}}}");
 let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);assert!(!output.status.success(),"{source}");
 }
}
#[test]
fn calendar_package_exports_with_its_locked_offline_backend(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip("import {parseDate,formatDate,addMonths} from 'std:date';function main(){console.log(formatDate(unwrap(addMonths(unwrap(parseDate('2024-01-31')),1))));}"));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let home=dir.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked","--jobs","1"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
 support::success(&support::run(&mut command,180));let output=support::run(&mut std::process::Command::new(target.join("debug/program")),10);support::success(&output);assert_eq!(output.stdout,b"2024-02-29\n");
 assert!(relocated.join("vendor/chrono-0.4.45/LICENSE.txt").exists());
}
