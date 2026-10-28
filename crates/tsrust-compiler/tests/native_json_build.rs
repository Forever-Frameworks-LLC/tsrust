mod support;
#[path = "../src/cli/native_dependency_bundle.rs"]
#[expect(dead_code, reason = "Artifact-only integration fixture; CLI setup/materialization are exercised independently")]
mod native_dependency_bundle;

#[test]
fn json_runtime_links_automatically_and_survives_application_edits() {
 let directory=support::TempDirectory::new();let cache=directory.0.join("cache");
 let source=directory.0.join("main.tsr");let binary=directory.0.join(format!("program{}",std::env::consts::EXE_SUFFIX));
 for (index,value) in [42,43].into_iter().enumerate() {
  let text=format!(r#"import {{fromInt,parse}} from 'std:json';
import {{strictEqual}} from 'std:testing';
function main(){{
 match(parse('null secret')){{
  Ok(value)=>{{throw 'Expected invalid JSON';}},
  Err(error)=>{{strictEqual(error.code,'trailing-data');strictEqual(error.offset,5);strictEqual(error.message.includes('secret'),false);}}
 }}
 console.log(JSON.stringify(fromInt({value})));
}}"#);
  std::fs::write(&source,support::format_round_trip(&text)).unwrap();
  let output=support::run(support::cli().current_dir(&directory.0).arg("build").arg(&source).arg("-o").arg(&binary)
   .env("TSRUST_CACHE_DIR",&cache).env("TSRUST_BUILD_TIMINGS","1"),180);
  support::success(&output);
  assert!(String::from_utf8_lossy(&output.stderr).contains(if index==0{"runtime=miss"}else{"runtime=hit"}));
  let output=support::run(&mut std::process::Command::new(&binary),10);support::success(&output);
  assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),value.to_string());
 }
}

#[test]
fn invalid_artifact_members_are_rejected_before_writing() {
 let dir=support::TempDirectory::new();let packed=dir.0.join("artifacts");
 for (index,name) in ["../escape.rlib","/absolute.rlib","libtsrust_runtime.rlib"].iter().enumerate() {
  let mut bytes=b"TSRLIB3\0".to_vec();bytes.extend_from_slice(&(name.len() as u64).to_le_bytes());
  bytes.extend_from_slice(&10u64.to_le_bytes());bytes.extend_from_slice(name.as_bytes());
  if index!=2 {bytes.extend_from_slice(&[0;10]);}
  std::fs::write(&packed,bytes).unwrap();let target=dir.0.join(format!("unpack-{index}"));
  assert!(native_dependency_bundle::unpack(&packed,&target).is_err());assert!(!target.exists());
 }
}
