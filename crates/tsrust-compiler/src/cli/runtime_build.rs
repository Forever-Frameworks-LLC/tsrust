//! Shared runtime build preparation; callers own subprocess supervision.
use std::{fs,path::{Path,PathBuf},process::Command};
use tsrust_compiler::{native_cache::NativeCache,native_runtime};
fn artifact_override(key:&str)->bool {
 key.starts_with("CARGO_PROFILE_") || (key.starts_with("CARGO_BUILD_") && key!="CARGO_BUILD_JOBS") || key.starts_with("CARGO_TARGET_") || matches!(key,"RUSTFLAGS"|"CARGO_ENCODED_RUSTFLAGS"|"RUSTC"|"RUSTC_WRAPPER"|"RUSTC_WORKSPACE_WRAPPER")
}
#[test]
fn runtime_build_preserves_scheduling_limits_but_rejects_artifact_overrides(){
 assert!(!artifact_override("CARGO_BUILD_JOBS"));
 for key in ["CARGO_BUILD_RUSTFLAGS","CARGO_BUILD_TARGET","CARGO_PROFILE_DEV_OPT_LEVEL","CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS","RUSTFLAGS","RUSTC_WRAPPER"] {assert!(artifact_override(key),"{key}");}
}
pub(crate) struct Prepared {
 pub application:PathBuf,
 pub library:Option<PathBuf>,
 pub command:Option<Command>,
 pub state:&'static str,
 cache:Option<NativeCache>,
 opportunistic:bool,
 dependencies:Option<PathBuf>,
 packed:Option<PathBuf>,
}
impl Prepared {
 pub fn new(source:&str,backend:&str,root:&Path,temporary:&Path,release:bool)->Result<Self,String> {
  Self::prepare(source,backend,root,temporary,release,false)
 }
 pub fn for_dev(source:&str,backend:&str,root:&Path,temporary:&Path,release:bool)->Result<Self,String> {
  Self::prepare(source,backend,root,temporary,release,true)
 }
 fn prepare(source:&str,backend:&str,root:&Path,temporary:&Path,release:bool,opportunistic:bool)->Result<Self,String> {
  let units=native_runtime::separate(source);let application=temporary.join("program.rs");fs::write(&application,units.application).map_err(|e|e.to_string())?;
  let Some(runtime)=units.runtime else{return Ok(Self{application,library:None,command:None,state:"none",cache:None,opportunistic,dependencies:None,packed:None});};
  if units.dependencies {return Self::with_dependencies(application,&runtime,backend,root,temporary,release,opportunistic);}
  let library=temporary.join("libtsrust_runtime.rlib");
  let cache=NativeCache::new(&root.join("runtime"),&[b"runtime-rlib-abi-1",backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on;crate=tsrust_runtime",runtime.as_bytes()]);
  let reused=if opportunistic{cache.try_copy_to(&library)}else{cache.copy_to(&library)};
  let command=if reused {None}else{
   let input=temporary.join("runtime.rs");fs::write(&input,runtime).map_err(|e|e.to_string())?;
   let mut command=crate::backend_command();command.args(["--edition=2024","--crate-type=rlib","--crate-name=tsrust_runtime","-C",if release{"opt-level=3"}else{"opt-level=0"},"-C","overflow-checks=on"]).arg(input).arg("-o").arg(&library);Some(command)
  };
  Ok(Self{application,library:Some(library),command,state:if reused{"hit"}else{"miss"},cache:Some(cache),opportunistic,dependencies:None,packed:None})
 }
 fn with_dependencies(application:PathBuf,runtime:&str,backend:&str,root:&Path,temporary:&Path,release:bool,opportunistic:bool)->Result<Self,String> {
  use crate::native_dependency_bundle as bundle;
  let cache=NativeCache::new(&root.join("runtime"),&[b"runtime-dependencies-abi-2",bundle::IDENTITY.as_bytes(),backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},runtime.as_bytes()]);
  let packed=temporary.join("runtime-artifacts");let restored=temporary.join("dependencies");
  let copied=if opportunistic{cache.try_copy_to(&packed)}else{cache.copy_to(&packed)};
  if copied && bundle::unpack(&packed,&restored).is_ok() {
   return Ok(Self{application,library:Some(restored.join("libtsrust_runtime.rlib")),command:None,state:"hit",cache:Some(cache),opportunistic,dependencies:Some(restored),packed:Some(packed)});
  }
  let package=temporary.join("native-support");bundle::materialize(&package).map_err(|e|e.to_string())?;
  crate::rust_binding_packages::overlay(&package,&crate::rust_binding_packages::packages(runtime)?)?;
  let manifest=package.join("Cargo.toml");let mut text=fs::read_to_string(&manifest).map_err(|e|e.to_string())?;
  text.push_str("\n[lib]\nname = \"tsrust_runtime\"\n[profile.dev]\noverflow-checks = true\n[profile.release]\noverflow-checks = true\n");
  fs::write(&manifest,text).map_err(|e|e.to_string())?;fs::write(package.join("src/lib.rs"),runtime).map_err(|e|e.to_string())?;
  let target=temporary.join("native-target");let profile=target.join(if release{"release"}else{"debug"});
  let mut command=crate::backend_cargo_command();command.current_dir(&package).args(["build","--offline","--locked","--lib","--manifest-path"]).arg(&manifest)
   .arg("--config").arg(package.join(".cargo/config.toml")).arg("--target-dir").arg(&target).env("CARGO_INCREMENTAL","0").env("CARGO_HOME",temporary.join("cargo-home"));
  // Compiler-owned support must not inherit user Cargo profiles or wrappers:
  // those would change artifacts without changing the verified cache identity.
  // The job limit only controls scheduling, so retain the caller's memory bound.
  for (name,_) in std::env::vars_os() {if artifact_override(&name.to_string_lossy()) {command.env_remove(name);}}
  if runtime.contains("pub mod tsrust_native_threads {") || runtime.contains("pub use rayon;") { command.args(["--features", "threads"]); }
  if runtime.contains("pub mod tsrust_native_futures {") {command.args(["--features","async"]);}
  if runtime.contains("pub mod tsrust_native_http {"){command.args(["--features","http"]);}
  if runtime.contains("pub mod tsrust_native_date {"){command.args(["--features","calendar"]);}
  if runtime.contains("pub mod tsrust_native_crypto {"){command.args(["--features","crypto"]);}
  if runtime.contains("pub mod tsrust_native_signals {"){command.args(["--features","signals"]);}
  if runtime.contains("pub mod tsrust_native_uuid {"){command.args(["--features","uuid"]);}
  if runtime.contains("pub mod tsrust_native_decimal {"){command.args(["--features","decimal"]);}
  if runtime.contains("pub mod tsrust_native_sql {"){command.args(["--features","sql"]);}
  if runtime.contains("pub mod tsrust_native_sqlite {"){command.args(["--features","sqlite"]);}
  if release {command.arg("--release");}
  Ok(Self{application,library:Some(profile.join("libtsrust_runtime.rlib")),command:Some(command),state:"miss",cache:Some(cache),opportunistic,dependencies:Some(profile.join("deps")),packed:Some(packed)})
 }
 pub fn publish(&self) {if let (Some(cache),Some(library))=(&self.cache,&self.library){let artifact=if let (Some(dependencies),Some(packed))=(&self.dependencies,&self.packed) {
   if crate::native_dependency_bundle::pack(library,dependencies,packed).is_err(){return;}packed
  }else{library};let _=if self.opportunistic{cache.try_publish(artifact)}else{cache.publish(artifact)};}}
 pub fn link(&self,command:&mut Command) {if let Some(dependencies)=&self.dependencies {command.arg("-L").arg(format!("dependency={}",dependencies.display()));}if let Some(library)=&self.library {command.arg("--extern").arg(format!("tsrust_runtime={}",library.display()));}}
}
