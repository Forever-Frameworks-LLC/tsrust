mod support;
#[test]
fn temporal_values_preserve_nanoseconds_dst_and_owned_transport(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/date-time/appointments.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,b"2024-11-03T06:30:00Z\n");
}
#[test]
fn temporal_documentation_examples_execute_as_one_native_program(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let prior:[&str;15]=["parseDate","formatDate","format","addDays","addMonths","differenceInCalendarDays","compareAsc","isLeapYear","getYear","getMonth","getDate","startOfMonth","endOfMonth","Date","DateError"];
 let mut imports=std::collections::BTreeSet::new();let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
 for entry in catalog["std:date"].as_array().unwrap().iter().filter(|entry|!prior.contains(&entry["name"].as_str().unwrap())){
  for example in entry["examples"].as_array().unwrap(){let source=example["source"].as_str().unwrap();let (prefix,body)=source.split_once("function main()").unwrap();let start=prefix.find('{').unwrap();let end=prefix.find('}').unwrap();for name in prefix[start+1..end].split(','){imports.insert(name.trim().to_owned());}
   functions.push_str(&format!("function example{count}(){body}\n"));calls.push_str(&format!("example{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;
  }
 }
 assert_eq!(count,31);let source=format!("import {{{}}} from 'std:date';{functions}function main(){{{calls}}}",imports.into_iter().collect::<Vec<_>>().join(","));
 let directory=support::TempDirectory::new();let source=support::format_round_trip(&source);let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());
}
#[test]
fn temporal_values_reject_forging_and_implicit_json(){
 for name in ["Time","Instant","Duration","TimeZone"]{
  let dir=support::TempDirectory::new();let source=format!("import {{{name}}} from 'std:date';function main(){{const value:{name}={{}};}}");
  let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);assert!(!output.status.success(),"{name}");
 }
 let dir=support::TempDirectory::new();let source="import {parseISO} from 'std:date';function main(){const value=unwrap(parseISO('2024-01-01T00:00:00Z'));JSON.stringify(value);}";
 let output=support::run(support::cli().arg("check").arg(dir.source(source)),30);assert!(!output.status.success());
}
#[test]
fn temporal_package_exports_with_its_locked_offline_backend(){
 let dir=support::TempDirectory::new();let input=dir.source(&support::format_round_trip("import {parseDate,parseTime,parseTimeZone,resolveDateTime,formatISO} from 'std:date';function main(){console.log(formatISO(unwrap(resolveDateTime(unwrap(parseDate('2024-11-03')),unwrap(parseTime('01:30:00')),unwrap(parseTimeZone('America/New_York')),'later'))));}"));
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(&input).unwrap();let relocated=dir.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));let home=dir.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked","--jobs","1"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
 support::success(&support::run(&mut command,180));let output=support::run(&mut std::process::Command::new(target.join("debug/program")),10);support::success(&output);assert_eq!(output.stdout,b"2024-11-03T06:30:00Z\n");
 assert!(relocated.join("vendor/chrono-tz-0.10.4/LICENSE").exists());
}
