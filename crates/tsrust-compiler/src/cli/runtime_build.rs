//! Shared runtime build preparation; callers own subprocess supervision.
use std::{fs,path::{Path,PathBuf},process::Command};
use tsrust_compiler::{native_cache::NativeCache,native_runtime};
use sha2::{Digest,Sha256};
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
 if !native_runtime::requires_dependencies(source){return None;}
 if !source.contains("mod tsrust_native_ui {"){return Some(DEPENDENCY_RUNTIME_ABI);}
 static GPUI_IDENTITY:std::sync::OnceLock<Vec<u8>>=std::sync::OnceLock::new();
 Some(GPUI_IDENTITY.get_or_init(||gpui_application_identity(crate::backend_sources::GPUI_LOCK.as_bytes(),&tsrust_compiler::native_dependency_patches::source_identity(crate::backend_sources::GPUI_MANIFEST).expect("bundled GPUI patch manifest"))).as_slice())
}
fn gpui_application_identity(lock:&[u8],patch:&str)->Vec<u8>{
 let mut hash=Sha256::new();for part in [lock,patch.as_bytes()]{hash.update((part.len() as u64).to_le_bytes());hash.update(part);}
 let digest=hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect::<String>();
 [DEPENDENCY_RUNTIME_ABI,b";gpui=",digest.as_bytes()].concat()
}

