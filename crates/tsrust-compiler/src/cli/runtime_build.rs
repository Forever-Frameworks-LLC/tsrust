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
 shared_sources:bool,
 source_snapshot:Option<(PathBuf,String)>,
 cache:Option<NativeCache>,
 opportunistic:bool,
 dependencies:Option<PathBuf>,
 packed:Option<PathBuf>,
 _workspace:Option<crate::native_workspace::Workspace>,
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
  let Some(runtime)=units.runtime else{return Ok(Self{application,library:None,command:None,state:"none",shared_sources:false,source_snapshot:None,cache:None,opportunistic,dependencies:None,packed:None,_workspace:None});};
  if units.dependencies {return Self::with_dependencies(application,&runtime,backend,root,temporary,release,opportunistic);}
  let library=temporary.join("libtsrust_runtime.rlib");
  let cache=NativeCache::runtime(&root.join("runtime"),&[b"runtime-rlib-abi-1",backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on;crate=tsrust_runtime",runtime.as_bytes()]);
  let reused=if opportunistic{cache.try_copy_to(&library)}else{cache.copy_to(&library)};
  let command=if reused {None}else{
   let input=temporary.join("runtime.rs");fs::write(&input,runtime).map_err(|e|e.to_string())?;
   let mut command=crate::backend_command();command.args(["--edition=2024","--crate-type=rlib","--crate-name=tsrust_runtime","-C",if release{"opt-level=3"}else{"opt-level=0"},"-C","overflow-checks=on"]).arg(input).arg("-o").arg(&library);Some(command)
  };
  Ok(Self{application,library:Some(library),command,state:if reused{"hit"}else{"miss"},shared_sources:false,source_snapshot:None,cache:Some(cache),opportunistic,dependencies:None,packed:None,_workspace:None})
 }
 fn with_dependencies(application:PathBuf,runtime:&str,backend:&str,root:&Path,temporary:&Path,release:bool,opportunistic:bool)->Result<Self,String> {
  use crate::native_dependency_bundle as bundle;
  let absolute_root=crate::absolute(root)?;let root=absolute_root.as_path();
  let packages=crate::rust_binding_packages::packages(runtime)?;
  let shared=crate::backend_sources::shared(runtime)?;
  let dependency_identity=if shared{crate::backend_sources::IDENTITY}else{bundle::IDENTITY};
  let shared_inputs=if shared{Some(crate::backend_sources::identity(root)?)}else{None};
  let mut cache_parts:Vec<&[u8]>=vec![b"runtime-dependencies-abi-4-import-libraries",dependency_identity.as_bytes(),backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},runtime.as_bytes()];
  if let Some(inputs)=&shared_inputs{cache_parts.push(inputs.as_bytes());}
  let cache=NativeCache::runtime(&root.join("runtime"),&cache_parts);
  let packed=temporary.join("runtime-artifacts");let restored=temporary.join("dependencies");
  let copied=if opportunistic{cache.try_copy_to(&packed)}else{cache.copy_to(&packed)};
  if copied && bundle::unpack(&packed,&restored).is_ok() {
   return Ok(Self{application,library:Some(restored.join("libtsrust_runtime.rlib")),command:None,state:"hit",shared_sources:shared,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:Some(restored),packed:Some(packed),_workspace:None});
  }
  let native_ui=runtime.contains("pub mod tsrust_native_ui {") && runtime.contains("use vello::");
  let identity=serde_json::to_vec(&("native-build-workspace-1",dependency_identity,backend,std::env::consts::OS,std::env::consts::ARCH,release,profiles(native_ui))).map_err(|e|e.to_string())?;
  // Resolved binding overlays retain their own source/cache protocol. Ordinary
  // backend dependencies use shared Cargo sources and one stable target across
  // runtime revisions. The retained UI and binding export paths still use vendors.
  let workspace=if packages.is_empty(){crate::native_workspace::Workspace::acquire(root,&identity,opportunistic)?}else{None};
  let build_root=workspace.as_ref().map(|workspace|workspace.root.as_path()).unwrap_or(temporary);
  let package=build_root.join("native-support");
  if workspace.is_none() || !package.exists(){
   if shared{crate::backend_sources::materialize(&package)?;}else{bundle::materialize(&package).map_err(|e|e.to_string())?;}
  }
  if shared{crate::backend_sources::validate(&package)?;}
  crate::rust_binding_packages::overlay(&package,&packages)?;
  let manifest=package.join("Cargo.toml");
  let mut text=if shared{crate::backend_sources::MANIFEST.to_owned()}else if packages.is_empty(){include_str!("../../native-dependencies/Cargo.toml").to_owned()}else{fs::read_to_string(&manifest).map_err(|e|e.to_string())?};
  text.push_str("\n[lib]\nname = \"tsrust_runtime\"\n");
  text.push_str(profiles(native_ui));
  crate::native_workspace::write_changed(&manifest,text)?;crate::native_workspace::write_changed(&package.join("src/lib.rs"),runtime)?;
  let target=build_root.join("native-target");let profile=target.join(if release{"release"}else{"debug"});
  let mut command=crate::backend_cargo_command();command.current_dir(&package).args(["build","--offline","--locked","--lib","--manifest-path"]).arg(&manifest)
   .arg("--config").arg(package.join(".cargo/config.toml")).arg("--target-dir").arg(&target).env("CARGO_INCREMENTAL","0");
  if shared{command.current_dir(root).env("CARGO_HOME",crate::backend_sources::cargo_home()?);}
  else{command.env("CARGO_HOME",build_root.join("cargo-home"));}
  // Compiler-owned support must not inherit user Cargo profiles or wrappers:
  // those would change artifacts without changing the verified cache identity.
  // The job limit only controls scheduling, so retain the caller's memory bound.
  for (name,_) in std::env::vars_os() {if artifact_override(&name.to_string_lossy()) {command.env_remove(name);}}
  if runtime.contains("pub mod tsrust_native_threads {") || runtime.contains("pub use rayon;") { command.args(["--features", "threads"]); }
  if runtime.contains("pub mod tsrust_native_futures {") {command.args(["--features","async"]);}
  if runtime.contains("pub mod tsrust_native_http {"){command.args(["--features","http"]);}
  if runtime.contains("pub mod tsrust_native_ui {"){command.args(["--features","ui"]);}
  if runtime.contains("pub mod tsrust_native_ui {") && runtime.contains("use vello::"){command.args(["--features", "ui-native"]);
        }
        if runtime.contains("pub mod tsrust_native_date {") {
            command.args(["--features", "calendar"]);}
  if runtime.contains("pub mod tsrust_native_crypto {"){command.args(["--features","crypto"]);}
  if runtime.contains("pub mod tsrust_native_signals {"){command.args(["--features","signals"]);}
  if runtime.contains("pub mod tsrust_native_uuid {"){command.args(["--features","uuid"]);}
  if runtime.contains("pub mod tsrust_native_decimal {"){command.args(["--features","decimal"]);}
  if runtime.contains("pub mod tsrust_native_sql {"){command.args(["--features","sql"]);}
  if runtime.contains("pub mod tsrust_native_sqlite {"){command.args(["--features","sqlite"]);}
  if release {command.arg("--release");}
  Ok(Self{application,library:Some(profile.join("libtsrust_runtime.rlib")),command:Some(command),state:"miss",shared_sources:shared,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:Some(profile.join("deps")),packed:Some(packed),_workspace:workspace})
 }
 pub fn publish(&self) {
  if let Some((root,expected))=&self.source_snapshot
   && !crate::backend_sources::unchanged(Some(expected.as_str()),root) {return;}
  if let (Some(cache),Some(library))=(&self.cache,&self.library){let artifact=if let (Some(dependencies),Some(packed))=(&self.dependencies,&self.packed) {
   if crate::native_dependency_bundle::pack(library,dependencies,packed).is_err(){return;}packed
  }else{library};let _=if self.opportunistic{cache.try_publish(artifact)}else{cache.publish(artifact)};}}
 pub fn compilation_error(&self,detail:&str)->String {
  let hint=if self.shared_sources && (detail.contains("offline") || detail.contains("no matching package")) {"\nPrepare missing sources explicitly with `tsrust setup --dependencies --yes`, or use `--offline` with a populated Cargo home. Native builds never fetch sources."}else{""};
  format!("Runtime compilation failed:\n{detail}{hint}")
 }
 pub fn link(&self,command:&mut Command)->Result<(),String> {if let Some(dependencies)=&self.dependencies {
  let imports=crate::native_dependency_bundle::restore_import_libraries(dependencies).map_err(|e|e.to_string())?;
  command.arg("-L").arg(format!("dependency={}",dependencies.display()));
  if imports>0{command.arg("-L").arg(format!("native={}",dependencies.display()));}
 }if let Some(library)=&self.library {command.arg("--extern").arg(format!("tsrust_runtime={}",library.display()));}Ok(())}
}

