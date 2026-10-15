//! Shared locked dependency inputs for native builds and package exports.
use std::{fs,path::Path};
use tsrust_compiler::project::rust::dependencies::{Package,MARKER};
pub(crate) fn packages(source:&str)->Result<Vec<Package>,String>{
 source.lines().filter_map(|line|line.strip_prefix(MARKER)).map(|json|serde_json::from_str(json).map_err(|error|format!("Invalid binding package: {error}"))).collect()
}
pub(crate) fn overlay(root:&Path,packages:&[Package])->Result<(),String>{
 let Some(package)=packages.first() else{return Ok(())};
 if packages.iter().any(|other|other.manifest!=package.manifest || other.lock!=package.lock){return Err("All binding modules in one program must share the same dependency versions and features.".into());}
 let lockfile=fs::OpenOptions::new().read(true).write(true).open(package.vendor.parent().unwrap().join(".lock")).map_err(|error|format!("Missing binding cache: {error}; run tsrust bindings resolve again."))?;
 lockfile.lock_shared().map_err(|error|error.to_string())?;
 fs::remove_dir_all(root.join("vendor")).map_err(|error|error.to_string())?;
 copy(&package.vendor,&root.join("vendor"))?;
 fs::write(root.join("Cargo.toml"),&package.manifest).and_then(|_|fs::write(root.join("Cargo.lock"),&package.lock)).map_err(|error|error.to_string())
}

pub(crate) fn copy(from:&Path,to:&Path)->Result<(),String>{
  fs::create_dir(to).map_err(|error|error.to_string())?;
  for entry in fs::read_dir(from).map_err(|error|format!("Missing resolved binding dependencies: {error}. Run tsrust bindings resolve again."))? {
   let entry=entry.map_err(|error|error.to_string())?;let kind=entry.file_type().map_err(|error|error.to_string())?;
   if kind.is_dir(){copy(&entry.path(),&to.join(entry.file_name()))?;}else if kind.is_file(){fs::copy(entry.path(),to.join(entry.file_name())).map_err(|error|error.to_string())?;}else{return Err("Binding vendor trees cannot contain symlinks.".into());}
  }Ok(())
 }
