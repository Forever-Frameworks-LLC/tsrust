//! A bounded, persistent frontend worker. Cancelled responses are discarded by callers.
use std::{path::PathBuf,sync::mpsc::{self,Receiver,SyncSender},time::{Duration,Instant}};
use tsrust_compiler::{project::{FileSourceProvider,SourceProvider},project_cli::CheckedInput,project_session::{ProjectSession,SessionStats}};
pub(crate) struct Output {pub rust:String,pub features:Vec<&'static str>,pub identity:Vec<u8>,pub stats:SessionStats,pub check:Duration,pub emit:Duration}
pub(crate) struct Request {pub entry:PathBuf,pub sources:Snapshot,pub reply:mpsc::Sender<Result<Output,String>>}
/// Watcher-owned bytes remain fixed for the lifetime of a request. Files outside
/// the existing watch policy retain normal filesystem resolution and reading.
pub(crate) struct Snapshot {pub root:PathBuf,pub files:std::collections::BTreeMap<PathBuf,crate::dev_watch::Contents>}
impl SourceProvider for Snapshot {
 fn resolve(&self,importer:Option<&tsrust_compiler::project::SourceId>,path:&str)->Result<tsrust_compiler::project::SourceId,String> {FileSourceProvider.resolve(importer,path)}
 fn read(&self,id:&tsrust_compiler::project::SourceId)->Result<String,String> {
  let path=std::path::Path::new(&id.0);
  match self.files.get(path) {
   Some(contents)=>String::from_utf8(contents.bytes().to_vec()).map_err(|e|e.to_string()),
   None if path.extension().is_some_and(|ext|ext=="tsr") && path.strip_prefix(&self.root).is_ok_and(|relative|!relative.components().any(|part|matches!(part.as_os_str().to_str(),Some(".git"|"node_modules"|"target"))))=>Err(format!("{} is absent from this build snapshot",path.display())),
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
 for request in requests {let result=compile(&mut session,&request.entry,&request.sources);let _=request.reply.send(result);}
}
fn compile(session:&mut ProjectSession,entry:&std::path::Path,sources:&Snapshot)->Result<Output,String> {
 let started=Instant::now();
 let id=sources.resolve(None,entry.to_str().ok_or("Source path must be valid UTF-8.")?)?;
 let source=sources.read(&id)?;
 let analysis=tsrust_compiler::project_cli::check_input_cached_with_provider(entry,&source,session,sources).map_err(|error|error.render_with_provider(&entry.display().to_string(),&source,false,sources))?;
 let check=started.elapsed();let started=Instant::now();
 let(rust,features)=analysis.emit_rust_with_native_features();let emit=started.elapsed();
 let mut identity=Vec::new();
 match &analysis {
  CheckedInput::Project(project)=>for(id,module)in &project.graph.modules {for value in [id.0.as_bytes(),module.source.text.as_bytes()] {identity.extend_from_slice(&(value.len()as u64).to_le_bytes());identity.extend_from_slice(value);}},
  CheckedInput::Single(_)=>{let id=FileSourceProvider.resolve(None,entry.to_str().ok_or("Source path must be valid UTF-8.")?)?;identity.extend_from_slice(&(id.0.len()as u64).to_le_bytes());identity.extend_from_slice(id.0.as_bytes());identity.extend_from_slice(source.as_bytes());},
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
 fn frozen_sources_preserve_emission_and_diagnostic_lines_after_disk_edits() {
  let dir=crate::TempDirectory::new().unwrap();let root=std::fs::canonicalize(&dir.0).unwrap();let entry=root.join("main.tsr");let dep=root.join("dep.tsr");
  write(&entry,"import {value} from './dep.tsr';function main(){console.log(value());}");write(&dep,"export function value():int{return 1;}");
  std::fs::write(root.join("tsrust.config.json"),"{}").unwrap();
  let mut watcher=crate::dev_watch::Watcher::default();let sources=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};let mut session=ProjectSession::default();
  let later=root.join("later.tsr");write(&later,"export function later():int{return 1;}");
  assert!(sources.read(&sources.resolve(None,later.to_str().unwrap()).unwrap()).is_err());
  std::fs::remove_file(later).unwrap();
  let original=compile(&mut session,&entry,&sources).unwrap().rust;
  write(&entry,"function main(){console.log('changed');}");write(&dep,"export function value():int{return 2;}");
  assert_eq!(compile(&mut session,&entry,&sources).unwrap().rust,original);
  write(&entry,"import {value} from './dep.tsr';function main(){console.log(value());}");write(&dep,"export function value():int{return 'snapshot-error';}");
  let invalid=Snapshot{root:root.clone(),files:watcher.snapshot(&root).unwrap()};
  write(&dep,"export function value():int{return 99;}");
  let error=compile(&mut session,&entry,&invalid).err().unwrap();assert!(error.contains("snapshot-error"),"{error}");assert!(!error.contains("return 99"),"{error}");
 }
}
