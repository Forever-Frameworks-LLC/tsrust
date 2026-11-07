mod support;
#[test]
fn form_and_multipart_factories_execute_through_aliases_and_stored_calls(){
 let source=support::format_round_trip(include_str!("../../../examples/http-client-forms/main.tsr"));
 let directory=support::TempDirectory::new();let path=directory.source(&source);
 support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
 let package=directory.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));
 std::fs::remove_file(path).unwrap();
 let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();
 let document=manifest.parse::<toml_edit::DocumentMut>().unwrap();
 assert!(document["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("http")));
 assert!(document["dependencies"]["reqwest"]["features"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("multipart")));
 assert!(relocated.join("vendor/form_urlencoded-1.2.2/LICENSE-MIT").is_file());
 let generated=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&generated).unwrap();
 std::fs::write(generated,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 assert!(target.is_absolute());
 let home=directory.0.join("empty-cargo-home");std::fs::create_dir(&home).unwrap();
 for release in [false,true]{
  let mut command=std::process::Command::new("rustup");
  command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated)
   .env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");
  if release{command.arg("--release");}
  support::success(&support::run(&mut command,180));
  let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));
  let output=support::run(std::process::Command::new(executable).current_dir(&relocated),30);support::success(&output);
  assert_eq!(output.stdout,b"form and multipart exact\n");assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 }
}
#[test]
fn form_borrow_and_multipart_single_owner_contracts_reject_invalid_arguments(){let prefix="import {requestForm,requestMultipart,FormField,MultipartPart} from 'std:http';import {fromArray,length} from 'std:bytes';";for body in ["requestForm('POST','http://localhost/',[{name:'x',value:1}]);","requestMultipart('POST','http://localhost/',[{name:'x',data:'not bytes'}]);","const parts:MultipartPart[]=[{name:'x',data:unwrap(fromArray([1]))}];requestMultipart('POST','http://localhost/',parts);console.log(parts.length);","const parts:MultipartPart[]=[];const make=requestMultipart;make('POST','http://localhost/',parts);console.log(parts.length);"]{let source=support::format_round_trip(&format!("{prefix}function main(){{{body}}}"));let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted {body}");}}
