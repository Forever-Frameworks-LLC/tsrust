mod support;
#[path = "../src/native_dependency_bundle.rs"] mod native_dependency_bundle;
#[path = "../src/runtime_build.rs"] mod runtime_build;
fn backend_command() -> std::process::Command { support::rustc() }
fn backend_cargo_command() -> std::process::Command {
 let mut command=std::process::Command::new("rustup");
 command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo"]);command
}
#[test]
fn json_runtime_links_automatically_and_survives_application_edits() {
 let dir=support::TempDirectory::new();let cache=dir.0.join("cache");
 for (index,expected) in [(0,"42\n"),(1,"43\n")] {
  let temporary=dir.0.join(format!("build-{index}"));std::fs::create_dir(&temporary).unwrap();
  let source=format!("{}\nfn main() {{ let value=tsrust_native_json::parse(b\"{}\",tsrust_native_json::Limits{{bytes:100,depth:4,entries:4,nodes:10}}).unwrap(); println!(\"{{}}\",value); }}",tsrust_compiler::native_runtime::json_module(),42+index);
  let source=source.replace("fn main() {",r#"fn main() {
   let error=tsrust_native_json::parse(b"null secret",tsrust_native_json::Limits{bytes:100,depth:4,entries:4,nodes:10}).unwrap_err();
   assert_eq!(error.code,"trailing-data");assert_eq!(error.offset,5);
   assert!(!error.to_string().contains("secret"));
  "#);
  let mut prepared=(if index==0 {runtime_build::Prepared::new(&source,"test-pinned-backend",&cache,&temporary,false)} else {runtime_build::Prepared::for_dev(&source,"test-pinned-backend",&cache,&temporary,false)}).unwrap();
  assert_eq!(prepared.state,if index==0{"miss"}else{"hit"});
  if let Some(mut command)=prepared.command.take() {
   command.env("CARGO_HOME",dir.0.join("empty-cargo"));
   support::success(&support::run(&mut command,60));prepared.publish();
  }
  let binary=temporary.join("program");let mut command=backend_command();prepared.link(&mut command);
  command.arg("--edition=2024").arg(&prepared.application).arg("-o").arg(&binary);
  support::success(&support::run(&mut command,60));
  let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
  assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
  std::fs::remove_dir_all(temporary).unwrap();
  if index==1 {
   for (label,backend,release) in [("backend-change","different-backend",false),("profile-change","test-pinned-backend",true)] {
    let temporary=dir.0.join(label);std::fs::create_dir(&temporary).unwrap();
    let prepared=runtime_build::Prepared::new(&source,backend,&cache,&temporary,release).unwrap();
    assert_eq!(prepared.state,"miss");assert!(prepared.command.is_some());
   }
  }
 }
}

#[test]
fn invalid_artifact_members_are_rejected_before_writing() {
 let dir=support::TempDirectory::new();let packed=dir.0.join("artifacts");
 for (index,name) in ["../escape.rlib","/absolute.rlib","libtsrust_runtime.rlib"].iter().enumerate() {
  let mut bytes=b"TSRLIB1\0".to_vec();bytes.extend_from_slice(&(name.len() as u64).to_le_bytes());
  bytes.extend_from_slice(&10u64.to_le_bytes());bytes.extend_from_slice(name.as_bytes());
  if index!=2 {bytes.extend_from_slice(&[0;10]);}
  std::fs::write(&packed,bytes).unwrap();let target=dir.0.join(format!("unpack-{index}"));
  assert!(native_dependency_bundle::unpack(&packed,&target).is_err());assert!(!target.exists());
 }
}
