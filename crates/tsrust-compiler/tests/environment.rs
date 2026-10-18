mod support;
#[test]
fn environment_lookup_executes_with_owned_optional_results() {
 let dir=support::TempDirectory::new();
 let source=r#"import {get as lookup, EnvError} from 'std:env';import {strictEqual} from 'std:testing';
 function expect(name:string,expected:string|undefined){const saved=lookup;match(saved(name)){Ok(value)=>{strictEqual(value,expected);},Err(error)=>{throw error.message;}}console.log(name);}
 function main(){expect("TSRUST_ENV_EMPTY","");expect("TSRUST_ENV_TEXT","héllo 🦀");expect("TSRUST_ENV_MISSING",undefined);match(lookup("A=B")){Ok(_)=>{throw "accepted invalid name";},Err(error)=>{const typed:EnvError=error;strictEqual(typed.code,"InvalidName");}}}"#;
 std::fs::write(dir.0.join("main.tsr"),support::format_round_trip(source)).unwrap();
 for release in [false,true] {
  let mut command=support::cli();command.arg("run").arg(dir.0.join("main.tsr")).env("TSRUST_ENV_EMPTY","").env("TSRUST_ENV_TEXT","héllo 🦀").env_remove("TSRUST_ENV_MISSING");if release {command.arg("--release");}
  support::success(&support::run(&mut command,60));
 }
}

#[test]
fn environment_lookup_rejects_invalid_calls() {
 let dir=support::TempDirectory::new();
 for call in ["get()","get(1)","get('A','B')"] {
  std::fs::write(dir.0.join("main.tsr"),support::format_round_trip(&format!("import {{get}} from 'std:env';function main(){{{call};}}"))).unwrap();
  let result=support::run(support::cli().arg("check").arg(dir.0.join("main.tsr")),20);assert!(!result.status.success(),"accepted {call}");
 }
}
#[cfg(unix)]
#[test]
fn environment_lookup_reports_non_unicode_values() {
 use std::os::unix::ffi::OsStringExt;
 let dir=support::TempDirectory::new();
 let source="import {get} from 'std:env';import {strictEqual} from 'std:testing';function main(){match(get('TSRUST_ENV_RAW')){Ok(_)=>{throw 'accepted invalid Unicode';},Err(error)=>{strictEqual(error.code,'NotUnicode');}}}";
 std::fs::write(dir.0.join("main.tsr"),support::format_round_trip(source)).unwrap();
 for release in [false,true] {
  let mut command=support::cli();command.arg("run").arg(dir.0.join("main.tsr")).env("TSRUST_ENV_RAW",std::ffi::OsString::from_vec(vec![0xff]));if release {command.arg("--release");}support::success(&support::run(&mut command,60));
 }
}

#[test]
fn process_arguments_are_independent_and_cli_forwards_exact_values() {
 let dir=support::TempDirectory::new();
 let source=r#"import {args,get} from 'std:env';import {strictEqual} from 'std:testing';function main(){const first=unwrap(args());strictEqual(first.length,4);strictEqual(first[0],"--help");strictEqual(first[1],"");strictEqual(first[2],"two words");strictEqual(first[3],"héllo 🦀");first[0]="changed";const second=unwrap(args());strictEqual(second[0],"--help");const value=unwrap(get("TSRUST_ARG_TEST"));if(value===undefined){throw "missing environment";}strictEqual(value,"ready");}"#;
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(source)).unwrap();
 let output=support::run(support::cli().arg("run").arg(&path).args(["--","--help","","two words","héllo 🦀"]).env("TSRUST_ARG_TEST","ready"),60);support::success(&output);
 std::fs::write(&path,"import {args} from 'std:env';function main(){args(1);}").unwrap();
 assert!(!support::run(support::cli().arg("check").arg(&path),20).status.success());
}

#[cfg(unix)]
#[test]
fn process_arguments_reject_non_unicode_without_exposing_values() {
 use std::os::unix::ffi::OsStringExt;
 let dir=support::TempDirectory::new();
 let source=r#"import {args} from 'std:env';import {strictEqual} from 'std:testing';function main(){match(args()){Ok(_)=>{throw "expected Unicode failure";},Err(error)=>{strictEqual(error.code,"NotUnicode");strictEqual(error.message,"Process argument is not valid Unicode");}}}"#;
 let path=dir.0.join("main.tsr");std::fs::write(&path,support::format_round_trip(source)).unwrap();let binary=dir.0.join("program");
 support::success(&support::run(support::cli().arg("build").arg(&path).arg("-o").arg(&binary),60));
 let output=support::run(std::process::Command::new(binary).arg(std::ffi::OsString::from_vec(vec![b's',b'e',b'c',b'r',b'e',b't',0xff])),20);support::success(&output);assert!(output.stdout.is_empty()&&output.stderr.is_empty());
}