pub(super) fn artifact_override(key:&str)->bool {
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
 /// Per-module crates for this development build (off with `TSRUST_CRATE_SPLIT=0`).
 pub split:Option<SplitCrates>,
}
impl Prepared {
    pub fn new(
        source: &str,
        backend: &str,
        root: &Path,
        temporary: &Path,
        release: bool,
    ) -> Result<Self, String> {
        Self::prepare(source, backend, root, temporary, release, false, None)
    }
    /// As [`Self::new`], naming generated items through an incremental session ledger.
    pub fn new_named(
        source: &str,
        backend: &str,
        root: &Path,
        temporary: &Path,
        release: bool,
        naming: Option<&Naming>,
    ) -> Result<Self, String> {
        Self::prepare(source, backend, root, temporary, release, false, naming)
    }
    pub fn for_dev(
        source: &str,
        backend: &str,
        root: &Path,
        temporary: &Path,
        release: bool,
        naming: Option<&Naming>,
    ) -> Result<Self, String> {
        Self::prepare(source, backend, root, temporary, release, true, naming)
    }
    fn prepare(
        source: &str,
        backend: &str,
        root: &Path,
        temporary: &Path,
        release: bool,
        opportunistic: bool,
        naming: Option<&Naming>,
    ) -> Result<Self, String> {
        let units = native_runtime::separate(source);
        let application = temporary.join("program.rs");
        // Stable names keep unchanged items byte-identical across edits; split afterwards.
        let mut split = None;
        let named = match naming {
            Some(naming) => {
                let mut ledger = tsrust_compiler::native_stable_names::Ledger::load(&naming.ledger);
                let (named, origins) = tsrust_compiler::native_stable_names::stabilize_with_origins(
                    &units.application,
                    &naming.keys,
                    &mut ledger,
                );
                ledger.save(&naming.ledger);
                if crate_split_enabled() && !naming.keys.modules.is_empty() {
                    split = SplitCrates::write(&named, &naming.keys, &origins, naming)?;
                }
                named
            }
            None => units.application.clone(),
        };
        if split.is_some() {
            // The crates are the program; the single-file form is only a fallback marker.
            fs::write(&application, "").map_err(|e| e.to_string())?;
        } else if naming.is_some() {
            // Incremental builds spread items over files so an insertion shifts lines in one file.
            for part in tsrust_compiler::native_codegen_units::partition_files(
                &named,
                tsrust_compiler::native_codegen_units::DEFAULT_UNIT_BYTES,
            ) {
                fs::write(temporary.join(&part.file_name), part.text).map_err(|e| e.to_string())?;
            }
        } else {
            fs::write(
                &application,
                tsrust_compiler::native_codegen_units::partition(&named),
            )
            .map_err(|e| e.to_string())?;
        }
        let Some(runtime) = units.runtime else {
            return Ok(Self {
                application,
                library: None,
                command: None,
                state: "none",
                shared_sources: false,
                source_snapshot: None,
                cache: None,
                opportunistic,
                dependencies: None,
                packed: None,
                _workspace: None,
                cargo: None,
                split,
            });
        };
        if units.dependencies {
            let mut prepared = Self::with_dependencies(
                application,
                &runtime,
                backend,
                root,
                temporary,
                release,
                opportunistic,
            )?;
            prepared.split = split;
            return Ok(prepared);
        }
        let library = temporary.join("libtsrust_runtime.rlib");
        let cache = NativeCache::runtime(
            &root.join("runtime"),
            &[
                b"runtime-rlib-abi-1",
                backend.as_bytes(),
                std::env::consts::OS.as_bytes(),
                std::env::consts::ARCH.as_bytes(),
                if release { b"release" } else { b"debug" },
                b"edition=2024;overflow-checks=on;crate=tsrust_runtime",
                runtime.as_bytes(),
            ],
        );
        let reused = if opportunistic {
            cache.try_copy_to(&library)
        } else {
            cache.copy_to(&library)
        };
        let command = if reused {
            None
        } else {
            let input = temporary.join("runtime.rs");
            fs::write(&input, runtime).map_err(|e| e.to_string())?;
            let mut command = crate::backend_command();
            command
                .args([
                    "--edition=2024",
                    "--crate-type=rlib",
                    "--crate-name=tsrust_runtime",
                    "-C",
                    if release {
                        "opt-level=3"
                    } else {
                        "opt-level=0"
                    },
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(input)
                .arg("-o")
                .arg(&library);
            Some(command)
        };
        Ok(Self {
            application,
            library: Some(library),
            command,
            state: if reused { "hit" } else { "miss" },
            shared_sources: false,
            source_snapshot: None,
            cache: Some(cache),
            opportunistic,
            dependencies: None,
            packed: None,
            _workspace: None,
            cargo: None,
            split,
        })
    }
 fn with_dependencies(application:PathBuf,runtime:&str,backend:&str,root:&Path,temporary:&Path,release:bool,opportunistic:bool)->Result<Self,String> {

  let absolute_root=crate::absolute(root)?;fs::create_dir_all(&absolute_root).map_err(|e|e.to_string())?;let absolute_root=absolute_root.canonicalize().map_err(|e|e.to_string())?;let root=absolute_root.as_path();
  let packages=crate::rust_binding_packages::packages(runtime)?;
  let binding=crate::rust_binding_packages::checked(&packages)?;
  let dependency_identity=crate::backend_sources::dependency_identity();
  let shared_inputs=Some(crate::backend_sources::identity(root,true)?);
  let gpui=runtime.contains("mod tsrust_native_ui {");
  if gpui && binding.is_some(){return Err("std:gpui programs cannot import Rust packages yet".into());}
  let mut cache_parts:Vec<&[u8]>=vec![DEPENDENCY_RUNTIME_ABI,dependency_identity.as_bytes(),backend.as_bytes(),std::env::consts::OS.as_bytes(),std::env::consts::ARCH.as_bytes(),if release{b"release"}else{b"debug"},runtime.as_bytes()];
  let gpui_patch=if gpui{tsrust_compiler::native_dependency_patches::source_identity(crate::backend_sources::GPUI_MANIFEST).map_err(|e|e.to_string())?}else{String::new()};
  if gpui{cache_parts.push(crate::backend_sources::GPUI_LOCK.as_bytes());cache_parts.push(gpui_patch.as_bytes());}
  // Profile settings change compiled artifacts, so they key the cache too.
  cache_parts.push(profiles(gpui).as_bytes());
  if let Some(inputs)=&shared_inputs{cache_parts.push(inputs.as_bytes());}
  let cache=NativeCache::runtime(&root.join("runtime"),&cache_parts);
  let packed=temporary.join("runtime-artifacts");
  if let Some(restored)=staged_dependencies(root,&cache,&packed,opportunistic) {
   return Ok(Self{application,library:Some(restored.join("libtsrust_runtime.rlib")),command:None,state:"hit",shared_sources:true,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:Some(restored),packed:Some(packed),_workspace:None,cargo:None,split:None});
  }
  let mut identity=serde_json::to_vec(&("native-build-workspace-1",dependency_identity,backend,std::env::consts::OS,std::env::consts::ARCH,release,profiles(gpui))).map_err(|e|e.to_string())?;
  if gpui{identity.extend(crate::backend_sources::GPUI_LOCK.as_bytes());identity.extend(gpui_patch.as_bytes());}
  if let Some(binding)=binding{identity.extend(serde_json::to_vec(&(&binding.manifest,&binding.lock)).map_err(|e|e.to_string())?);}
  let library_name=runtime_library_name(&crate::native_workspace::Workspace::location(root,&identity),runtime);
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
  let mut text=tsrust_compiler::native_dependency_patches::normalize_manifest(input_manifest).map_err(|e|e.to_string())?;
  text.push_str(&format!("\n[lib]\nname = {library_name:?}\n"));
  text.push_str(profiles(gpui));
  crate::native_workspace::write_changed(&manifest,text)?;crate::native_workspace::write_changed(&package.join("src/lib.rs"),runtime)?;
  let explicit_target=std::env::var_os("CARGO_TARGET_DIR");
  let cargo=artifacts::Build::new(root,manifest.clone(),build_root.join("artifacts"),temporary.join("runtime.stdout"),explicit_target.as_deref().map(Path::new),&library_name)?;
  // GPUI shader build scripts append `/file` to OUT_DIR, invalid in verbatim Windows paths.
  // Keep the canonical target for locking; give Cargo its ordinary drive/UNC spelling.
  let target=cargo_target_path(&cargo.target,cfg!(windows));
  let mut command=crate::backend_cargo_command();command.current_dir(&package).args(["build","--offline","--locked","--lib","--message-format=json-render-diagnostics","--manifest-path"]).arg(&manifest)
   .arg("--config").arg(package.join(".cargo/config.toml")).arg("--target-dir").arg(&target).env("CARGO_INCREMENTAL","0");
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
  if native_runtime::requires_discovery(runtime){command.args(["--features","file-discovery"]);}
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
  if gpui{
   command.env("TSRUST_GPUI_PATCH_REVISION",&gpui_patch);
   // GPUI programs add opt-in Kit features only for the components they use.
   let mut features=vec![if runtime.contains("use gpui_kit::"){"gpui"}else{"ui"}];
   if runtime.contains("// tsrust-ui-highlight-selected\n"){features.push("gpui-highlight");}
   if runtime.contains("// tsrust-ui-speech-selected\n"){features.push("gpui-speech");}
   if runtime.contains("// tsrust-ui-http-selected\n"){features.push("gpui-http");}
   if runtime.contains("// tsrust-ui-terminal-selected\n"){features.push("gpui-terminal");}
   command.args(["--features",&features.join(",")]);
  }

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
  Ok(Self{application,library:None,command:Some(command),state:"miss",shared_sources:true,source_snapshot:shared_inputs.map(|inputs|(root.to_owned(),inputs)),cache:Some(cache),opportunistic,dependencies:None,packed:Some(packed),_workspace:workspace,cargo:Some(cargo),split:None})
 }
 /// Acquire before spawning Cargo; dev polls this without blocking cancellation.
 pub fn acquire_target(&mut self,blocking:bool)->Result<bool,String>{match &mut self.cargo{Some(cargo)=>cargo.acquire(blocking),None=>Ok(true)}}
 pub fn finish(&mut self)->Result<(),String>{
  if let Some(cargo)=&mut self.cargo {let directory=cargo.finish()?;self.library=Some(directory.join("libtsrust_runtime.rlib"));self.dependencies=Some(directory);}
  Ok(())
 }
 pub fn publish(&self) {
  if let Some((root,expected))=&self.source_snapshot
   && !crate::backend_sources::unchanged(Some(expected.as_str()),root,true) {return;}
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

/// Unpack a cached dependency bundle once per runtime identity and reuse the directory.
/// A GPUI bundle is near a gigabyte; copying and unpacking it on every rebuild cost
/// seconds. The directory is published by an atomic rename after a completion marker,
/// so concurrent builds see either nothing or a complete, immutable set.
fn staged_dependencies(root:&Path,cache:&NativeCache,packed:&Path,opportunistic:bool)->Option<PathBuf>{
 let parent=root.join(STAGED_DIRECTORY);
 let staged=parent.join(cache.identity_key());
 if staged.join(STAGED_MARKER).is_file(){touch(&staged.join(STAGED_MARKER));return Some(staged);}
 let copied=if opportunistic{cache.try_copy_to(packed)}else{cache.copy_to(packed)};
 if !copied {return None;}
 fs::create_dir_all(&parent).ok()?;
 prune_staged(&parent,STAGED_ENTRIES);
 let pending=parent.join(format!("{}.pending-{}",cache.identity_key(),std::process::id()));
 let _=fs::remove_dir_all(&pending);
 if crate::native_dependency_bundle::unpack(packed,&pending).is_err(){let _=fs::remove_dir_all(&pending);return None;}
 fs::write(pending.join(STAGED_MARKER),b"").ok()?;
 if fs::rename(&pending,&staged).is_err(){let _=fs::remove_dir_all(&pending);}
 staged.join(STAGED_MARKER).is_file().then_some(staged)
}
fn touch(path:&Path){if let Ok(file)=fs::File::options().write(true).open(path){let _=file.set_modified(std::time::SystemTime::now());}}
const STAGED_DIRECTORY:&str="runtime-staged";
const STAGED_MARKER:&str=".complete";
const STAGED_ENTRIES:usize=3;
fn prune_staged(parent:&Path,keep:usize){
 let Ok(entries)=fs::read_dir(parent) else{return};
 let mut staged:Vec<(std::time::SystemTime,PathBuf)>=entries.flatten().filter(|entry|entry.file_type().is_ok_and(|kind|kind.is_dir()))
  .map(|entry|(fs::metadata(entry.path().join(STAGED_MARKER)).and_then(|meta|meta.modified()).unwrap_or(std::time::UNIX_EPOCH),entry.path())).collect();
 if staged.len()<keep{return;}
 staged.sort_by(|a,b|b.0.cmp(&a.0));
 for (_,path) in staged.into_iter().skip(keep-1){let _=fs::remove_dir_all(path);}
}

// Keep Cargo graph/body aliases distinct without renaming them for environment-only misses.
fn runtime_library_name(workspace:&Path,runtime:&str)->String {
 use std::hash::{Hash,Hasher};
 let mut hash=std::collections::hash_map::DefaultHasher::new();
 (workspace,runtime).hash(&mut hash);
 format!("tsrust_runtime_{:016x}",hash.finish())
}

/// Match the native UI support profile in application code as well. Large retained
/// component compositions otherwise keep every debug temporary on the Windows
/// main thread stack, even while the separately built renderer is optimized.
/// Overflow checks remain enabled independently at both compiler call sites.
pub(crate) fn application_optimization(release:bool,native_ui:bool)->&'static str {
 if release {"opt-level=3"} else if native_ui {"opt-level=1"} else {"opt-level=0"}
}

/// Development application builds keep rustc's incremental query cache per program
/// entry, so an edit re-checks and re-codegens only changed generated items. Release
/// builds stay non-incremental to keep full cross-item optimization. Lints are capped:
/// generated Rust is not user source, and rendering thousands of style warnings costs
/// seconds per build while hiding real errors in failure output.
/// Returns the generated source path to compile: incremental sessions use a stable copy,
/// because rustc keys cached items by source file name and a temporary path would
/// invalidate every item on each build.
pub(crate) fn application_backend_args(
    command: &mut Command,
    cache_root: &Path,
    entry: &Path,
    release: bool,
    application: &Path,
) -> PathBuf {
    command.args(["--cap-lints", "allow"]);
    if release {
        return application.to_owned();
    }
    let session = incremental_session(cache_root, entry);
    let stable = session.join("program.rs");
    // Copy the multi-file parts too, leaving unchanged files (and their mtimes) alone.
    if let Some(directory) = application.parent()
        && let Ok(entries) = fs::read_dir(directory)
    {
        let mut current = std::collections::BTreeSet::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(text_name) = name.to_str() else {
                continue;
            };
            if !(text_name.starts_with("tsrust_part_") && text_name.ends_with(".rs")) {
                continue;
            }
            current.insert(name.clone());
            let Ok(bytes) = fs::read(entry.path()) else {
                continue;
            };
            let target = session.join(&name);
            if fs::read(&target).ok().as_deref() != Some(bytes.as_slice()) {
                let pending = session.join(format!("{text_name}.{}", std::process::id()));
                if fs::write(&pending, &bytes)
                    .and_then(|()| fs::rename(&pending, &target))
                    .is_err()
                {
                    let _ = fs::remove_file(&pending);
                    return application.to_owned();
                }
            }
        }
        // Remove parts a previous build used that this one no longer has.
        if let Ok(entries) = fs::read_dir(&session) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name
                    .to_str()
                    .is_some_and(|text| text.starts_with("tsrust_part_") && text.ends_with(".rs"))
                    && !current.contains(&name)
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    let source = match fs::read(application) {
        Ok(source) => source,
        Err(_) => return application.to_owned(),
    };
    // Keep the old file (and its mtime) when the generated text is unchanged.
    if fs::read(&stable).ok().as_deref() != Some(source.as_slice()) {
        // Rename atomically: a concurrent build of the same entry reads a whole file.
        let pending = session.join(format!("program.rs.{}", std::process::id()));
        if fs::write(&pending, &source)
            .and_then(|()| fs::rename(&pending, &stable))
            .is_err()
        {
            let _ = fs::remove_file(&pending);
            return application.to_owned();
        }
    }
    command
        .arg("-C")
        .arg(format!("incremental={}", session.join("rustc").display()));
    stable
}
/// The incremental session directory for one program entry, created and marked used.
pub(crate) fn incremental_session(cache_root: &Path, entry: &Path) -> PathBuf {
    let directory = cache_root.join(INCREMENTAL_DIRECTORY);
    let _ = fs::create_dir_all(&directory);
    prune_incremental(&directory, INCREMENTAL_ENTRIES);
    let mut hash = Sha256::new();
    hash.update(b"application-incremental-1");
    hash.update(entry.to_string_lossy().as_bytes());
    let key = hash
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let session = directory.join(key);
    let _ = fs::create_dir_all(&session);
    // Touch so pruning keeps recently used programs.
    let marker = session.join(".used");
    if !marker.exists() {
        let _ = fs::write(&marker, b"");
    }
    touch(&marker);
    session
}

/// Stable naming for an incremental build: item keys and the session's ledger.
pub(crate) struct Naming {
    pub keys: tsrust_compiler::native_stable_names::Keys,
    pub ledger: PathBuf,
}
impl Naming {
    pub(crate) fn for_session(
        keys: tsrust_compiler::native_stable_names::Keys,
        cache_root: &Path,
        entry: &Path,
    ) -> Self {
        Self {
            keys,
            ledger: incremental_session(cache_root, entry).join("names.json"),
        }
    }
}

/// Per-module crates for development builds (docs/plans/crate-per-module.md).
/// On by default; `TSRUST_CRATE_SPLIT=0` keeps the single-crate build.
pub(crate) fn crate_split_enabled() -> bool {
    std::env::var("TSRUST_CRATE_SPLIT").as_deref() != Ok("0")
}

/// A program split into one crate per source module, written to the incremental
/// session. Unchanged crate files keep their bytes and mtimes, and a crate whose
/// source, flags and dependencies are unchanged is not recompiled at all.
pub(crate) struct SplitCrates {
    directory: PathBuf,
    crates: Vec<SplitCrate>,
}
struct SplitCrate {
    name: String,
    dependencies: Vec<String>,
    program: bool,
}
impl SplitCrates {
    fn write(
        named: &str,
        keys: &tsrust_compiler::native_stable_names::Keys,
        origins: &tsrust_compiler::native_stable_names::Origins,
        naming: &Naming,
    ) -> Result<Option<Self>, String> {
        let Some(session) = naming.ledger.parent() else {
            return Ok(None);
        };
        let directory = session.join("split");
        let function_module = |number: u64| {
            let original = origins.functions.get(&number).copied().unwrap_or(number);
            keys.function_modules.get(usize::try_from(original).ok()?).copied().flatten()
        };
        let closure_module = |number: u64| {
            let original = origins.closures.get(&number).copied().unwrap_or(number);
            keys.closure_modules.get(usize::try_from(original).ok()?).copied().flatten()
        };
        let provenance = tsrust_compiler::native_crate_split::Provenance {
            modules: &keys.modules,
            function_module: &function_module,
            closure_module: &closure_module,
        };
        let Some(crates) = tsrust_compiler::native_crate_split::split(named, &provenance) else {
            return Ok(None);
        };
        fs::create_dir_all(directory.join("lib")).map_err(|e| e.to_string())?;
        let mut current = std::collections::BTreeSet::new();
        for unit in &crates {
            let file = format!("{}.rs", unit.name);
            write_if_changed(&directory.join(&file), unit.source.as_bytes())?;
            current.insert(file);
        }
        if let Ok(entries) = fs::read_dir(&directory) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".rs") && !current.contains(&name) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        Ok(Some(Self {
            directory,
            crates: crates
                .into_iter()
                .map(|unit| SplitCrate {
                    name: unit.name,
                    dependencies: unit.dependencies,
                    program: unit.program,
                })
                .collect(),
        }))
    }

    /// Build every crate with `base` (backend, runtime link flags and codegen
    /// options), the types crate first, then module crates in parallel, then the
    /// program into `binary`. Returns a timing summary, or `None` when `cancelled`
    /// was set: running compilers are killed and no further crates start.
    pub(crate) fn build(
        &self,
        base: &(dyn Fn() -> Command + Sync),
        binary: &Path,
        runtime: Option<&Path>,
        cancelled: &std::sync::atomic::AtomicBool,
    ) -> Result<Option<String>, String> {
        use std::sync::atomic::Ordering::Relaxed;
        let started = std::time::Instant::now();
        let lib = self.directory.join("lib");
        if let Some(runtime) = runtime {
            alias_runtime(runtime, &lib)?;
        }
        let mut flags = format!("{:?}", base().get_args().collect::<Vec<_>>());
        // The staged runtime directory is keyed by the whole environment, so the
        // same runtime appears under another path after an unrelated variable
        // changes. Fingerprint it by content; crates built against it stay valid.
        if let Some(runtime) = runtime
            && let Some(directory) = runtime.parent()
        {
            flags = flags.replace(&directory.display().to_string(), &format!("<runtime {}>", content_identity(runtime)?));
        }
        let fingerprints = std::sync::Mutex::new(std::collections::HashMap::<String, String>::new());
        let rebuilt = std::sync::atomic::AtomicUsize::new(0);
        let run = |unit: &SplitCrate| -> Result<(), String> {
            let source = self.directory.join(format!("{}.rs", unit.name));
            let output = if unit.program { binary.to_owned() } else { lib.join(format!("lib{}.rlib", unit.name)) };
            let mut hash = Sha256::new();
            hash.update(b"split-crate-1");
            hash.update(flags.as_bytes());
            hash.update(fs::read(&source).map_err(|e| e.to_string())?);
            {
                let known = fingerprints.lock().map_err(|e| e.to_string())?;
                for dependency in &unit.dependencies {
                    hash.update(dependency.as_bytes());
                    hash.update(known.get(dependency).map(String::as_str).unwrap_or("?").as_bytes());
                }
            }
            let fingerprint = hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            let stamp = self.directory.join("incremental").join(format!("{}.fingerprint", unit.name));
            let fresh = !unit.program
                && output.is_file()
                && fs::read_to_string(&stamp).ok().as_deref() == Some(fingerprint.as_str());
            if !fresh {
                if cancelled.load(Relaxed) {
                    return Err(String::new());
                }
                // A cancelled compile can leave a partial output; forget the old stamp first.
                let _ = fs::remove_file(&stamp);
                let mut command = base();
                command.arg("-L").arg(format!("dependency={}", lib.display()));
                // Crates built against the runtime find it again by search path.
                if let Some(directory) = runtime.and_then(Path::parent) {
                    command.arg("-L").arg(format!("dependency={}", directory.display()));
                }
                for dependency in &unit.dependencies {
                    command.arg("--extern").arg(format!("{dependency}={}", lib.join(format!("lib{dependency}.rlib")).display()));
                }
                command.arg("-C").arg(format!("incremental={}", self.directory.join("incremental").join(&unit.name).display()));
                if !unit.program {
                    command.args(["--crate-type", "rlib", "--crate-name", &unit.name]);
                }
                let _ = fs::create_dir_all(self.directory.join("incremental"));
                let errors = self.directory.join("incremental").join(format!("{}.stderr", unit.name));
                let mut child = command
                    .arg(&source)
                    .arg("-o")
                    .arg(&output)
                    .stdout(std::process::Stdio::null())
                    .stderr(fs::File::create(&errors).map_err(|e| e.to_string())?)
                    .spawn()
                    .map_err(|e| format!("Cannot invoke pinned Rust: {e}"))?;
                let status = loop {
                    if cancelled.load(Relaxed) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(String::new());
                    }
                    if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                        break status;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                };
                let stderr = fs::read_to_string(&errors).unwrap_or_default();
                let _ = fs::remove_file(&errors);
                if !status.success() {
                    return Err(format!(
                        "Rust rejected generated code. This is a compiler defect; inspect with the emit command.\n{}",
                        stderr.trim()
                    ));
                }
                if !unit.program {
                    let _ = fs::write(&stamp, &fingerprint);
                }
                rebuilt.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            fingerprints.lock().map_err(|e| e.to_string())?.insert(unit.name.clone(), fingerprint);
            Ok(())
        };
        // An empty error means the build was cancelled.
        let finish = |result: Result<(), String>| match result {
            Err(error) if error.is_empty() => Ok(false),
            Err(error) => Err(error),
            Ok(()) => Ok(true),
        };
        let types = self.crates.iter().filter(|unit| unit.dependencies.is_empty() && !unit.program);
        for unit in types {
            if !finish(run(unit))? {
                return Ok(None);
            }
        }
        let types_done = started.elapsed();
        let modules = self.crates.iter().filter(|unit| !unit.dependencies.is_empty() && !unit.program).collect::<Vec<_>>();
        let queue = std::sync::Mutex::new(modules.into_iter());
        let failure = std::sync::Mutex::new(None::<String>);
        let workers = std::thread::available_parallelism().map_or(4, usize::from);
        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| loop {
                    if failure.lock().map(|failed| failed.is_some()).unwrap_or(true) {
                        return;
                    }
                    let Some(unit) = queue.lock().ok().and_then(|mut queue| queue.next()) else {
                        return;
                    };
                    if let Err(error) = run(unit)
                        && let Ok(mut failed) = failure.lock()
                    {
                        failed.get_or_insert(error);
                    }
                });
            }
        });
        if let Some(error) = failure.into_inner().map_err(|e| e.to_string())?
            && !finish(Err(error))?
        {
            return Ok(None);
        }
        let modules_done = started.elapsed();
        for unit in self.crates.iter().filter(|unit| unit.program) {
            if !finish(run(unit))? {
                return Ok(None);
            }
        }
        Ok(Some(format!(
            "crates={} rebuilt={} types={:.1}ms modules={:.1}ms program={:.1}ms",
            self.crates.len(),
            rebuilt.load(std::sync::atomic::Ordering::Relaxed),
            types_done.as_secs_f64() * 1000.0,
            (modules_done - types_done).as_secs_f64() * 1000.0,
            (started.elapsed() - modules_done).as_secs_f64() * 1000.0
        )))
    }
}

