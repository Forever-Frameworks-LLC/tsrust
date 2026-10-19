//! Explicit disk housekeeping; valid artifacts have no hidden size or entry quota.
use std::{fs,path::Path};
pub(super) fn run(mut args:impl Iterator<Item=String>)->Result<u8,String> {
 let command=args.next().ok_or("Usage: tsrust cache <info|clean>")?;
 if args.next().is_some() || !matches!(command.as_str(),"info"|"clean") {return Err("Usage: tsrust cache <info|clean>".into());}
 let root=match std::env::var_os("TSRUST_CACHE_DIR"){Some(root)=>std::path::PathBuf::from(root),None=>super::project_config::default_root()?.join("target/tsrust-native")};
 let mut count=0;let mut bytes=0;
 for path in [root.join("v2"),root.join("runtime/v2")] {
  if !path.exists(){continue;}
  if fs::symlink_metadata(&path).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Cache namespace must not be a symlink".into());}
  let lock=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path.join(".cache.lock")).map_err(|e|e.to_string())?;lock.lock().map_err(|e|e.to_string())?;
  for entry in fs::read_dir(&path).map_err(|e|e.to_string())? {
   let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name();let name=name.to_string_lossy();
   if !entry.file_type().map_err(|e|e.to_string())?.is_dir() || !(name.len()==16 && name.bytes().all(|v|v.is_ascii_hexdigit()) || name.starts_with(".staging-")){continue;}
   count+=1;bytes+=size(&entry.path())?;
   if command=="clean" {fs::remove_dir_all(entry.path()).map_err(|e|e.to_string())?;}
  }
 }
 println!("{}: {count} artifacts, {bytes} bytes{}",root.display(),if command=="clean" {" removed; running private copies remain valid"}else{"; no automatic quota"});Ok(0)
}
fn size(path:&Path)->Result<u64,String> {
 let mut bytes=0;for entry in fs::read_dir(path).map_err(|e|e.to_string())? {let entry=entry.map_err(|e|e.to_string())?;if entry.file_type().map_err(|e|e.to_string())?.is_file(){bytes+=entry.metadata().map_err(|e|e.to_string())?.len();}}Ok(bytes)
}
