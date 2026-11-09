//! Shared runtime build preparation; callers own subprocess supervision.
use std::{fs,path::{Path,PathBuf},process::Command};
use tsrust_compiler::{native_cache::NativeCache,native_runtime};
#[path="runtime_artifacts.rs"] mod artifacts;
// MSVC link.exe rejects deep output paths; Rust's bundled LLVM linker supports them.
pub(crate) const MSVC_LINKER_FLAG:&str="-Clinker-flavor=lld-link";
pub(crate) fn exported_linker_config()->String {
 format!("\n[target.'cfg(all(target_os = \"windows\", target_env = \"msvc\"))']\nrustflags = [{:?}]\n",MSVC_LINKER_FLAG)
}

// Old bundles could contain another Cargo graph's un-hashed primary library.
const DEPENDENCY_RUNTIME_ABI:&[u8]=b"runtime-dependencies-abi-6-graph-library";
/// Scope the application cache migration to programs that use a Cargo runtime.
pub(crate) fn application_cache_identity(source:&str)->Option<&'static [u8]> {
 native_runtime::requires_dependencies(source).then_some(DEPENDENCY_RUNTIME_ABI)
}

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
 cargo:Option<artifacts::Build>,
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
  let Some(runtime)=units.runtime else{return Ok(Self{application,library:None,command:None,state:"none",shared_sources:false,source_snapshot:None,cache:None,opportunistic,dependencies:None,packed:None,_workspace:None,cargo:None});};
  if units.dependencies {return Self::with_dependencies(application,&runtime,backend,root,temporary,release,opportunistic);}
  let library=temporary.join("libtsrust_runtime.rlib");
  let cache=NativeCache::runtime(&root.join("runtime"),&[b"runtime-rlib-abi-1",backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},b"edition=2024;overflow-checks=on;crate=tsrust_runtime",runtime.as_bytes()]);
  let reused=if opportunistic{cache.try_copy_to(&library)}else{cache.copy_to(&library)};
  let command=if reused {None}else{
   let input=temporary.join("runtime.rs");fs::write(&input,runtime).map_err(|e|e.to_string())?;
   let mut command=crate::backend_command();command.args(["--edition=2024","--crate-type=rlib","--crate-name=tsrust_runtime","-C",if release{"opt-level=3"}else{"opt-level=0"},"-C","overflow-checks=on"]).arg(input).arg("-o").arg(&library);Some(command)
  };
  Ok(Self{application,library:Some(library),command,state:if reused{"hit"}else{"miss"},shared_sources:false,source_snapshot:None,cache:Some(cache),opportunistic,dependencies:None,packed:None,_workspace:None,cargo:None})
 }
 fn with_dependencies(application:PathBuf,runtime:&str,backend:&str,root:&Path,temporary:&Path,release:bool,opportunistic:bool)->Result<Self,String> {
  use crate::native_dependency_bundle as bundle;
  let absolute_root=crate::absolute(root)?;fs::create_dir_all(&absolute_root).map_err(|e|e.to_string())?;let absolute_root=absolute_root.canonicalize().map_err(|e|e.to_string())?;let root=absolute_root.as_path();
  let packages=crate::rust_binding_packages::packages(runtime)?;
  let binding=crate::rust_binding_packages::checked(&packages)?;
  let dependency_identity=crate::backend_sources::dependency_identity();
  let shared_inputs=Some(crate::backend_sources::identity(root)?);
  let gpui=runtime.contains("mod tsrust_native_ui {");
  if gpui && binding.is_some(){return Err("std:gpui programs cannot import Rust packages yet".into());}
  let mut cache_parts:Vec<&[u8]>=vec![DEPENDENCY_RUNTIME_ABI,dependency_identity.as_bytes(),backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},runtime.as_bytes()];
  if gpui{cache_parts.push(crate::backend_sources::GPUI_LOCK.as_bytes());}
  if let Some(inputs)=&shared_inputs{cache_parts.push(inputs.as_bytes());}
  let cache=NativeCache::runtime(&root.join("runtime"),&cache_parts);
  // Cargo primary aliases/fingerprints can otherwise collide across support workspaces.
  // A stable external alias is supplied when linking the application.
  let library_name=format!("tsrust_runtime_{}",cache.identity_key());
  let packed=temporary.join("runtime-artifacts");let restored=temporary.join("dependencies");
  let copied=if opportunistic{cache.try_copy_to(&packed)}else{cache.copy_to(&packed)};
  if copied && bundle::unpack(&packed,&restored).is_ok() {
   return Ok(Self{application,library:Some(restored.join("libtsrust_runtime.rlib")),command:None,state:"hit",shared_sources:true,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:Some(restored),packed:Some(packed),_workspace:None,cargo:None});
  }
  let mut identity=serde_json::to_vec(&("native-build-workspace-1",dependency_identity,backend,std::env::consts::OS,std::env::consts::ARCH,release,profiles(gpui))).map_err(|e|e.to_string())?;
  if gpui{identity.extend(crate::backend_sources::GPUI_LOCK.as_bytes());}
  if let Some(binding)=binding{identity.extend(serde_json::to_vec(&(&binding.manifest,&binding.lock)).map_err(|e|e.to_string())?);}
  let workspace=crate::native_workspace::Workspace::acquire(root,&identity,opportunistic)?;
  let build_root=workspace.as_ref().map(|workspace|workspace.root.as_path()).unwrap_or(temporary);
  let package=build_root.join("native-support");
  let default=if gpui{(crate::backend_sources::GPUI_MANIFEST,crate::backend_sources::GPUI_LOCK)}else{(crate::backend_sources::MANIFEST,crate::backend_sources::LOCK)};
  let (input_manifest,input_lock)=binding.map_or(default,|p|(p.manifest.as_str(),p.lock.as_str()));
  if workspace.is_none() || !package.exists(){
   crate::backend_sources::materialize_with(&package,input_manifest,input_lock)?;
  }
  {
   crate::backend_sources::restore_missing(&package,input_manifest,input_lock)?;
   crate::backend_sources::validate(&package,input_manifest,input_lock).map_err(|e|format!("Validating native support workspace {}: {e}",package.display()))?;
  }
  let manifest=package.join("Cargo.toml");
  let mut text=tsrust_compiler::native_url_patch::normalize_manifest(input_manifest).map_err(|e|e.to_string())?;
  text.push_str(&format!("\n[lib]\nname = {library_name:?}\n"));
  text.push_str(profiles(gpui));
  crate::native_workspace::write_changed(&manifest,text)?;crate::native_workspace::write_changed(&package.join("src/lib.rs"),runtime)?;
  let explicit_target=std::env::var_os("CARGO_TARGET_DIR");
  let cargo=artifacts::Build::new(root,manifest.clone(),build_root.join("artifacts"),temporary.join("runtime.stdout"),explicit_target.as_deref().map(Path::new),&library_name)?;
  let target=&cargo.target;
  let mut command=crate::backend_cargo_command();command.current_dir(&package).args(["build","--offline","--locked","--lib","--message-format=json-render-diagnostics","--manifest-path"]).arg(&manifest)
   .arg("--config").arg(package.join(".cargo/config.toml")).arg("--target-dir").arg(target).env("CARGO_INCREMENTAL","0");
  command.current_dir(root).env("CARGO_HOME",crate::backend_sources::cargo_home()?);
  // Compiler-owned support must not inherit user Cargo profiles or wrappers:
  // those would change artifacts without changing the verified cache identity.
  // The job limit only controls scheduling, so retain the caller's memory bound.
  for (name,_) in std::env::vars_os() {if artifact_override(&name.to_string_lossy()) {command.env_remove(name);}}
  if cfg!(all(windows,target_env="msvc")){command.env("CARGO_ENCODED_RUSTFLAGS",MSVC_LINKER_FLAG);}
  if runtime.contains("pub mod tsrust_native_threads {") || runtime.contains("pub use rayon;") { command.args(["--features", "threads"]); }
  if runtime.contains("mod tsrust_native_compression_streams {"){command.args(["--features","compression-streams"]);}
  if runtime.contains("mod tsrust_native_compression {"){command.args(["--features","compression"]);}
  if tsrust_compiler::native_runtime::requires_byte_codecs(runtime){command.args(["--features","byte-codecs"]);}
  if tsrust_compiler::native_runtime::requires_byte_search(runtime){command.args(["--features","byte-search"]);}
  if tsrust_compiler::native_runtime::requires_hostname(runtime){command.args(["--features","hostname"]);}
  if runtime.contains("pub mod tsrust_native_files {"){command.args(["--features","files"]);}
  if tsrust_compiler::native_runtime::requires_temporary_files(runtime){command.args(["--features","temporary-files"]);}
  if runtime.contains("pub mod tsrust_native_process {"){command.args(["--features","process"]);}
  if runtime.contains("pub mod tsrust_native_io {"){command.args(["--features","io"]);}
  if runtime.contains("pub mod tsrust_native_futures {") {command.args(["--features","async"]);}
  if runtime.contains("pub mod tsrust_native_cancellation {"){command.args(["--features","cancellation"]);}
  if runtime.contains("pub mod tsrust_native_logging {"){command.args(["--features","logging"]);}
  if runtime.contains("pub mod tsrust_native_locale {"){command.args(["--features","locale"]);}
  if runtime.contains("pub mod tsrust_native_regex {"){command.args(["--features","regex"]);}
  if runtime.contains("pub mod tsrust_native_url {"){command.args(["--features","url"]);}
  if runtime.contains("pub mod tsrust_native_http {"){command.args(["--features","http"]);}
  if runtime.contains("// tsrust-http-websocket-selected\n"){command.args(["--features","http-websocket"]);}
  if runtime.contains("// tsrust-http-middleware-selected\n"){command.args(["--features","http-middleware"]);}
  if gpui{command.args(["--features",if !runtime.contains("use gpui_kit::"){"ui"}else if runtime.contains("// tsrust-ui-highlight-selected\n"){"gpui-highlight"}else{"gpui"}]);}

        if runtime.contains("pub mod tsrust_native_date {") {
            command.args(["--features", "calendar"]);}
  if runtime.contains("pub mod tsrust_native_crypto {"){command.args(["--features","crypto"]);}
  if runtime.contains("pub mod tsrust_native_crypto_password {"){command.args(["--features","crypto-password"]);}
  if runtime.contains("pub mod tsrust_native_signals {"){command.args(["--features","signals"]);}
  if runtime.contains("pub mod tsrust_native_uuid {"){command.args(["--features","uuid"]);}
  if runtime.contains("pub mod tsrust_native_decimal {"){command.args(["--features","decimal"]);}
  if runtime.contains("pub mod tsrust_native_sql {"){command.args(["--features","sql"]);}
  if runtime.contains("pub mod tsrust_native_sqlite {"){command.args(["--features","sqlite"]);}
  if release {command.arg("--release");}
  cargo.capture(&mut command)?;
  Ok(Self{application,library:None,command:Some(command),state:"miss",shared_sources:true,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:None,packed:Some(packed),_workspace:workspace,cargo:Some(cargo)})
 }
 /// Acquire before spawning Cargo; dev polls this without blocking cancellation.
 pub fn acquire_target(&mut self,blocking:bool)->Result<bool,String>{match &mut self.cargo{Some(cargo)=>cargo.acquire(blocking),None=>Ok(true)}}
 pub fn finish(&mut self)->Result<(),String>{
  if let Some(cargo)=&mut self.cargo {let directory=cargo.finish()?;self.library=Some(directory.join("libtsrust_runtime.rlib"));self.dependencies=Some(directory);}
  Ok(())
 }
 pub fn publish(&self) {
  if let Some((root,expected))=&self.source_snapshot
   && !crate::backend_sources::unchanged(Some(expected.as_str()),root) {return;}
  if let (Some(cache),Some(library))=(&self.cache,&self.library){let artifact=if let (Some(dependencies),Some(packed))=(&self.dependencies,&self.packed) {
   if crate::native_dependency_bundle::pack(library,dependencies,packed).is_err(){return;}packed
  }else{library};let _=if self.opportunistic{cache.try_publish(artifact)}else{cache.publish(artifact)};}}
 pub fn compilation_error(&self,detail:&str)->String {
  let diagnostics=self.cargo.as_ref().map(artifacts::Build::diagnostics).unwrap_or_default();
  let combined=format!("{detail}{diagnostics}");let detail=combined.as_str();
  let hint=if self.shared_sources && (detail.contains("offline") || detail.contains("no matching package")) {"\nPrepare missing sources explicitly with `tsrust setup --dependencies --yes`, or use `--offline` with a populated Cargo home. For binding dependencies, run `tsrust bindings resolve <file.bindings.json>` instead. Native builds never fetch sources."}else{""};
  format!("Runtime compilation failed:\n{detail}{hint}")
 }
 pub fn link(&self,command:&mut Command)->Result<(),String> {if self.cargo.is_some() && self.dependencies.is_none(){return Err("Cargo runtime artifacts have not been staged".into());}if let Some(dependencies)=&self.dependencies {
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
 #[test] fn application_cache_migration_only_tags_dependency_runtimes(){
  use tsrust_compiler::project::{SourceId,SourceProvider,load_project,check_graph};
  struct Provider(String);
  impl SourceProvider for Provider {
   fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
   fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.clone())}else{Err(format!("Missing {}",id.0))}}
  }
  for(source,dependencies)in[("function main(){console.log(1);}",false),("import {alloc,length} from 'std:bytes';function main(){console.log(length(unwrap(alloc(1))));}",false),("import {readFile} from 'std:fs';function main(){readFile('input');}",true)]{
   let checked=check_graph(load_project(&Provider(source.into()),"main.tsr").unwrap()).unwrap();
   assert_eq!(application_cache_identity(&checked.emit_rust()),dependencies.then_some(DEPENDENCY_RUNTIME_ABI));
  }
 }
 #[test] fn distinct_runtime_workspaces_preserve_fresh_cargo_graphs_and_warm_cache_links(){
  let directory=crate::TempDirectory::new().unwrap();
  let runtimes=["pub mod tsrust_native_files {pub fn witness()->i64{1}}pub mod tsrust_native_bytes {pub fn witness()->i64{serde_json::from_str::<i64>(\"41\").unwrap()}}", "pub mod tsrust_native_files {pub fn witness()->i64{serde_json::from_str::<i64>(\"7\").unwrap()}}"];
  let mut first_library=None;let mut first_target=None;let mut first_name=None;
  for(index,graph)in[0usize,1,0,0].into_iter().enumerate(){
   let temporary=directory.0.join(format!("build-{index}"));fs::create_dir(&temporary).unwrap();let application=temporary.join("program.rs");
   let expression=if graph==0{"tsrust_runtime::tsrust_native_bytes::witness()+tsrust_runtime::tsrust_native_files::witness(),42"}else{"tsrust_runtime::tsrust_native_files::witness(),7"};
   fs::write(&application,format!("fn main(){{assert_eq!({expression});}}")).unwrap();
   let cache=directory.0.join(format!("cache-{graph}"));let mut prepared=Prepared::with_dependencies(application,runtimes[graph],"pinned-graph-alias-test",&cache,&temporary,false,false).unwrap();
   if index<3 {
    assert_eq!(prepared.state,"miss");let mut command=prepared.command.take().unwrap();let manifest=command.get_args().skip_while(|arg|*arg!="--manifest-path").nth(1).map(PathBuf::from).unwrap();
    let manifest=fs::read_to_string(manifest).unwrap().parse::<toml_edit::DocumentMut>().unwrap();let name=manifest["lib"]["name"].as_str().unwrap().to_owned();
    let target=prepared.cargo.as_ref().unwrap().target.clone();// Distinct cache roots share one target only through an explicit CARGO_TARGET_DIR (as in CI).
    if let Some(first)=&first_target{if std::env::var_os("CARGO_TARGET_DIR").is_some(){assert_eq!(first,&target);}}else{first_target=Some(target);}
    prepared.acquire_target(true).unwrap();let output=command.output().unwrap();assert!(output.status.success(),"{}",prepared.compilation_error(&String::from_utf8_lossy(&output.stderr)));
    if index==2 {let text=fs::read_to_string(temporary.join("runtime.stdout")).unwrap();let fresh=text.lines().filter_map(|line|serde_json::from_str::<serde_json::Value>(line).ok()).any(|row|row["reason"]=="compiler-artifact"&&row["target"]["name"]==name&&row["fresh"]==true);assert!(fresh,"Unchanged A must use Cargo's fresh graph");}
    prepared.finish().unwrap();let bytes=fs::read(prepared.library.as_ref().unwrap()).unwrap();
    if index==0 {first_library=Some(bytes);first_name=Some(name);}else if index==1 {assert_ne!(first_name.as_ref(),Some(&name));assert_ne!(first_library.as_ref(),Some(&bytes));}else{assert_eq!(first_library.as_ref(),Some(&bytes));}
    if index>0 {prepared.publish();}
   }else{assert_eq!(prepared.state,"hit");assert!(prepared.command.is_none());assert!(prepared.cargo.is_none());}
   let binary=temporary.join(format!("program{}",std::env::consts::EXE_SUFFIX));let mut command=crate::backend_command();prepared.link(&mut command).unwrap();let output=command.arg("--edition=2024").arg(&prepared.application).arg("-o").arg(&binary).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));assert!(Command::new(binary).status().unwrap().success());
  }
 }
 #[test] fn changed_runtime_reuses_dependency_builds_and_links_current_values() {
  struct Cleanup(PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
  let dir=Cleanup(std::env::temp_dir().join(format!("tsrust-runtime-reuse-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));fs::create_dir(&dir.0).unwrap();
  let cache=dir.0.join("cache");let mut previous=None;
  for (index,value) in [42,43,43].into_iter().enumerate(){
   let temporary=dir.0.join(format!("build-{index}"));fs::create_dir(&temporary).unwrap();let application=temporary.join("program.rs");
   fs::write(&application,format!("fn main(){{assert_eq!(tsrust_runtime::answer(),{value});}}")).unwrap();
   let runtime=format!("pub fn answer()->i64{{let value:serde_json::Value=serde_json::from_str(\"{value}\").unwrap();value.as_i64().unwrap()}}");
   let mut prepared=Prepared::with_dependencies(application,&runtime,"pinned-backend-test",&cache,&temporary,false,false).unwrap();
   if index<2 {assert_eq!(prepared.state,"miss");
   let mut command=prepared.command.take().unwrap();let package=command.get_current_dir().unwrap().to_owned();
   prepared.acquire_target(true).unwrap();
   let output=command.output().unwrap();assert!(output.status.success(),"{}",prepared.compilation_error(&String::from_utf8_lossy(&output.stderr)));
   prepared.finish().unwrap();
   let dependencies=prepared.dependencies.as_ref().unwrap();
   assert_eq!(prepared.library.as_deref(),Some(dependencies.join("libtsrust_runtime.rlib").as_path()));
   let json=prepared.cargo.as_ref().unwrap().sources.iter().find(|(name,_)|name.starts_with("libserde_json-") && name.ends_with(".rlib")).unwrap().1.clone();
   assert_ne!(json.parent().unwrap(),dependencies);assert!(!package.join("native-target").exists());
   let stamp=fs::metadata(&json).unwrap().modified().unwrap();
   if let Some((previous_package,previous_json,previous_stamp))=&previous {assert_eq!(&package,previous_package);assert_eq!(&json,previous_json);assert_eq!(&stamp,previous_stamp);assert!(!String::from_utf8_lossy(&output.stderr).contains("Compiling serde_json"));}
   previous=Some((package,json,stamp));prepared.publish();
   }else{assert_eq!(prepared.state,"hit");assert!(prepared.command.is_none());assert!(prepared.cargo.is_none());}
   let binary=temporary.join("program");let mut command=crate::backend_command();prepared.link(&mut command).unwrap();let output=command.arg("--edition=2024").arg(&prepared.application).arg("-o").arg(&binary).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));assert!(Command::new(binary).status().unwrap().success());
  }
 }
}

#[test]
fn exported_linker_policy_is_valid_toml_and_windows_msvc_only() {
 let text=format!("[net]\noffline = true\n{}",exported_linker_config());
 let config=text.parse::<toml_edit::DocumentMut>().unwrap();
 assert_eq!(config["net"]["offline"].as_bool(),Some(true));
 assert!(config.get("build").is_none());
 let targets=config["target"].as_table().unwrap();assert_eq!(targets.len(),1);
 assert_eq!(targets["cfg(all(target_os = \"windows\", target_env = \"msvc\"))"]["rustflags"][0].as_str(),Some(MSVC_LINKER_FLAG));
}
