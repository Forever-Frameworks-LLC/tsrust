//! Copy the current Cargo graph out of the shared target while its lock is held.
use std::{collections::BTreeMap,fs,io::{BufRead,BufReader},path::{Component,Path,PathBuf},process::Command};
use crate::native_workspace::Workspace;
const SHARED_TARGET_ID:&[u8]=b"shared-cargo-target-1";

pub(super) struct Build {
 pub target:PathBuf,
 root:PathBuf,
 explicit:bool,
 manifest:PathBuf,
 library_name:String,
 stdout:PathBuf,
 destination:PathBuf,
 workspace:Option<Workspace>,
 lock:Option<fs::File>,
 #[cfg(test)] pub sources:BTreeMap<String,PathBuf>,
}
impl Build {
 pub fn new(root:&Path,manifest:PathBuf,destination:PathBuf,stdout:PathBuf,explicit:Option<&Path>,library_name:&str)->Result<Self,String>{
  let target=match explicit{Some(path)=>{let path=crate::absolute(path)?;fs::create_dir_all(&path).map_err(|e|e.to_string())?;path.canonicalize().map_err(|e|e.to_string())?},None=>Workspace::location(root,SHARED_TARGET_ID).join("target")};
  Ok(Self{target,root:root.to_owned(),explicit:explicit.is_some(),manifest,library_name:library_name.into(),stdout,destination,workspace:None,lock:None,#[cfg(test)] sources:BTreeMap::new()})
 }
 // File stdout avoids pipe backpressure while dev supervises edits and cancellation.
 pub fn capture(&self,command:&mut Command)->Result<(),String>{command.stdout(fs::File::create(&self.stdout).map_err(|e|e.to_string())?);Ok(())}
 pub fn acquire(&mut self,blocking:bool)->Result<bool,String>{
  if self.lock.is_some() || self.workspace.is_some(){return Ok(true);}
  if !self.explicit {
   self.workspace=Workspace::acquire(&self.root,SHARED_TARGET_ID,!blocking)?;
   return Ok(self.workspace.is_some());
  }
  fs::create_dir_all(&self.target).map_err(|e|e.to_string())?;
  let canonical=self.target.canonicalize().map_err(|e|e.to_string())?;
  // An explicit path may name the cache-owned target (including via symlinks).
  // Use its existing workspace lock so cache clean and default builds agree.
  // Cache clean removes identity, so recognize the exact owned slot by its
  // location and let Workspace recreate/check identity under the retained lock.
  if let Some(parent)=canonical.parent() && let Some(root)=parent.ancestors().nth(3) && Workspace::location(root,SHARED_TARGET_ID)==parent {
   self.workspace=Workspace::acquire(root,SHARED_TARGET_ID,!blocking)?;return Ok(self.workspace.is_some());
  }
  let file=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(canonical.join(".tsrust-runtime.lock")).map_err(|e|e.to_string())?;
  if blocking{file.lock().map_err(|e|e.to_string())?;}else{match file.try_lock(){Ok(())=>{},Err(fs::TryLockError::WouldBlock)=>return Ok(false),Err(error)=>return Err(error.to_string())}}
  self.lock=Some(file);Ok(true)
 }
 pub fn diagnostics(&self)->String {
  let Ok(file)=fs::File::open(&self.stdout) else{return String::new();};
  BufReader::new(file).lines().filter_map(Result::ok).filter_map(|line|serde_json::from_str::<serde_json::Value>(&line).ok()).filter(|message|message["reason"]=="compiler-message").filter_map(|message|message["message"]["rendered"].as_str().map(str::to_owned)).collect()
 }
 pub fn finish(&mut self)->Result<PathBuf,String>{
  if self.workspace.is_none() && self.lock.is_none(){return Err("Shared Cargo target must remain locked through artifact staging".into());}
  let artifacts=reported(&self.stdout,&self.target,&self.manifest,&self.library_name)?;
  // The graph workspace is also locked. Independent copies survive later builds
  // replacing artifacts or cleaning the shared target.
  let staging=self.destination.with_extension("staging");
  if staging.exists(){fs::remove_dir_all(&staging).map_err(|e|e.to_string())?;}
  fs::create_dir(&staging).map_err(|e|e.to_string())?;
  let result=(||{
   for (name,source) in &artifacts {fs::copy(source,staging.join(name)).map_err(|e|e.to_string())?;}
   crate::native_dependency_bundle::restore_import_libraries(&staging).map_err(|e|e.to_string())?;
   if self.destination.exists(){fs::remove_dir_all(&self.destination).map_err(|e|e.to_string())?;}
   fs::rename(&staging,&self.destination).map_err(|e|e.to_string())?;Ok(())
  })();
  if let Err(error)=result{let _=fs::remove_dir_all(staging);return Err(error);}
  #[cfg(test)] {self.sources=artifacts;}
  self.workspace=None;self.lock=None;
  Ok(self.destination.clone())
 }
}

fn reported(stdout:&Path,target:&Path,manifest:&Path,library_name:&str)->Result<BTreeMap<String,PathBuf>,String>{
 let target=target.canonicalize().map_err(|e|e.to_string())?;
 let manifest=manifest.canonicalize().map_err(|e|e.to_string())?;
 let file=fs::File::open(stdout).map_err(|e|e.to_string())?;
 let mut artifacts=BTreeMap::new();let mut finished=false;let mut runtime=None;
 for line in BufReader::new(file).lines(){
  let line=line.map_err(|e|e.to_string())?;
  // Cargo permits other tools to print ordinary text on stdout.
  if !line.trim_start().starts_with('{'){continue;}
  let message:serde_json::Value=serde_json::from_str(&line).map_err(|e|format!("Invalid Cargo artifact JSON: {e}"))?;
  if message["reason"]=="build-finished"{finished=message["success"].as_bool()==Some(true);continue;}
  if message["reason"]!="compiler-artifact"{continue;}
  let kinds=message["target"]["kind"].as_array().ok_or("Cargo artifact has no target kinds")?;
  let proc_macro=kinds.iter().any(|kind|kind=="proc-macro");
  if !proc_macro && !kinds.iter().any(|kind|matches!(kind.as_str(),Some("lib"|"rlib"|"dylib"|"staticlib"))){continue;}
  let root=message["target"]["name"]==library_name && message["manifest_path"].as_str().is_some_and(|path|Path::new(path).canonicalize().ok().as_ref()==Some(&manifest));
  for value in message["filenames"].as_array().ok_or("Cargo artifact has no filenames")?{
   let source=Path::new(value.as_str().ok_or("Cargo artifact filename is not a string")?);
   let extension=source.extension().and_then(|part|part.to_str()).unwrap_or("");
   if !matches!(extension,"rlib"|"lib") && !(proc_macro && matches!(extension,"dll"|"so"|"dylib")){continue;}
   if !source.is_absolute() || source.components().any(|part|matches!(part,Component::ParentDir)){return Err("Cargo artifact path must be absolute without traversal".into());}
   let canonical=source.canonicalize().map_err(|e|e.to_string())?;
   if !canonical.starts_with(&target) || !canonical.is_file(){return Err("Cargo artifact is outside the selected shared target or is not a file".into());}
   let name=source.file_name().and_then(|part|part.to_str()).ok_or("Cargo artifact has no UTF-8 filename")?;
   if root && extension=="rlib" {
    if runtime.replace(canonical.clone()).is_some(){return Err("Cargo reported multiple runtime libraries".into());}
    continue;
   }
   if name=="libtsrust_runtime.rlib"{return Err("Unrelated Cargo artifact uses the runtime library name".into());}
   if let Some(previous)=artifacts.insert(name.to_owned(),canonical.clone()) && previous!=canonical{return Err(format!("Conflicting Cargo artifact filename: {name}"));}
  }
 }
 if !finished{return Err("Cargo did not report a successful completed build".into());}
 artifacts.insert("libtsrust_runtime.rlib".into(),runtime.ok_or("Cargo did not report the selected runtime library")?);
 Ok(artifacts)
}

#[cfg(test)] mod tests {
 use super::*;
 use serde_json::{json,Value};
 struct Fixture{root:PathBuf,build:Build}
 impl Fixture{
  fn new()->Self{
   let root=std::env::temp_dir().join(format!("tsrust-artifacts-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
   fs::create_dir(&root).unwrap();let manifest=root.join("Cargo.toml");fs::write(&manifest,"[package]").unwrap();
   let target=root.join("target");fs::create_dir(&target).unwrap();
   let build=Build::new(&root,manifest,root.join("artifacts"),root.join("stdout"),Some(&target),"tsrust_runtime").unwrap();
   Self{root,build}
  }
  fn artifact(&self,name:&str,kind:&str,filename:&str)->Value{
   let source=self.build.target.join(filename);fs::write(&source,filename).unwrap();
   json!({"reason":"compiler-artifact","manifest_path":self.build.manifest,"target":{"name":name,"kind":[kind]},"filenames":[source],"fresh":true})
  }
  fn messages(&self,rows:&[Value]){let mut text=String::from("ordinary Cargo tool output\n");for row in rows{text.push_str(&row.to_string());text.push('\n');}text.push_str("{\"reason\":\"build-finished\",\"success\":true}\n");fs::write(&self.build.stdout,text).unwrap();}
 }
 impl Drop for Fixture{fn drop(&mut self){let _=fs::remove_dir_all(&self.root);}}
 #[test] fn fresh_exact_graph_staging_is_immutable_and_excludes_unreported_files(){
  let mut fixture=Fixture::new();
  let root=fixture.artifact("tsrust_runtime","lib","reported-runtime.rlib");
  let dependency=fixture.artifact("serde_json","lib","libserde_json-fresh.rlib");
  let macro_library=fixture.artifact("derive","proc-macro",if cfg!(windows){"derive.dll"}else if cfg!(target_os="macos"){"libderive.dylib"}else{"libderive.so"});
  let ignored=fixture.artifact("build-script-build","custom-build","build-script.exe");
  fs::write(fixture.build.target.join("libunrelated.rlib"),"unrelated").unwrap();
  fixture.messages(&[root,dependency,macro_library,ignored]);
  assert!(fixture.build.finish().is_err());assert!(fixture.build.acquire(false).unwrap());
  let directory=fixture.build.finish().unwrap();
  assert_eq!(fs::read(directory.join("libtsrust_runtime.rlib")).unwrap(),b"reported-runtime.rlib");
  assert!(directory.join("libserde_json-fresh.rlib").is_file());assert_eq!(fs::read_dir(&directory).unwrap().count(),3);
  fs::write(fixture.build.target.join("reported-runtime.rlib"),"next graph").unwrap();
  assert_eq!(fs::read(directory.join("libtsrust_runtime.rlib")).unwrap(),b"reported-runtime.rlib");
  let packed=fixture.root.join("packed");crate::native_dependency_bundle::pack(&directory.join("libtsrust_runtime.rlib"),&directory,&packed).unwrap();
  let restored=fixture.root.join("restored");crate::native_dependency_bundle::unpack(&packed,&restored).unwrap();
  assert!(!restored.join("libunrelated.rlib").exists());assert_eq!(fs::read(restored.join("libtsrust_runtime.rlib")).unwrap(),b"reported-runtime.rlib");
 }
 #[test] fn selected_graph_target_is_exact_and_stages_under_the_stable_external_alias(){
  let mut fixture=Fixture::new();fixture.build.library_name="tsrust_runtime_graph_a".into();
  let selected=fixture.artifact("tsrust_runtime_graph_a","lib","libtsrust_runtime_graph_a.rlib");
  fixture.messages(&[selected]);assert!(fixture.build.acquire(false).unwrap());let directory=fixture.build.finish().unwrap();
  assert_eq!(fs::read(directory.join("libtsrust_runtime.rlib")).unwrap(),b"libtsrust_runtime_graph_a.rlib");
  assert!(!directory.join("libtsrust_runtime_graph_a.rlib").exists());
  let wrong=fixture.artifact("tsrust_runtime_graph_b","lib","libtsrust_runtime_graph_b.rlib");fixture.messages(&[wrong]);
  assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,&fixture.build.library_name).unwrap_err().contains("selected runtime"));
 }
 #[test] fn artifact_metadata_rejects_malformed_traversal_outside_and_wrong_runtime(){
  let fixture=Fixture::new();let root=fixture.artifact("tsrust_runtime","lib","runtime.rlib");
  fs::write(&fixture.build.stdout,"{bad json\n").unwrap();assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").unwrap_err().contains("Invalid Cargo"));
  for path in [fixture.root.join("outside.rlib"),fixture.build.target.join("../outside.rlib")] {
   fs::write(fixture.root.join("outside.rlib"),"outside").unwrap();let mut message=root.clone();message["filenames"]=json!([path]);fixture.messages(&[message]);assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").is_err());
  }
  let mut wrong=root.clone();wrong["manifest_path"]=json!(fixture.root.join("other-manifest"));fixture.messages(&[wrong]);assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").unwrap_err().contains("selected runtime"));
  fixture.messages(&[root.clone(),root.clone()]);assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").unwrap_err().contains("multiple runtime"));
  fs::write(&fixture.build.stdout,format!("{root}\n")).unwrap();assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").unwrap_err().contains("completed build"));
 }
 #[cfg(unix)] #[test] fn artifact_metadata_rejects_symlink_escape(){
  let fixture=Fixture::new();let mut root=fixture.artifact("tsrust_runtime","lib","runtime.rlib");
  let outside=fixture.root.join("outside.rlib");fs::write(&outside,"outside").unwrap();let alias=fixture.build.target.join("escape.rlib");std::os::unix::fs::symlink(outside,&alias).unwrap();root["filenames"]=json!([alias]);fixture.messages(&[root]);
  assert!(reported(&fixture.build.stdout,&fixture.build.target,&fixture.build.manifest,"tsrust_runtime").unwrap_err().contains("outside the selected"));
 }
 #[test] fn target_lock_serializes_explicit_default_and_alias_paths_and_releases_on_drop(){
  let fixture=Fixture::new();let mut first=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("one"),fixture.root.join("one.stdout"),None,"tsrust_runtime").unwrap();
  assert!(first.acquire(false).unwrap());fs::create_dir_all(&first.target).unwrap();
  let mut second=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("two"),fixture.root.join("two.stdout"),Some(&first.target),"tsrust_runtime").unwrap();
  assert!(!second.acquire(false).unwrap());drop(first);
  // Cache clean retains .lock but removes identity. The explicit alias must
  // still serialize with a default build recreating that identity afterward.
  fs::remove_file(second.target.parent().unwrap().join("identity")).unwrap();
  assert!(second.acquire(false).unwrap());assert_eq!(fs::read(second.target.parent().unwrap().join("identity")).unwrap(),SHARED_TARGET_ID);
  let mut third=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("three"),fixture.root.join("three.stdout"),None,"tsrust_runtime").unwrap();
  assert!(!third.acquire(false).unwrap());drop(second);assert!(third.acquire(false).unwrap());
  let mut external=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("external"),fixture.root.join("external.stdout"),Some(&fixture.build.target),"tsrust_runtime").unwrap();
  let mut competing=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("competing"),fixture.root.join("competing.stdout"),Some(&fixture.build.target),"tsrust_runtime").unwrap();
  assert!(external.acquire(false).unwrap());assert!(!competing.acquire(false).unwrap());drop(external);assert!(competing.acquire(false).unwrap());
  #[cfg(unix)] {let alias=fixture.root.join("alias");std::os::unix::fs::symlink(&third.target,&alias).unwrap();let mut aliased=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("four"),fixture.root.join("four.stdout"),Some(&alias),"tsrust_runtime").unwrap();assert!(!aliased.acquire(false).unwrap());drop(third);assert!(aliased.acquire(false).unwrap());}
 }
 #[test] fn diagnostics_preserve_rendered_errors_and_relative_target_is_absolute(){
  let fixture=Fixture::new();fs::write(&fixture.build.stdout,json!({"reason":"compiler-message","message":{"rendered":"error: concrete Rust failure\n"}}).to_string()).unwrap();
  assert_eq!(fixture.build.diagnostics(),"error: concrete Rust failure\n");
  let absolute=fixture.root.join("relative-target");
  #[cfg(unix)] let relative={let cwd=std::env::current_dir().unwrap();let mut path=PathBuf::new();for _ in cwd.components().skip(1){path.push("..");}path.push(absolute.strip_prefix("/").unwrap());path};
  #[cfg(not(unix))] let relative=absolute;
  let build=Build::new(&fixture.root,fixture.build.manifest.clone(),fixture.root.join("unused"),fixture.root.join("unused.stdout"),Some(&relative),"tsrust_runtime").unwrap();
  assert!(build.target.is_absolute());assert_eq!(build.target,fixture.root.join("relative-target").canonicalize().unwrap());
 }
 #[cfg(unix)] #[test] fn file_capture_does_not_block_a_polled_child_with_large_stdout(){
  let fixture=Fixture::new();let mut command=Command::new("/bin/sh");
  command.args(["-c","i=0; while [ $i -lt 4096 ]; do printf '%0256d\\n' 0; i=$((i+1)); done"]);fixture.build.capture(&mut command).unwrap();
  let mut child=command.spawn().unwrap();let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
  loop{if let Some(status)=child.try_wait().unwrap(){assert!(status.success());break;}if std::time::Instant::now()>deadline{let _=child.kill();let _=child.wait();panic!("file stdout blocked child supervision");}std::thread::sleep(std::time::Duration::from_millis(10));}
  assert!(fs::metadata(&fixture.build.stdout).unwrap().len()>1024*1024);
 }
}
