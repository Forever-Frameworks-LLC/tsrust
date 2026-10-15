//! Resolve dependencies explicitly; ordinary checks and builds remain offline.
use std::{fs,process::Stdio};
use crate::rust_binding_packages::copy;
use tsrust_compiler::project::rust::dependencies::Bindings;
pub(crate) fn resolve(mut args:impl Iterator<Item=String>)->Result<u8,String>{
 if args.next().as_deref()!=Some("resolve"){return Err("Usage: tsrust bindings resolve file.bindings.json".into());}
 let path=args.next().ok_or("Missing .bindings.json path.")?;
 let offline=match args.next().as_deref(){None=>false,Some("--offline")=>true,_=>return Err("Expected optional --offline.".into())};
 if args.next().is_some() || !path.ends_with(".bindings.json"){return Err("Expected one .bindings.json path.".into());}
 let path=fs::canonicalize(path).map_err(|error|error.to_string())?;
 let binding=Bindings::parse(&fs::read_to_string(&path).map_err(|error|error.to_string())?)?;
 let temporary=crate::TempDirectory::new().map_err(|error|error.to_string())?;let root=&temporary.0;
 fs::create_dir(root.join("src")).map_err(|error|error.to_string())?;
 fs::write(root.join("src/lib.rs"),"").map_err(|error|error.to_string())?;
 fs::write(root.join("Cargo.toml"),binding.manifest()).map_err(|error|error.to_string())?;
 if let Some(lock)=&binding.lockfile{fs::write(root.join("Cargo.lock"),lock).map_err(|error|error.to_string())?;}
 let mut command=crate::backend_cargo_command();command.current_dir(root).args(if binding.lockfile.is_some(){vec!["metadata","--format-version","1","--locked","--manifest-path"]}else{vec!["generate-lockfile","--manifest-path"]}).arg(root.join("Cargo.toml")).stdout(Stdio::null());
 if offline{command.arg("--offline");}
 if !command.status().map_err(|error|error.to_string())?.success(){return Err("Cargo dependency resolution failed; existing lock retained.".into());}
 let lock=fs::read_to_string(root.join("Cargo.lock")).map_err(|error|error.to_string())?;
 let package=binding.package(&path,lock.clone());
 let vendor=root.join("vendor");
 let mut command=crate::backend_cargo_command();command.current_dir(root).args(["vendor","--locked","--versioned-dirs","--manifest-path"]).arg(root.join("Cargo.toml")).arg(&vendor).stdout(Stdio::null());
 if offline{command.arg("--offline");}
 if !command.status().map_err(|error|error.to_string())?.success(){return Err("Cargo vendoring failed; existing lock retained.".into());}
 let parent=package.vendor.parent().unwrap();fs::create_dir_all(parent).map_err(|error|error.to_string())?;
 fs::write(parent.parent().unwrap().join(".gitignore"),"*\n").map_err(|error|error.to_string())?;
 let lockfile=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(parent.join(".lock")).map_err(|error|error.to_string())?;
 lockfile.lock().map_err(|error|error.to_string())?;
 let staged_vendor=parent.join(format!("vendor-pending-{}",std::process::id()));
 if let Err(error)=copy(&vendor,&staged_vendor){let _=fs::remove_dir_all(&staged_vendor);return Err(error);}
 if package.vendor.exists(){fs::remove_dir_all(&package.vendor).map_err(|error|error.to_string())?;}
 fs::rename(&staged_vendor,&package.vendor).map_err(|error|error.to_string())?;
 let destination=Bindings::lock_path(&path);let staged=destination.with_extension("lock.tmp");
 if binding.lockfile.is_none(){fs::write(&staged,lock).and_then(|_|fs::rename(&staged,&destination)).map_err(|error|error.to_string())?;}
 println!("Resolved {}",destination.display());Ok(0)
}