/// SHA-256 of a file, remembered beside it by length and modification time.
fn content_identity(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let modified = metadata.modified().ok().and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |time| time.as_nanos());
    let key = format!("{} {modified}", metadata.len());
    let stamp = path.with_extension("rlib.identity");
    if let Ok(text) = fs::read_to_string(&stamp)
        && let Some((recorded, identity)) = text.rsplit_once(' ')
        && recorded == key
    {
        return Ok(identity.to_owned());
    }
    let identity = Sha256::digest(fs::read(path).map_err(|e| e.to_string())?).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let _ = fs::write(&stamp, format!("{key} {identity}"));
    Ok(identity)
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let pending = path.with_extension(format!("rs.{}", std::process::id()));
    fs::write(&pending, bytes)
        .and_then(|()| fs::rename(&pending, path))
        .map_err(|e| {
            let _ = fs::remove_file(&pending);
            e.to_string()
        })
}

/// A Cargo-built runtime rlib names its crate `tsrust_runtime_<hash>`; crates
/// compiled against it find it by that name, so link it under that file name.
fn alias_runtime(runtime: &Path, lib: &Path) -> Result<(), String> {
    let bytes = fs::read(runtime).map_err(|e| e.to_string())?;
    let prefix = b"tsrust_runtime_";
    let name = bytes.windows(prefix.len() + 16).find(|window| {
        window.starts_with(prefix) && window[prefix.len()..].iter().all(|byte| byte.is_ascii_hexdigit())
    });
    let Some(name) = name else {
        return Ok(());
    };
    let alias = lib.join(format!("lib{}.rlib", String::from_utf8_lossy(name)));
    if fs::metadata(&alias).is_ok_and(|meta| meta.len() == bytes.len() as u64) {
        return Ok(());
    }
    let _ = fs::remove_file(&alias);
    fs::hard_link(runtime, &alias).or_else(|_| fs::write(&alias, &bytes)).map_err(|e| e.to_string())
}

