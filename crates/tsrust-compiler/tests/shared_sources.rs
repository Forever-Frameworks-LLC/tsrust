mod support;
use std::{fs, path::Path, process::Command};

fn workspaces(root: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(root.join("runtime/build-v1")).unwrap().map(|entry|entry.unwrap().path())
        .filter(|path|path.is_dir()).collect()
}

#[test]
fn missing_shared_sources_fail_offline_and_emit_remains_toolchain_free() {
    let directory=support::TempDirectory::new();
    let home=directory.0.join("empty-cargo");fs::create_dir(&home).unwrap();
    let cache=directory.0.join("native");
    let source=directory.0.join("main.tsr");
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    let setup=support::run(support::cli().current_dir(&directory.0)
        .args(["setup","--dependencies","--offline"]).env("CARGO_HOME",&home).env("TSRUST_CACHE_DIR",&cache),40);
    assert!(!setup.status.success());
    assert!(String::from_utf8_lossy(&setup.stderr).contains("source preparation failed"),"{}",String::from_utf8_lossy(&setup.stderr));
    let failed=support::run(support::cli().current_dir(&directory.0).arg("build").arg(&source)
        .arg("-o").arg(directory.0.join("program")).env("CARGO_HOME",&home).env("TSRUST_CACHE_DIR",&cache),40);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("tsrust setup --dependencies --yes"));
    let paths=workspaces(&cache);assert_eq!(paths.len(),1);
    let package=paths[0].join("native-support");
    assert!(!package.join("vendor").exists());assert!(!paths[0].join("cargo-home").exists());
    assert_eq!(fs::read(package.join("Cargo.lock")).unwrap(),include_bytes!("../backend-dependencies/Cargo.lock"));
    assert!(!fs::read_to_string(package.join(".cargo/config.toml")).unwrap().contains("replace-with"));
    let rust=directory.0.join("program.rs");
    let emitted=support::run(support::cli().current_dir(&directory.0).arg("emit").arg(&source)
        .arg("-o").arg(&rust).env("PATH",directory.0.join("no-tools"))
        .env("RUSTUP_HOME",directory.0.join("no-rustup")).env("CARGO_HOME",&home),20);
    support::success(&emitted);assert!(rust.is_file());
}

#[test]
fn shared_backend_builds_reuse_targets_and_invalidate_changed_cargo_configuration() {
    let directory=support::TempDirectory::new();let cache=directory.0.join("native");
    let source=directory.0.join("main.tsr");let binary=directory.0.join(format!("program{}",std::env::consts::EXE_SUFFIX));
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    let build=|| {
        let output=support::run(support::cli().current_dir(&directory.0).arg("build").arg(&source)
            .arg("-o").arg(&binary).env("TSRUST_CACHE_DIR",&cache).env("TSRUST_BUILD_TIMINGS","1"),180);
        support::success(&output);String::from_utf8_lossy(&output.stderr).into_owned()
    };
    assert!(build().contains("cache=miss"));assert!(build().contains("cache=hit"));
    let first=workspaces(&cache);assert_eq!(first.len(),1);
    assert!(!first[0].join("native-support/vendor").exists());assert!(!first[0].join("cargo-home").exists());
    fs::create_dir(directory.0.join(".cargo")).unwrap();
    fs::write(directory.0.join(".cargo/config.toml"),"[build]\nrustflags = ['--cfg', 'tsrust_cache_provenance_test']\n").unwrap();
    assert!(build().contains("cache=miss"),"Cargo configuration changes must invalidate executable artifacts");
    assert_eq!(workspaces(&cache),first,"Cargo's target stays reusable across configuration revisions");
    assert!(build().contains("cache=hit"));
    let deployed=directory.0.join("runtime-only");fs::create_dir(&deployed).unwrap();
    let executable=deployed.join(binary.file_name().unwrap());fs::copy(&binary,&executable).unwrap();
    let output=support::run(Command::new(executable).current_dir(&deployed).env("PATH",&deployed)
        .env("CARGO_HOME",deployed.join("no-cargo")).env("RUSTUP_HOME",deployed.join("no-rustup")),10);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"7");
}

#[test]
fn portable_export_remains_toolchain_free_and_rebuilds_after_relocation_offline() {
    let directory=support::TempDirectory::new();let source=directory.0.join("main.tsr");
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    let exported=directory.0.join("exported");let relocated=directory.0.join("relocated");
    let home=directory.0.join("empty-cargo");
    // Preparation is explicit and precedes the toolchain-free export. The test
    // runner supplies TSRUST_SOURCE_PACK_DIR so the later empty Cargo home does
    // not also change the location of the prepared portable pack.
    support::success(&support::run(support::cli().current_dir(&directory.0).arg("emit").arg(&source)
        .args(["--package","-o"]).arg(&exported).env("PATH",directory.0.join("no-tools"))
        .env("RUSTUP_HOME",directory.0.join("no-rustup")).env("CARGO_HOME",&home),120));
    assert!(exported.join("vendor").is_dir());
    let metadata:serde_json::Value=serde_json::from_slice(&fs::read(exported.join("tsrust-export.json")).unwrap()).unwrap();
    assert_eq!(metadata["schemaVersion"],1);assert_eq!(metadata["rustToolchain"],tsrust_compiler::RUST_BACKEND_VERSION);
    assert_eq!(metadata["dependencySources"],"vendor");assert!(metadata["buildTarget"].is_null());
    fs::rename(&exported,&relocated).unwrap();
    let output=support::run(Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","run","--offline","--locked"])
        .arg("--manifest-path").arg(relocated.join("Cargo.toml")).arg("--config").arg(relocated.join(".cargo/config.toml"))
        .current_dir(&directory.0).env("CARGO_HOME",&home).env("CARGO_INCREMENTAL","0")
        .env("CARGO_TARGET_DIR",std::env::var_os("CARGO_TARGET_DIR").expect("shared Cargo target")),120);
    support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"7");
}
