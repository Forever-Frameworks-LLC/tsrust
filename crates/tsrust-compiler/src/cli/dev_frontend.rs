//! A bounded, persistent frontend worker. Cancelled responses are discarded by callers.
use std::{path::PathBuf,sync::mpsc::{self,Receiver,SyncSender},time::{Duration,Instant}};
use tsrust_compiler::{project::{FileSourceProvider,SourceProvider},project_cli::CheckedInput,project_session::{ProjectSession,SessionStats}};
pub(crate) struct Output {pub rust:String,pub features:Vec<&'static str>,pub identity:Vec<u8>,pub stats:SessionStats,pub check:Duration,pub emit:Duration}
pub(crate) struct Request {pub entry:PathBuf,pub target:&'static str,pub sources:Snapshot,pub reply:mpsc::Sender<Result<Output,String>>}
/// Watcher-owned bytes and captured entry-graph resolutions remain fixed for a
/// request, including imported dependencies outside the ordinary watch tree.
pub(crate) struct Snapshot {pub root:PathBuf,pub files:crate::dev_watch::Inputs}
impl Snapshot {
 fn normalize(&self,path:&std::path::Path)->PathBuf {
  let mut normalized=PathBuf::new();
  for component in path.components(){match component {std::path::Component::CurDir=>{},std::path::Component::ParentDir=>{normalized.pop();},other=>normalized.push(other.as_os_str())}}
  normalized
 }
 fn captured(&self,path:&std::path::Path)->bool {
  (path.extension().is_some_and(|ext|matches!(ext.to_str(),Some("tsr"|"tsrx"|"rs")))||path.file_name().is_some_and(|name|name=="tsrust.config.json"))&&path.strip_prefix(&self.root).is_ok_and(|relative|!relative.components().any(|part|matches!(part.as_os_str().to_str(),Some(".git"|"node_modules"|"target"))))
 }
}
impl SourceProvider for Snapshot {
 fn resolve(&self,importer:Option<&tsrust_compiler::project::SourceId>,path:&str)->Result<tsrust_compiler::project::SourceId,String> {
  use tsrust_compiler::{project::SourceId,project_resolution};
  if let Some(result)=self.files.resolutions.get(&(importer.cloned(),path.to_owned())){return result.clone();}
  let candidates=if let Some(importer)=importer {
   let aliases=project_resolution::discover(std::path::Path::new(&importer.0),|path|{
    let path=self.normalize(path);
    if let Some(contents)=self.files.get(&path){return String::from_utf8(contents.bytes().to_vec()).map(Some).map_err(|error|error.to_string());}
    if self.captured(&path){return Ok(None);}
    match std::fs::read_to_string(&path){Ok(text)=>Ok(Some(text)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(format!("{}: {error}",path.display()))}
   })?;
   project_resolution::candidates(std::path::Path::new(&importer.0),path,aliases.as_ref())?
  }else{vec![PathBuf::from(path)]};
  for candidate in &candidates {
   let candidate=self.normalize(candidate);
   if self.files.contains_key(&candidate){return Ok(SourceId(candidate.to_str().ok_or("Source path must be valid UTF-8.")?.into()));}
   if !self.captured(&candidate)&&candidate.is_file(){return FileSourceProvider.resolve(None,candidate.to_str().ok_or("Source path must be valid UTF-8.")?);}
  }
  Err(format!("Source {path} is absent from this build snapshot; tried {}",candidates.iter().map(|path|path.display().to_string()).collect::<Vec<_>>().join(", ")))
 }
 fn read(&self,id:&tsrust_compiler::project::SourceId)->Result<String,String> {
  let path=std::path::Path::new(&id.0);
  if let Some(error)=self.files.failures.get(path){return Err(error.clone());}
  match self.files.get(path) {
   Some(contents)=>String::from_utf8(contents.bytes().to_vec()).map_err(|e|e.to_string()),
   None if self.captured(path)=>Err(format!("{} is absent from this build snapshot",path.display())),
   None=>FileSourceProvider.read(id),
  }
 }
}
pub(crate) struct Worker {pub requests:SyncSender<Request>}
impl Worker {
 pub fn new()->Result<Self,String> {
  let (requests,rx)=mpsc::sync_channel::<Request>(1);
  std::thread::Builder::new().name("dev-frontend".into()).spawn(move||run(rx)).map_err(|e|e.to_string())?;
  Ok(Self{requests})
 }
}
fn run(requests:Receiver<Request>) {
 let mut session=ProjectSession::default();
 for request in requests {let result=compile_target(&mut session,&request.entry,&request.sources,request.target);let _=request.reply.send(result);}
}
#[cfg(test)]
fn compile(session:&mut ProjectSession,entry:&std::path::Path,sources:&Snapshot)->Result<Output,String> {
 compile_target(session,entry,sources,"native")
}
fn compile_target(session:&mut ProjectSession,entry:&std::path::Path,sources:&Snapshot,target:&str)->Result<Output,String>{
 let started=Instant::now();
 let id=sources.resolve(None,entry.to_str().ok_or("Source path must be valid UTF-8.")?)?;
 let source=sources.read(&id)?;
 let analysis=tsrust_compiler::project_cli::check_input_cached_with_provider(entry,&source,session,sources).map_err(|error|error.render_with_provider(&entry.display().to_string(),&source,false,sources))?;
 let check=started.elapsed();let started=Instant::now();
 let(rust,features)=analysis.emit_rust_with_native_features();let emit=started.elapsed();
 if features.contains(&"ui-native"){tsrust_compiler::ui_project::require_target(entry,target,|path|sources.read(&tsrust_compiler::project::SourceId(path.to_string_lossy().into_owned())).map(Some).or(Ok(None)))?;}
 let mut identity=Vec::new();
 match &analysis {
  CheckedInput::Project(project)=>for(id,module)in &project.graph.modules {for value in [id.0.as_bytes(),module.source.text.as_bytes()] {identity.extend_from_slice(&(value.len()as u64).to_le_bytes());identity.extend_from_slice(value);}},
  CheckedInput::Single(_)=>{let id=sources.resolve(None,entry.to_str().ok_or("Source path must be valid UTF-8.")?)?;identity.extend_from_slice(&(id.0.len()as u64).to_le_bytes());identity.extend_from_slice(id.0.as_bytes());identity.extend_from_slice(source.as_bytes());},
 }
 let stats=if matches!(analysis,CheckedInput::Project(_)){session.stats()}else{SessionStats::default()};
 Ok(Output{rust,features,identity,stats,check,emit})
}

#[cfg(test)]
mod tests {
 use super::*;
 fn write(path:&std::path::Path,text:&str) {
  let formatted=tsrust_compiler::format(text).unwrap();assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);std::fs::write(path,formatted).unwrap();
 }
 #[test]
 fn external_and_symlink_dependencies_are_captured_and_reconciled() {
  let dir=crate::TempDirectory::new().unwrap();let base=std::fs::canonicalize(&dir.0).unwrap();let root=base.join("app");std::fs::create_dir(&root).unwrap();
  let external=base.join("dependency.tsr");write(&external,"export function value():int{return 1;}");
  std::os::unix::fs::symlink(&external,root.join("linked.tsr")).unwrap();
  std::fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"app","build":{"entry":"main.tsr","output":"app"},"paths":{"@dep":"./linked.tsr"}}"#).unwrap();
  let entry=root.join("main.tsr");write(&entry,"import {value} from '@dep';function main(){console.log(value());}");
  let mut watcher=crate::dev_watch::Watcher::default();let first=watcher.snapshot(&root).unwrap();let frozen=Snapshot{root:root.clone(),files:first.clone()};let mut session=ProjectSession::default();
  let original=compile(&mut session,&entry,&frozen).unwrap().rust;
  write(&external,"export function value():int{return 2;}");
  assert_eq!(compile(&mut session,&entry,&frozen).unwrap().rust,original);
  let second=watcher.snapshot(&root).unwrap();assert_ne!(first,second);
  assert_ne!(compile(&mut session,&entry,&Snapshot{root:root.clone(),files:second}).unwrap().rust,original);
  let replacement=base.join("replacement.tsr");write(&replacement,"export function value():int{return 3;}");std::fs::remove_file(root.join("linked.tsr")).unwrap();std::os::unix::fs::symlink(replacement,root.join("linked.tsr")).unwrap();
  assert_eq!(compile(&mut session,&entry,&frozen).unwrap().rust,original);
  assert_ne!(compile(&mut session,&entry,&Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()}).unwrap().rust,original);
  std::fs::write(base.join("replacement.tsr"),[0xff]).unwrap();
  let invalid=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};
  let error=compile(&mut session,&entry,&invalid).err().unwrap();
  write(&base.join("replacement.tsr"),"export function value():int{return 4;}");
  assert_eq!(compile(&mut session,&entry,&invalid).err().unwrap(),error);
  assert!(compile(&mut session,&entry,&Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()}).is_ok());
 }
 #[test]
 fn aliases_and_candidate_selection_remain_frozen_after_disk_changes() {
  let dir=crate::TempDirectory::new().unwrap();let root=std::fs::canonicalize(&dir.0).unwrap();let entry=root.join("main.tsr");
  std::fs::create_dir_all(root.join("src/component")).unwrap();
  std::fs::write(root.join("tsrust.config.json"),r#"{"paths":{"@/*":"./src/*"}}"#).unwrap();
  write(&entry,"import {value} from '@/component';function main(){console.log(value());}");
  write(&root.join("src/component/index.tsr"),"export function value():int{return 1;}");
  let mut watcher=crate::dev_watch::Watcher::default();let snapshot=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};let mut session=ProjectSession::default();
  let original=compile(&mut session,&entry,&snapshot).unwrap().rust;
  write(&root.join("src/component.tsr"),"export function value():int{return 2;}");
  assert_eq!(compile(&mut session,&entry,&snapshot).unwrap().rust,original);
  std::fs::write(root.join("tsrust.config.json"),r#"{"paths":{"@/*":"./missing/*"}}"#).unwrap();
  std::fs::remove_file(root.join("src/component/index.tsr")).unwrap();
  assert_eq!(compile(&mut session,&entry,&snapshot).unwrap().rust,original);
  let current=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};assert!(compile(&mut session,&entry,&current).is_err());
 }
 #[test]
 fn nested_config_is_captured_and_changed_mappings_invalidate_snapshot() {
  let dir=crate::TempDirectory::new().unwrap();let root=std::fs::canonicalize(&dir.0).unwrap();std::fs::create_dir(root.join("nested")).unwrap();
  std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();let config=root.join("nested/tsrust.config.json");std::fs::write(&config,r#"{"paths":{"@/*":"./*"}}"#).unwrap();
  let mut watcher=crate::dev_watch::Watcher::default();let first=watcher.snapshot(&root).unwrap();assert!(first.contains_key(&config));
  std::fs::write(config,r#"{"paths":{"@/*":"../*"}}"#).unwrap();assert_ne!(first,watcher.snapshot(&root).unwrap());
 }
 #[test]
 fn frozen_sources_preserve_emission_and_diagnostic_lines_after_disk_edits() {
  let dir=crate::TempDirectory::new().unwrap();let root=std::fs::canonicalize(&dir.0).unwrap();let entry=root.join("main.tsr");let dep=root.join("dep.tsr");
  write(&entry,"import {value} from './dep.tsr';function main(){console.log(value());}");write(&dep,"export function value():int{return 1;}");
  std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
  let mut watcher=crate::dev_watch::Watcher::default();let sources=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};let mut session=ProjectSession::default();
  let later=root.join("later.tsr");write(&later,"export function later():int{return 1;}");
  assert!(sources.resolve(None,later.to_str().unwrap()).is_err());
  std::fs::remove_file(later).unwrap();
  let original=compile(&mut session,&entry,&sources).unwrap().rust;
  write(&entry,"function main(){console.log('changed');}");write(&dep,"export function value():int{return 2;}");
  assert_eq!(compile(&mut session,&entry,&sources).unwrap().rust,original);
  write(&entry,"import {value} from './dep.tsr';function main(){console.log(value());}");write(&dep,"export function value():int{return 'snapshot-error';}");
  let invalid=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};
  write(&dep,"export function value():int{return 99;}");
  let error=compile(&mut session,&entry,&invalid).err().unwrap();assert!(error.contains("snapshot-error"),"{error}");assert!(!error.contains("return 99"),"{error}");
 }
 #[test]
 fn rust_body_edits_refresh_captured_dev_output() {
  let dir=crate::TempDirectory::new().unwrap();let root=dir.0.canonicalize().unwrap();let entry=root.join("main.tsr");
  std::fs::write(root.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"app","build":{"entry":"main.tsr","output":"app"}}"#).unwrap();
  write(&entry,"import {value} from './compute.rs';function main(){console.log(value());}");
  let dep=root.join("compute.rs");std::fs::write(&dep,"pub fn value()->i64{1}").unwrap();
  let mut watcher=crate::dev_watch::Watcher::default();let mut session=ProjectSession::default();
  let before=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};
  let first=compile(&mut session,&entry,&before).unwrap();assert!(first.rust.contains("pub fn value()->i64{1}"));
  std::fs::write(&dep,"pub fn value()->i64{2}").unwrap();
  let after=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};
  let next=compile(&mut session,&entry,&after).unwrap();assert!(next.rust.contains("pub fn value()->i64{2}"));assert_ne!(next.identity,first.identity);
  assert!(compile(&mut session,&entry,&before).unwrap().rust.contains("pub fn value()->i64{1}"));
 }

}