const INCREMENTAL_DIRECTORY:&str="incremental";
const INCREMENTAL_ENTRIES:usize=6;
fn prune_incremental(directory:&Path,keep:usize){
 let Ok(entries)=fs::read_dir(directory) else{return};
 let mut sessions:Vec<(std::time::SystemTime,PathBuf)>=entries.flatten().filter(|entry|entry.file_type().is_ok_and(|kind|kind.is_dir()))
  .map(|entry|{let used=fs::metadata(entry.path().join(".used")).and_then(|meta|meta.modified()).unwrap_or(std::time::UNIX_EPOCH);(used,entry.path())}).collect();
 if sessions.len()<keep{return;}
 sessions.sort_by(|a,b|b.0.cmp(&a.0));
 for (_,path) in sessions.into_iter().skip(keep-1){let _=fs::remove_dir_all(path);}
}

#[cfg(test)] mod application_backend_tests {
 use super::*;
 #[test] fn development_builds_are_incremental_per_entry_and_release_builds_are_not(){
  let directory=crate::TempDirectory::new().unwrap();let root=directory.0.as_path();
  let generated=root.join("generated.rs");fs::write(&generated,"fn main(){}").unwrap();
  let args=|entry:&str,release:bool|{let mut command=Command::new("rustc");let source=application_backend_args(&mut command,root,Path::new(entry),release,&generated);(command.get_args().map(|arg|arg.to_string_lossy().into_owned()).collect::<Vec<_>>(),source)};
  assert_eq!(args("/a/main.tsr",true),(vec!["--cap-lints".to_owned(),"allow".to_owned()],generated.clone()));
  let (first,source)=args("/a/main.tsr",false);let (again,same)=args("/a/main.tsr",false);let (other,elsewhere)=args("/b/main.tsr",false);
  assert!(first[3].starts_with("incremental="));assert_eq!(first,again);assert_ne!(first,other);
  assert_eq!(source,same);assert_ne!(source,elsewhere);assert_ne!(source,generated);
  assert_eq!(fs::read_to_string(&source).unwrap(),"fn main(){}");
 }
 #[test] fn pruning_keeps_the_most_recently_used_sessions(){
  let directory=crate::TempDirectory::new().unwrap();let root=directory.0.join(INCREMENTAL_DIRECTORY);
  let generated=directory.0.join("generated.rs");fs::write(&generated,"fn main(){}").unwrap();
  for index in 0..INCREMENTAL_ENTRIES+2 {
   let mut command=Command::new("rustc");application_backend_args(&mut command,&directory.0,Path::new(&format!("/entry/{index}")),false,&generated);
   std::thread::sleep(std::time::Duration::from_millis(15));
  }
  assert_eq!(fs::read_dir(&root).unwrap().count(),INCREMENTAL_ENTRIES);
  let mut newest=Command::new("rustc");let source=application_backend_args(&mut newest,&directory.0,Path::new(&format!("/entry/{}",INCREMENTAL_ENTRIES+1)),false,&generated);
  assert!(source.exists(),"the newest session survives pruning");
 }
}

