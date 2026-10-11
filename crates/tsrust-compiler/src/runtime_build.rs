//! Shared runtime build preparation; callers own subprocess supervision.
use std::{fs,path::{Path,PathBuf},process::Command};
use tsrust_compiler::{native_cache::NativeCache,native_runtime};
pub(crate) struct Prepared {
 pub application:PathBuf,
 pub library:Option<PathBuf>,
 pub command:Option<Command>,
 pub state:&'static str,
 cache:Option<NativeCache>,
}
impl Prepared {
 pub fn new(source:&str,backend:&str,root:&Path,temporary:&Path,release:bool)->Result<Self,String> {
  let units=native_runtime::separate(source);let application=temporary.join("program.rs");fs::write(&application,units.application).map_err(|e|e.to_string())?;
  let Some(runtime)=units.runtime else{return Ok(Self{application,library:None,command:None,state:"none",cache:None});};
  let library=temporary.join("libtsrust_runtime.rlib");
  let cache=NativeCache::new(&root.join("runtime"),&[b"runtime-rlib-abi-1",backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on;crate=tsrust_runtime",runtime.as_bytes()]);
  let reused=cache.copy_to(&library);
  let command=if reused {None}else{
   let input=temporary.join("runtime.rs");fs::write(&input,runtime).map_err(|e|e.to_string())?;
   let mut command=crate::backend_command();command.args(["--edition=2024","--crate-type=rlib","--crate-name=tsrust_runtime","-C",if release{"opt-level=3"}else{"opt-level=0"},"-C","overflow-checks=on"]).arg(input).arg("-o").arg(&library);Some(command)
  };
  Ok(Self{application,library:Some(library),command,state:if reused{"hit"}else{"miss"},cache:Some(cache)})
 }
 pub fn publish(&self) {if let (Some(cache),Some(library))=(&self.cache,&self.library){let _=cache.publish(library);}}
 pub fn link(&self,command:&mut Command) {if let Some(library)=&self.library {command.arg("--extern").arg(format!("tsrust_runtime={}",library.display()));}}
}
