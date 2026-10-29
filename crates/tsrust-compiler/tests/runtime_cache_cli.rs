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

#[cfg(feature = "legacy-ui")]
#[test]
fn project_theme_edits_reuse_runtime_support() {
 let dir=support::TempDirectory::new();let root=&dir.0;let cache=root.join("cache");
 fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"theme-cache","build":{"entry":"main.tsrx","output":"program"},"ui":{"theme":"ui.theme.json","components":"components/ui","lock":"ui.lock.json"}}"#).unwrap();
 support::success(&support::run(support::cli().current_dir(root).args(["ui","add","tokens@1.0.0","--target","native","--offline"]),30));
 let source="import {withTheme,Text,document} from 'std:ui';import theme from './ui.theme.json';function main(){console.log(document(withTheme(<Text>Theme</Text>,theme),'UI'));}";
 let entry=root.join("main.tsrx");fs::write(&entry,support::format_round_trip(source)).unwrap();let binary=root.join("program");
 for (index,color) in ["#123456","#654321"].iter().enumerate() {
  fs::write(root.join("ui.theme.json"),tsrust_compiler::ui_project::theme::DEFAULT_JSON.replacen("#ffffff",color,1)).unwrap();
  let output=support::run(support::cli().args(["build",entry.to_str().unwrap(),"-o",binary.to_str().unwrap()]).env("TSRUST_CACHE_DIR",&cache).env("TSRUST_BUILD_TIMINGS","1"),120);support::success(&output);
  let log=String::from_utf8_lossy(&output.stderr);assert!(log.contains(if index==0{"runtime=miss"}else{"runtime=hit"}),"{log}");
  let output=support::run(&mut std::process::Command::new(&binary),10);support::success(&output);assert!(String::from_utf8(output.stdout).unwrap().contains(color));
 }
}

#[test]
#[cfg(unix)]
fn explicit_clean_counts_persistent_dependency_workspaces_and_preserves_lock_inodes() {
 use std::os::unix::fs::{MetadataExt,symlink};
 let dir=support::TempDirectory::new();let cache=dir.0.join("cache");let workspace=cache.join("runtime/build-v1/0123456789abcdef");let nested=workspace.join("native-support/vendor");fs::create_dir_all(&nested).unwrap();
 fs::write(workspace.join("identity"),"workspace").unwrap();fs::write(workspace.join(".lock"),"").unwrap();fs::write(nested.join("payload"),"dependency bytes").unwrap();
 let outside=dir.0.join("owner-file");fs::write(&outside,"preserve").unwrap();symlink(&outside,nested.join("linked")).unwrap();let inode=fs::metadata(workspace.join(".lock")).unwrap().ino();
 let output=support::run(support::cli().args(["cache","info"]).env("TSRUST_CACHE_DIR",&cache),20);support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("1 artifacts"));
 let output=support::run(support::cli().args(["cache","clean"]).env("TSRUST_CACHE_DIR",&cache),20);support::success(&output);assert!(!workspace.join("native-support").exists());assert!(!workspace.join("identity").exists());assert_eq!(fs::metadata(workspace.join(".lock")).unwrap().ino(),inode);assert_eq!(fs::read(outside).unwrap(),b"preserve");
}