#[cfg(test)] mod application_profile_tests {
 #[test] fn native_ui_development_matches_support_without_changing_backend_or_release(){
  use super::application_optimization;
  assert_eq!(application_optimization(false,true),"opt-level=1");
  assert_eq!(application_optimization(false,false),"opt-level=0");
  for ui in [false,true]{assert_eq!(application_optimization(true,ui),"opt-level=3");}
 }
 /// Faults are planned to unwind to their catch points (docs/plans/fault-unwinding.md);
 /// a profile that aborts on panic would turn every caught fault into a crash.
 #[test] fn generated_profiles_keep_panics_unwinding(){
  for ui in [false,true]{
   assert!(!super::profiles(ui).contains("panic"),"profiles must not choose a panic strategy");
   for release in [false,true]{assert!(!super::application_optimization(release,ui).contains("panic"));}
  }
 }
}

/// UI rendering must remain responsive in development; omit bulky dependency debug symbols.
/// The runtime build profile a program links against; it keys application caches.
pub(crate) fn profile_identity(rust: &str) -> &'static str {profiles(rust.contains("mod tsrust_native_ui {"))}

fn cargo_target_path(path:&Path,windows:bool)->PathBuf {
 if windows && let Some(text)=path.to_str(){return PathBuf::from(tsrust_compiler::display_path(text).as_ref());}
 path.to_owned()
}

