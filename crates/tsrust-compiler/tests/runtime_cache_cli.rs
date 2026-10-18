mod support;
use std::fs;
#[test]
fn application_edits_reuse_runtime_but_optimization_changes_do_not() {
 let dir=support::TempDirectory::new();let entry=dir.0.join("main.tsr");let binary=dir.0.join("program");let cache=dir.0.join("cache");
 for release in [false,true] {
  for (index,value) in ["first","second"].iter().enumerate() {
   let source=format!("import {{basename}} from 'std:path';function main(){{console.log(basename('/tmp/{value}'));}}");
   fs::write(&entry,support::format_round_trip(&source)).unwrap();
   let mut command=support::cli();command.arg("build").arg(&entry).arg("-o").arg(&binary).env("TSRUST_CACHE_DIR",&cache).env("TSRUST_BUILD_TIMINGS","1");if release {command.arg("--release");}
   let output=support::run(&mut command,60);support::success(&output);
   let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains(if index==0{"runtime=miss"}else{"runtime=hit"}),"{stderr}");
   let output=support::run(&mut std::process::Command::new(&binary),20);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),format!("{value}\n"));
  }
 }
 assert_eq!(fs::read_dir(cache.join("runtime/v2")).unwrap().filter(|entry|entry.as_ref().unwrap().path().is_dir()).count(),2,"one runtime per optimization profile");
}

#[test]
fn repeated_project_runs_reuse_default_persistent_cache() {
 let dir=support::TempDirectory::new();
 support::success(&support::run(support::cli().current_dir(&dir.0).args(["new","counter"]),20));
 let root=dir.0.join("counter");
 for expected in ["cache=miss", "cache=hit"] {
  let output=support::run(support::cli().current_dir(&root).arg("run").env_remove("TSRUST_CACHE_DIR").env("TSRUST_BUILD_TIMINGS","1"),60);
  support::success(&output);
  assert!(String::from_utf8_lossy(&output.stderr).contains(expected),"{}",String::from_utf8_lossy(&output.stderr));
  assert_eq!(output.stdout,b"Hello from counter!\n");
 }
}
