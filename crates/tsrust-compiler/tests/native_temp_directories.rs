mod support;

#[test]
fn temporary_runtime_is_selected_independently_of_ordinary_files() {
    let plain=tsrust_compiler::check("import {readFile} from 'std:fs';function main(){readFile('input');}").unwrap().emit_rust();
    assert!(!plain.contains("tempfile::"));assert!(!plain.contains("mod tsrust_native_tempfiles"));
    assert!(!tsrust_compiler::native_runtime::requires_temporary_files(&plain));
    let temporary=tsrust_compiler::check("import {createTempDir} from 'std:fs';function main(){createTempDir();}").unwrap().emit_rust();
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

#[cfg(unix)]
#[test]
fn non_unicode_absolute_temp_path_fails_before_keep_and_attempts_cleanup() {
    use std::os::unix::ffi::OsStringExt;
    let directory=support::TempDirectory::new();let odd=directory.0.join(std::ffi::OsString::from_vec(vec![0xff]));std::fs::create_dir(&odd).unwrap();
    let source=support::format_round_trip("import {createTempDir} from 'std:fs';import {strictEqual} from 'std:testing';function main(){match(createTempDir('.','temp-')){Ok(_)=>{throw 'expected path error';},Err(error)=>{strictEqual(error.code,'not-unicode');strictEqual(error.operation,'createTempDir');}}}");
    let path=directory.source(&source);let binary=directory.0.join("temp-directory");support::success(&support::run(support::cli().arg("build").arg(path).arg("-o").arg(&binary),120));
    support::success(&support::run(std::process::Command::new(&binary).current_dir(&odd),30));assert_eq!(std::fs::read_dir(&odd).unwrap().count(),0);
}