#[test]
fn cargo_target_paths_allow_build_scripts_to_append_forward_slashes(){
 for (canonical,ordinary) in [(r"\\?\D:\a\tsrust\target",r"D:\a\tsrust\target"),(r"\\?\UNC\server\share\target",r"\\server\share\target")]{
  assert_eq!(cargo_target_path(Path::new(canonical),true),PathBuf::from(ordinary));
  assert_eq!(cargo_target_path(Path::new(canonical),false),PathBuf::from(canonical));
 }
 assert_eq!(cargo_target_path(Path::new("/tmp/target"),false),PathBuf::from("/tmp/target"));
}

pub(crate) fn profiles(native_ui: bool) -> &'static str {
 // Third-party debug assertions stay off in dev, as in release: GPUI panics when
 // GPUI Kit's own NumberInput draws one focus handle twice for accessibility.
 if native_ui {"\n[profile.dev]\noverflow-checks = true\nincremental = false\ndebug = false\nopt-level = 1\n[profile.dev.package.\"*\"]\nopt-level = 2\ndebug = false\ndebug-assertions = false\n[profile.release]\noverflow-checks = true\nincremental = false\n"}
 else {"\n[profile.dev]\noverflow-checks = true\nincremental = false\n[profile.release]\noverflow-checks = true\nincremental = false\n"}
}