/// Match the native UI support profile in application code as well. Large retained
/// component compositions otherwise keep every debug temporary on the Windows
/// main thread stack, even while the separately built renderer is optimized.
/// Overflow checks remain enabled independently at both compiler call sites.
pub(crate) fn application_optimization(release:bool,native_ui:bool)->&'static str {
 if release {"opt-level=3"} else if native_ui {"opt-level=1"} else {"opt-level=0"}
}

#[cfg(test)] mod application_profile_tests {
 #[test] fn native_ui_development_matches_support_without_changing_backend_or_release(){
  use super::application_optimization;
  assert_eq!(application_optimization(false,true),"opt-level=1");
  assert_eq!(application_optimization(false,false),"opt-level=0");
  for ui in [false,true]{assert_eq!(application_optimization(true,ui),"opt-level=3");}
 }
}

/// UI rendering must remain responsive in development; omit bulky dependency debug symbols.
pub(crate) fn profiles(native_ui: bool) -> &'static str {
 if native_ui {"\n[profile.dev]\noverflow-checks = true\nincremental = false\ndebug = false\nopt-level = 1\n[profile.dev.package.\"*\"]\nopt-level = 2\ndebug = false\n[profile.release]\noverflow-checks = true\nincremental = false\n"}
 else {"\n[profile.dev]\noverflow-checks = true\nincremental = false\n[profile.release]\noverflow-checks = true\nincremental = false\n"}
}

