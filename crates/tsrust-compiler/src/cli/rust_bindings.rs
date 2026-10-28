//! Explicit shared-cache resolution; optional portable preparation never runs in a build.
use std::{fs,process::Stdio};
use tsrust_compiler::project::rust::dependencies::Bindings;
pub(crate) fn resolve(mut args:impl Iterator<Item=String>)->Result<u8,String>{
 if args.next().as_deref()!=Some("resolve"){return Err("Usage: tsrust bindings resolve file.bindings.json [--offline] [--portable]".into());}
 let path=args.next().ok_or("Missing .bindings.json path.")?;
 let (mut offline,mut portable)=(false,false);
 for argument in args {match argument.as_str(){"--offline" if !offline=>offline=true,"--portable" if !portable=>portable=true,_=>return Err("Expected optional --offline and --portable once each.".into())}}
 if !path.ends_with(".bindings.json"){return Err("Expected one .bindings.json path.".into());}
 let path=fs::canonicalize(path).map_err(|error|error.to_string())?;
 let _project_lock=crate::package_cli::project_lock(&path)?;
 let text=fs::read_to_string(&path).map_err(|error|error.to_string())?;
 let binding=Bindings::parse(&text)?;
 let destination=Bindings::lock_path(&path);
 let existing=crate::package_validation::read_optional(&destination)?;
 let pinned=binding.lockfile.as_deref().or(existing.as_deref());
 let temporary=crate::TempDirectory::new().map_err(|error|error.to_string())?;let root=&temporary.0;
 fs::create_dir(root.join("src")).map_err(|error|error.to_string())?;
 fs::write(root.join("src/lib.rs"),"").map_err(|error|error.to_string())?;
 fs::write(root.join("Cargo.toml"),binding.manifest()).map_err(|error|error.to_string())?;
 if let Some(lock)=pinned{fs::write(root.join("Cargo.lock"),lock).map_err(|error|error.to_string())?;}
 if pinned.is_none(){
  let mut command=crate::backend_cargo_command();command.current_dir(root).arg("generate-lockfile").stdout(Stdio::null());
  if offline{command.arg("--offline");}
  if !command.status().map_err(|error|error.to_string())?.success(){return Err("Cargo dependency resolution failed; existing lock retained.".into());}
 }
 let mut command=crate::backend_cargo_command();command.current_dir(root).args(["fetch","--locked"]).stdout(Stdio::null());
 if offline{command.arg("--offline");}
 if !command.status().map_err(|error|error.to_string())?.success(){return Err("Cargo source preparation failed; existing lock retained.".into());}
 let lock=fs::read_to_string(root.join("Cargo.lock")).map_err(|error|error.to_string())?;
 if portable{crate::native_dependency_bundle::prepare_binding(crate::backend_cargo_command(),offline,&binding.package(lock.clone())).map_err(|error|error.to_string())?;}
 if fs::read_to_string(&path).map_err(|error|error.to_string())?!=text || crate::package_validation::read_optional(&destination)?!=existing{return Err("Binding configuration or lock changed during resolution; nothing applied.".into());}
 if binding.lockfile.is_none() && existing.is_none(){
  crate::package_cli::publish(&destination,&lock)?;
 }
 println!("Resolved shared Cargo sources for {}{}",path.display(),if portable{" and prepared its portable source pack"}else{""});Ok(0)
}