#[cfg(test)]mod web_target_tests{
 use super::*;
 #[test]fn web_only_project_uses_web_target_in_the_same_frozen_frontend(){
  let root=crate::TempDirectory::new().unwrap();let config=serde_json::json!({"schemaVersion":1,"name":"web-dev","build":{"entry":"main.tsrx","output":"target/app"},"ui":{"target":{"native":false,"web":true,"embedded":false},"theme":serde_json::from_str::<serde_json::Value>(tsrust_compiler::ui_project::theme::DEFAULT_JSON).unwrap(),"components":"src/components/ui","lock":"ui-components.lock.json"}});std::fs::write(root.0.join("tsrust.config.json"),config.to_string()).unwrap();let entry=root.0.join("main.tsrx");std::fs::write(&entry,"import {Text,run} from 'std:ui';function main(){unwrap(run(<Text>Web</Text>,'Web'));}").unwrap();let mut watcher=crate::dev_watch::Watcher::default();let snapshot=Snapshot{root:root.0.clone(),files:watcher.snapshot(&root.0).unwrap()};let mut session=ProjectSession::default();assert!(compile_target(&mut session,&entry,&snapshot,"web").is_ok());assert!(compile_target(&mut session,&entry,&snapshot,"native").err().unwrap().contains("disabled"));
 }
}