#[cfg(test)] mod dependency_reuse_tests {
 use super::*;
 #[test] fn changed_runtime_reuses_dependency_builds_and_links_current_values() {
  struct Cleanup(PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
  let dir=Cleanup(std::env::temp_dir().join(format!("tsrust-runtime-reuse-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));fs::create_dir(&dir.0).unwrap();
  let cache=dir.0.join("cache");let mut previous=None;
  for (index,value) in [42,43].into_iter().enumerate(){
   let temporary=dir.0.join(format!("build-{index}"));fs::create_dir(&temporary).unwrap();let application=temporary.join("program.rs");
   fs::write(&application,format!("fn main(){{assert_eq!(tsrust_runtime::answer(),{value});}}")).unwrap();
   let runtime=format!("pub fn answer()->i64{{let value:serde_json::Value=serde_json::from_str(\"{value}\").unwrap();value.as_i64().unwrap()}}");
   let mut prepared=Prepared::with_dependencies(application,&runtime,"pinned-backend-test",&cache,&temporary,false,false).unwrap();assert_eq!(prepared.state,"miss");
   let mut command=prepared.command.take().unwrap();let package=command.get_current_dir().unwrap().to_owned();
   let output=command.output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
   let dependencies=prepared.dependencies.as_ref().unwrap();
   let json=fs::read_dir(dependencies).unwrap().map(Result::unwrap).find(|entry|entry.file_name().to_string_lossy().starts_with("libserde_json-") && entry.path().extension().is_some_and(|extension|extension=="rlib")).unwrap().path();
   let stamp=fs::metadata(&json).unwrap().modified().unwrap();
   if let Some((previous_package,previous_json,previous_stamp))=&previous {assert_eq!(&package,previous_package);assert_eq!(&json,previous_json);assert_eq!(&stamp,previous_stamp);assert!(!String::from_utf8_lossy(&output.stderr).contains("Compiling serde_json"));}
   previous=Some((package,json,stamp));prepared.publish();
   let binary=temporary.join("program");let mut command=crate::backend_command();prepared.link(&mut command).unwrap();let output=command.arg("--edition=2024").arg(&prepared.application).arg("-o").arg(&binary).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));assert!(Command::new(binary).status().unwrap().success());
  }
 }
}
