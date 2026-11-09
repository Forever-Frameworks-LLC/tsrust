mod support;
use tsrust_compiler::project::{SourceId,SourceProvider,check_graph,load_project};
struct Provider(String);
impl SourceProvider for Provider {
    fn resolve(&self,_:Option<&SourceId>,path:&str)->Result<SourceId,String>{Ok(SourceId(path.into()))}
    fn read(&self,id:&SourceId)->Result<String,String>{if id.0=="main.tsr"{Ok(self.0.clone())}else{Err(format!("Missing {}",id.0))}}
}
fn checked(source:&str)->tsrust_compiler::project::ProjectAnalysis {
    check_graph(load_project(&Provider(support::format_round_trip(source)),"main.tsr").unwrap()).unwrap()
}

#[test]
fn temporary_runtime_is_selected_independently_of_ordinary_files() {
    let plain=checked("import {readFile} from 'std:fs';function main(){readFile('input');}").emit_rust();
    assert!(!plain.contains("tempfile::"));assert!(!plain.contains("mod tsrust_native_tempfiles"));
    assert!(!tsrust_compiler::native_runtime::requires_temporary_files(&plain));
    let temporary=checked("import {createTempDir} from 'std:fs';function main(){createTempDir();}").emit_rust();
    assert!(temporary.contains("mod tsrust_native_tempfiles"));assert!(temporary.contains("tempfile::Builder::new()"));assert!(!temporary.contains("crate::stdlib::fs::runtime"));assert!(!temporary.contains("unsafe"));
    assert!(tsrust_compiler::native_runtime::requires_temporary_files(&temporary));
    let units=tsrust_compiler::native_runtime::separate(&temporary);assert!(units.dependencies);
    let runtime=units.runtime.unwrap();assert!(runtime.contains("pub mod tsrust_native_tempfiles"));
    assert!(tsrust_compiler::native_runtime::requires_temporary_files(&runtime));
    assert!(!units.application.contains("tempfile::"));assert!(!runtime.contains("crate::stdlib::fs::runtime"));
}

#[test]
fn temporary_directories_are_kept_until_caller_removal_and_optional_inputs_work() {
    let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {createTempDir,stat,removeDirAll} from "std:fs";
import {basename} from "std:path";
import {strictEqual} from "std:testing";
function main(){
  const parent=unwrap(createTempDir(undefined,"tsrust-parent-"));
  const first=unwrap(createTempDir(parent,"é-🦀-"));const second=unwrap(createTempDir(parent,""));
  strictEqual(first===second,false);strictEqual(unwrap(stat(first)).isDirectory,true);strictEqual(unwrap(stat(second)).isDirectory,true);
  strictEqual(basename(first).startsWith("é-🦀-"),true);strictEqual(basename(second).length,6);
  const defaults=unwrap(createTempDir(parent));strictEqual(basename(defaults).startsWith(".tmp"),true);
  match(createTempDir("missing-parent","../escape")){Ok(_)=>{throw "invalid prefix";},Err(error)=>{strictEqual(error.code,"invalid-input");strictEqual(error.operation,"createTempDir");strictEqual(error.osCode,undefined);}}
  unwrap(removeDirAll(parent));match(stat(first)){Ok(_)=>{throw "cleanup failed";},Err(error)=>{strictEqual(error.code,"not-found");}}
}
"#);let path=directory.source(&source);support::success(&support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),120));
}

#[cfg(target_os="linux")]
#[test]
fn non_unicode_absolute_temp_path_fails_before_keep_and_attempts_cleanup() {
    use std::os::unix::ffi::OsStringExt;
    let directory=support::TempDirectory::new();let odd=directory.0.join(std::ffi::OsString::from_vec(vec![0xff]));std::fs::create_dir(&odd).unwrap();
    let source=support::format_round_trip("import {createTempDir} from 'std:fs';import {strictEqual} from 'std:testing';function main(){match(createTempDir('.','temp-')){Ok(_)=>{throw 'expected path error';},Err(error)=>{strictEqual(error.code,'not-unicode');strictEqual(error.operation,'createTempDir');}}}");
    let path=directory.source(&source);let binary=directory.0.join("temp-directory");support::success(&support::run(support::cli().arg("build").arg(path).arg("-o").arg(&binary),120));
    support::success(&support::run(std::process::Command::new(&binary).current_dir(&odd),30));assert_eq!(std::fs::read_dir(&odd).unwrap().count(),0);
}

#[test]
fn temporary_directory_package_builds_safely_offline_after_relocation() {
    let directory=support::TempDirectory::new();
    let source=support::format_round_trip("import {createTempDir,stat,removeDirAll} from 'std:fs';import {strictEqual} from 'std:testing';function main(){const parent=unwrap(createTempDir());strictEqual(unwrap(stat(parent)).isDirectory,true);const child=unwrap(createTempDir(parent,'native-'));strictEqual(unwrap(stat(child)).isDirectory,true);unwrap(removeDirAll(parent));match(stat(child)){Ok(_)=>{throw 'cleanup failed';},Err(error)=>{strictEqual(error.code,'not-found');}}}");
    let input=directory.source(&source);let package=directory.0.join("package");
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
    support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),60));
    std::fs::remove_file(input).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
    let main=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&main).unwrap();assert!(!emitted.contains("unsafe"));std::fs::write(main,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
    let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();assert!(manifest.contains("\"temporary-files\""));
    let backend=relocated.join("vendor/tempfile-3.27.0");assert!(backend.join("LICENSE-MIT").is_file());assert!(backend.join("LICENSE-APACHE").is_file());
    let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    for release in [false,true] {
        let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",directory.0.join("empty-cargo-home")).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1").env("CARGO_TARGET_DIR",&target);
        if release {command.arg("--release");}support::success(&support::run(&mut command,120));
        let program=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));let output=support::run(std::process::Command::new(program).current_dir(&relocated),30);support::success(&output);assert!(output.stdout.is_empty());
    }
}