#[cfg(test)] mod dependency_reuse_tests {
 use super::*;
 #[test] fn gpui_application_identity_tracks_lock_and_patch_bytes(){
  let first=gpui_application_identity(b"lock-a","patch-a");
  assert_ne!(first,gpui_application_identity(b"lock-b","patch-a"));
  assert_ne!(first,gpui_application_identity(b"lock-a","patch-b"));
  assert!(first.starts_with(DEPENDENCY_RUNTIME_ABI));
  assert_ne!(first,DEPENDENCY_RUNTIME_ABI);
 }
 #[test] fn application_cache_migration_only_tags_dependency_runtimes(){
  use tsrust_compiler::project::{SourceId,SourceProvider,load_project,check_graph};
  struct Provider(String);
  impl SourceProvider for Provider {
   fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
   fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.clone())}else{Err(format!("Missing {}",id.0))}}
  }
  for(source,dependencies)in[("function main(){console.log(1);}",false),("import {alloc,length} from 'std:bytes';function main(){console.log(length(unwrap(alloc(1))));}",false),("import {readFile} from 'std:fs';function main(){readFile('input');}",true)]{
   let checked=check_graph(load_project(&Provider(source.into()),"main.tsr").unwrap()).unwrap();
   let rust=checked.emit_rust();
   assert_eq!(application_cache_identity(&rust),dependencies.then_some(DEPENDENCY_RUNTIME_ABI));
   if dependencies {
    let ui=format!("{rust}\nmod tsrust_native_ui {{}}\n");
    assert_eq!(application_cache_identity(&ui).unwrap(),gpui_application_identity(crate::backend_sources::GPUI_LOCK.as_bytes(),&tsrust_compiler::native_dependency_patches::source_identity(crate::backend_sources::GPUI_MANIFEST).unwrap()));
   }
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
    let cargo_target=command.get_args().skip_while(|arg|*arg!="--target-dir").nth(1).map(PathBuf::from).unwrap();
    assert_eq!(cargo_target,cargo_target_path(&prepared.cargo.as_ref().unwrap().target,cfg!(windows)));
    prepared.acquire_target(true).unwrap();let output=command.output().unwrap();assert!(output.status.success(),"{}",prepared.compilation_error(&String::from_utf8_lossy(&output.stderr)));
    if index==2 {let text=fs::read_to_string(temporary.join("runtime.stdout")).unwrap();let fresh=text.lines().filter_map(|line|serde_json::from_str::<serde_json::Value>(line).ok()).any(|row|row["reason"]=="compiler-artifact"&&row["target"]["name"]==name&&row["fresh"]==true);assert!(fresh,"Unchanged A must use Cargo's fresh graph");}
    prepared.finish().unwrap();let bytes=fs::read(prepared.library.as_ref().unwrap()).unwrap();
    if index==0 {first_library=Some(bytes);first_name=Some(name);}else if index==1 {assert_ne!(first_name.as_ref(),Some(&name));assert_ne!(first_library.as_ref(),Some(&bytes));}else{assert_eq!(first_library.as_ref(),Some(&bytes));}
    if index>0 {prepared.publish();}
   }else{assert_eq!(prepared.state,"hit");assert!(prepared.command.is_none());assert!(prepared.cargo.is_none());}
   let binary=temporary.join(format!("program{}",std::env::consts::EXE_SUFFIX));let mut command=crate::backend_command();prepared.link(&mut command).unwrap();let output=command.arg("--edition=2024").arg(&prepared.application).arg("-o").arg(&binary).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));assert!(Command::new(binary).status().unwrap().success());
  }
 }
 #[test] fn stable_library_alias_keeps_cargo_compile_time_environment_invalidation(){
  let directory=crate::TempDirectory::new().unwrap();let application=directory.0.join("program.rs");
  fs::write(&application,"fn main(){println!(\"{}\",tsrust_runtime::tsrust_native_files::witness());}").unwrap();
  let runtime=r#"pub mod tsrust_native_files {pub fn witness()->i64{env!("TSRUST_TEST_CARGO_TRACKED").parse().unwrap()}}"#;
  let cache=directory.0.join("cache");let binary=directory.0.join(format!("program{}",std::env::consts::EXE_SUFFIX));
  for value in [41,42] {
   let temporary=directory.0.join(format!("build-{value}"));fs::create_dir(&temporary).unwrap();
   let mut prepared=Prepared::with_dependencies(application.clone(),runtime,"tracked-env-alias-test",&cache,&temporary,false,false).unwrap();
   assert_eq!(prepared.state,"miss");let mut command=prepared.command.take().unwrap();command.env("TSRUST_TEST_CARGO_TRACKED",value.to_string());
   prepared.acquire_target(true).unwrap();let output=command.output().unwrap();assert!(output.status.success(),"{}",prepared.compilation_error(&String::from_utf8_lossy(&output.stderr)));
   prepared.finish().unwrap();let mut command=crate::backend_command();prepared.link(&mut command).unwrap();
   let output=command.arg("--edition=2024").arg(&application).arg("-o").arg(&binary).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
   let output=Command::new(&binary).output().unwrap();assert!(output.status.success());assert_eq!(String::from_utf8(output.stdout).unwrap(),format!("{value}\n"));
   // Child-only test overrides must not be published under the parent environment key.
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

#[cfg(test)] mod staged_dependency_tests {
 use super::*;
 #[test] fn cached_dependency_bundles_unpack_once_and_are_then_reused_in_place(){
  let directory=crate::TempDirectory::new().unwrap();let root=directory.0.as_path();
  let built=root.join("built");fs::create_dir(&built).unwrap();
  fs::write(built.join("libtsrust_runtime.rlib"),b"runtime").unwrap();fs::write(built.join("libdependency.rlib"),b"dependency").unwrap();
  let bundle=root.join("bundle");crate::native_dependency_bundle::pack(&built.join("libtsrust_runtime.rlib"),&built,&bundle).unwrap();
  let cache=NativeCache::runtime(&root.join("runtime"),&[b"staged-test"]);cache.publish(&bundle).unwrap();
  let first=staged_dependencies(root,&cache,&root.join("copy-1"),false).expect("first build stages the bundle");
  assert_eq!(fs::read(first.join("libdependency.rlib")).unwrap(),b"dependency");
  assert!(root.join("copy-1").exists());
  let second=staged_dependencies(root,&cache,&root.join("copy-2"),false).expect("second build reuses the staged directory");
  assert_eq!(first,second);
  assert!(!root.join("copy-2").exists(),"a staged hit must not copy the cached bundle again");
  assert!(fs::read_dir(root.join(STAGED_DIRECTORY)).unwrap().flatten().all(|entry|!entry.file_name().to_string_lossy().contains(".pending-")));
 }
}
