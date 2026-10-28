//! A managed installation selects one immutable pair; running files are never replaced.
use std::{fs,io::Write,path::{Path,PathBuf},process::Command};
use serde::{Deserialize,Serialize};
use super::toolchain_release::{self as release,Manifest,Release};
const MARKER:&str="tsrust-managed-launcher-1\n";
const MARKER_FILE:&str=".tsrust-launcher";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Selected{pub release:Release,pub manifest_sha256:String}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct State{schema_version:u32,pub current:Selected,pub previous:Option<Selected>}
fn regular(path:&Path,directory:bool)->Result<(),String>{let m=fs::symlink_metadata(path).map_err(|e|e.to_string())?;if m.file_type().is_symlink()||if directory{!m.is_dir()}else{!m.is_file()}{return Err(format!("Managed installation path has an unexpected type: {}",path.display()));}Ok(())}
fn read(path:&Path,limit:u64)->Result<Vec<u8>,String>{regular(path,false)?;if fs::metadata(path).map_err(|e|e.to_string())?.len()>limit{return Err("Managed metadata exceeds its size limit".into());}fs::read(path).map_err(|e|e.to_string())}
fn selected(root:&Path,value:&Selected)->Result<PathBuf,String>{
 let r=&value.release;release::version(&r.version)?;
 if !release::hash(&r.sha256)||!release::hash(&value.manifest_sha256)||r.target!=release::host()?{return Err("Invalid managed toolchain identity or host".into());}
 regular(&root.join("versions"),true)?;let directory=root.join("versions").join(&r.sha256);regular(&directory,true)?;Ok(directory)
}
fn owner(root:&Path)->Result<(),String>{
 regular(root,true)?;
 if read(&root.join(MARKER_FILE),128)?!=MARKER.as_bytes(){return Err("This installation is not owned by the Tsrust installer; use its package manager or reinstall into a new directory".into());}
 Ok(())
}
pub(super) fn state(root:&Path)->Result<State,String>{
 owner(root)?;
 let state:State=serde_json::from_slice(&read(&root.join("installation.json"),65536)?).map_err(|_|"Invalid managed installation record; preserve it and reinstall into a new directory")?;
 if state.schema_version!=1{return Err("Unsupported installation protocol; reinstall with a current official toolchain".into());}
 selected(root,&state.current)?;if let Some(previous)=&state.previous{selected(root,previous)?;}Ok(state)
}
pub(super) fn manifest(root:&Path,value:&Selected)->Result<Manifest,String>{
 let directory=selected(root,value)?;let bytes=read(&directory.join("manifest.json"),2*1024*1024)?;
 if release::digest(&bytes)!=value.manifest_sha256{return Err("Installed toolchain manifest was modified; preserve it and reinstall into a new directory".into());}
 let manifest:Manifest=serde_json::from_slice(&bytes).map_err(|_|"Invalid installed manifest")?;
 if manifest.schema_version!=2||manifest.version!=value.release.version||manifest.target!=value.release.target||manifest.distribution!="release"||manifest.source_tree_dirty{return Err("Installed manifest differs from the selected release".into());}Ok(manifest)
}
pub(super) fn current_root()->Result<Option<PathBuf>,String>{
 let executable=std::env::current_exe().map_err(|e|e.to_string())?;
 let Some(bin)=executable.parent().filter(|p|p.file_name().is_some_and(|n|n=="bin")) else{return Ok(None)};
 let Some(parent)=bin.parent()else{return Ok(None)};
 if parent.join(MARKER_FILE).exists(){return Ok(Some(parent.into()));}
 if parent.file_name().and_then(|n|n.to_str()).is_some_and(release::hash)
  &&let Some(versions)=parent.parent().filter(|p|p.file_name().is_some_and(|n|n=="versions"))
  &&let Some(root)=versions.parent().filter(|p|p.join(MARKER_FILE).exists()){return Ok(Some(root.into()));}
 Ok(None)
}
pub(super) fn dispatch()->Result<Option<u8>,String>{
 let executable=std::env::current_exe().map_err(|e|e.to_string())?;
 let Some(bin)=executable.parent().filter(|p|p.file_name().is_some_and(|n|n=="bin"))else{return Ok(None)};
 let Some(root)=bin.parent().filter(|p|p.join(MARKER_FILE).exists())else{return Ok(None)};
 owner(root)?;let lock_path=root.join(".installation.lock");regular(&lock_path,false)?;
 let launch_lock=fs::File::open(lock_path).map_err(|e|e.to_string())?;launch_lock.lock_shared().map_err(|e|e.to_string())?;
 let state=state(root)?;let manifest=manifest(root,&state.current)?;
 let name=executable.file_name().and_then(|n|n.to_str()).ok_or("Invalid launcher name")?;
 if name!=format!("tsrust{}",std::env::consts::EXE_SUFFIX)&&name!=format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX){return Err("Unknown managed launcher name".into());}
 let relative=format!("bin/{name}");let record=manifest.files.get(&relative).ok_or("Selected toolchain lacks the requested executable")?;
 let directory=selected(root,&state.current)?;regular(&directory.join("bin"),true)?;let binary=directory.join(&relative);
 let bytes=read(&binary,256*1024*1024)?;
 if bytes.len() as u64!=record.bytes||release::digest(&bytes)!=record.sha256{return Err("Selected executable checksum differs; refusing to launch it".into());}
 let mut command=Command::new(binary);command.args(std::env::args_os().skip(1));
 #[cfg(unix)]{use std::os::unix::process::CommandExt;Err(format!("Cannot launch selected toolchain: {}",command.exec()))}
 #[cfg(windows)]{let mut process=super::dev_windows::Process::spawn(&mut command)?;drop(launch_lock);let status=process.child.wait().map_err(|e|e.to_string())?;Ok(Some(status.code().and_then(|n|u8::try_from(n).ok()).unwrap_or(1)))}
 #[cfg(not(any(unix,windows)))]{Err("Managed launchers are unsupported on this platform".into())}
}
pub(super) fn lock(root:&Path)->Result<fs::File,String>{
 owner(root)?;let path=root.join(".installation.lock");
 if path.exists(){regular(&path,false)?;}
 let file=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path).map_err(|e|e.to_string())?;
 file.try_lock().map_err(|_|"Another toolchain operation is running; wait for it to finish")?;Ok(file)
}
fn publish(root:&Path,state:&State)->Result<(),String>{
 let pending=root.join(".installation.pending");
 if pending.exists(){regular(&pending,false)?;fs::remove_file(&pending).map_err(|e|e.to_string())?;}
 let result=(||{
  let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&pending).map_err(|e|e.to_string())?;
  file.write_all(&serde_json::to_vec_pretty(state).map_err(|e|e.to_string())?).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;
  drop(file);fs::rename(&pending,root.join("installation.json")).map_err(|e|format!("Installation selection was not changed: {e}"))?;
  #[cfg(unix)]if let Err(error)=fs::File::open(root).and_then(|file|file.sync_all()){eprintln!("Toolchain selection committed, but directory sync failed: {error}");}
  Ok(())
 })();if result.is_err(){let _=fs::remove_file(pending);}result
}
pub(super) fn stage(root:&Path,release:&Release,bytes:&[u8])->Result<Selected,String>{
 let versions=root.join("versions");regular(&versions,true)?;
 let path=versions.join(&release.sha256);
 clean_stages(&versions);
 let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos();
 let container=versions.join(format!(".stage-{}-{stamp}",std::process::id()));
 fs::create_dir(&container).map_err(|e|e.to_string())?;
 fs::write(container.join(".managed-stage"),MARKER).map_err(|e|e.to_string())?;
 let staging=container.join("package");
 let result=(||{
  let manifest=release::unpack(&staging,release,bytes)?;
  consumer_versions(&staging,&manifest,&container)?;
  let manifest_sha256=release::digest(&read(&staging.join("manifest.json"),2*1024*1024)?);
  fs::write(staging.join(".managed-version"),MARKER).map_err(|e|e.to_string())?;
  if path.exists(){
   regular(&path,true)?;
   if read(&path.join(".managed-version"),128)?!=MARKER.as_bytes()||release::digest(&read(&path.join("manifest.json"),2*1024*1024)?)!=manifest_sha256{return Err("Existing release directory differs from the verified archive; preserved it".into());}
   release::verify(&path,release)?;
  }else{fs::rename(&staging,&path).map_err(|e|e.to_string())?;}
  #[cfg(unix)]fs::File::open(&versions).and_then(|file|file.sync_all()).map_err(|e|e.to_string())?;
  Ok(Selected{release:release.clone(),manifest_sha256})
 })();let _=fs::remove_dir_all(&container);result
}
fn clean_stages(versions:&Path){
 let Ok(entries)=fs::read_dir(versions)else{return};
 for entry in entries.flatten(){let path=entry.path();
  if entry.file_name().to_str().is_some_and(|name|name.starts_with(".stage-"))&&regular(&path,true).is_ok()
   &&read(&path.join(".managed-stage"),128).is_ok_and(|bytes|bytes==MARKER.as_bytes()){let _=fs::remove_dir_all(path);}
 }
}
pub(super) fn install(root:&Path,release:&Release,bytes:&[u8],launcher:&Path)->Result<State,String>{
 fs::create_dir(root).map_err(|e|format!("Installation requires a new directory; existing files are preserved: {e}"))?;
 let result=(||{
  #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;fs::set_permissions(root,fs::Permissions::from_mode(0o700)).map_err(|e|e.to_string())?;}
  fs::create_dir(root.join("versions")).map_err(|e|e.to_string())?;fs::create_dir(root.join("bin")).map_err(|e|e.to_string())?;
  fs::write(root.join(MARKER_FILE),MARKER).map_err(|e|e.to_string())?;
  fs::OpenOptions::new().write(true).create_new(true).open(root.join(".installation.lock")).map_err(|e|e.to_string())?;
  let current=stage(root,release,bytes)?;
  let compiler=root.join("bin").join(format!("tsrust{}",std::env::consts::EXE_SUFFIX));
  let server=root.join("bin").join(format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX));
  fs::copy(launcher,&compiler).map_err(|e|e.to_string())?;fs::OpenOptions::new().write(true).open(&compiler).and_then(|f|f.sync_all()).map_err(|e|e.to_string())?;
  if fs::hard_link(&compiler,&server).is_err(){fs::copy(&compiler,&server).map_err(|e|e.to_string())?;}
  let state=State{schema_version:1,current,previous:None};publish(root,&state)?;Ok(state)
 })();if result.is_err(){let _=fs::remove_dir_all(root);}result
}
fn consumer_versions(root:&Path,manifest:&Manifest,scratch:&Path)->Result<(),String>{
 use std::{process::Stdio,time::{Duration,Instant}};
 for (name,expected) in [("tsrust",&manifest.version),("tsrust-lsp",&manifest.lsp_version)]{
  let output=scratch.join(format!("{name}-version.txt"));let file=fs::OpenOptions::new().write(true).create_new(true).open(&output).map_err(|e|e.to_string())?;
  let mut child=Command::new(root.join("bin").join(format!("{name}{}",std::env::consts::EXE_SUFFIX))).arg("--version")
   .stdin(Stdio::null()).stdout(file).stderr(Stdio::null()).spawn().map_err(|e|format!("Staged {name} could not execute: {e}"))?;
  let deadline=Instant::now()+Duration::from_secs(10);
  let status=loop{match child.try_wait(){
   Ok(Some(status))=>break status,
   Ok(None) if Instant::now()<deadline=>std::thread::sleep(Duration::from_millis(10)),
   result=>{let _=child.kill();let _=child.wait();return Err(format!("Staged {name} version probe failed or timed out: {result:?}"));}
  }};
  if !status.success()||String::from_utf8(read(&output,4096)?).map_err(|_|"Invalid executable version output")?.trim()!=format!("{name} {expected}"){return Err(format!("Staged {name} does not report its manifest version"));}
 }Ok(())
}
pub(super) fn update(root:&Path,release:&Release,bytes:&[u8])->Result<State,String>{
 let previous=state(root)?;
 if release::version(&release.version)?<=release::version(&previous.current.release.version)?{return Err("Updates cannot downgrade or replace the installed version; use explicit rollback for the retained pair".into());}
 manifest(root,&previous.current)?;
 release::verify(&selected(root,&previous.current)?,&previous.current.release)?;
 let current=stage(root,release,bytes)?;let next=State{schema_version:1,current,previous:Some(previous.current)};publish(root,&next)?;cleanup(root,&next);Ok(next)
}
pub(super) fn rollback(root:&Path)->Result<State,String>{
 let previous=state(root)?;let current=previous.previous.ok_or("No previous verified toolchain is retained")?;
 manifest(root,&current)?;release::verify(&selected(root,&current)?,&current.release)?;
 let next=State{schema_version:1,current,previous:Some(previous.current)};publish(root,&next)?;Ok(next)
}
fn cleanup(root:&Path,state:&State){
 let Ok(entries)=fs::read_dir(root.join("versions"))else{return};
 for entry in entries.flatten(){
  let path=entry.path();let Some(name)=path.file_name().and_then(|n|n.to_str())else{continue};
  if !release::hash(name)||name==state.current.release.sha256||state.previous.as_ref().is_some_and(|p|name==p.release.sha256){continue;}
  if regular(&path,true).is_ok()&&read(&path.join(".managed-version"),128).is_ok_and(|bytes|bytes==MARKER.as_bytes())
   &&let Err(error)=fs::remove_dir_all(&path){eprintln!("Retained an older toolchain directory (it may still be in use): {}: {error}",path.display());}
 }
}
