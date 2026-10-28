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


#[test]
fn binding_sources_are_shared_and_portable_preparation_is_explicit_and_verified() {
    let directory=support::TempDirectory::new();let root=&directory.0;
    let manifest=root.join("native.bindings.json");let source=root.join("main.tsr");
    fs::write(&manifest,r#"{"schemaVersion":2,"source":"./native.rs","dependencies":{"unicode-ident":{"version":"=1.0.27"}}}"#).unwrap();
    fs::write(root.join("native.rs"),"pub fn valid()->bool{unicode_ident::is_xid_start('λ')}").unwrap();
    fs::write(&source,support::format_round_trip("import {valid} from './native.bindings.json';function main(){console.log(valid());}")).unwrap();
    let packs=root.join("packs");let cache=root.join("native-cache");let empty=root.join("empty-cargo");fs::create_dir(&empty).unwrap();
    let cli=||{let mut command=support::cli();command.current_dir(root).env("TSRUST_SOURCE_PACK_DIR",&packs).env("TSRUST_CACHE_DIR",&cache);command};
    support::success(&support::run(cli().args(["bindings","resolve"]).arg(&manifest).arg("--offline"),90));
    assert!(!packs.exists());assert!(!root.join("target/tsrust-native/bindings").exists());
    let lock=fs::read(root.join("native.bindings.lock")).unwrap();assert!(!String::from_utf8_lossy(&lock).contains("name = \"vello\""));
    for expected in ["cache=miss","cache=hit"] {
        let output=support::run(cli().arg("run").arg(&source).env("TSRUST_BUILD_TIMINGS","1"),120);support::success(&output);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"true");assert!(String::from_utf8_lossy(&output.stderr).contains(expected),"{}",String::from_utf8_lossy(&output.stderr));
    }
    let workspaces=workspaces(&cache);assert_eq!(workspaces.len(),1);
    assert!(!workspaces[0].join("native-support/vendor").exists());assert!(!workspaces[0].join("cargo-home").exists());
    let failed=support::run(cli().arg("run").arg(&source).env("CARGO_HOME",&empty),60);
    assert!(!failed.status.success());assert!(String::from_utf8_lossy(&failed.stderr).contains("tsrust bindings resolve"));
    let emitted=support::run(cli().arg("emit").arg(&source).env("PATH",root.join("no-tools")).env("RUSTUP_HOME",root.join("no-rustup")).env("CARGO_HOME",&empty),30);support::success(&emitted);
    assert!(!String::from_utf8_lossy(&emitted.stdout).contains("\"vendor\""));
    let exported=root.join("export");
    let export=||{support::run(cli().arg("emit").arg(&source).args(["--package","-o"]).arg(&exported).env("PATH",root.join("no-tools")).env("RUSTUP_HOME",root.join("no-rustup")).env("CARGO_HOME",&empty),120)};
    let missing=export();assert!(!missing.status.success());assert!(String::from_utf8_lossy(&missing.stderr).contains("--portable"));assert!(!exported.exists());
    support::success(&support::run(cli().args(["bindings","resolve"]).arg(&manifest).args(["--offline","--portable"]),180));
    assert_eq!(fs::read(root.join("native.bindings.lock")).unwrap(),lock,"portable preparation must preserve the exact lock");
    support::success(&export());assert!(exported.join("vendor/unicode-ident-1.0.27/LICENSE-MIT").is_file());
    let relocated=root.join("relocated");fs::rename(&exported,&relocated).unwrap();
    let output=support::run(Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","run","--offline","--locked"])
        .arg("--manifest-path").arg(relocated.join("Cargo.toml")).arg("--config").arg(relocated.join(".cargo/config.toml"))
        .current_dir(root).env("CARGO_HOME",&empty).env("CARGO_INCREMENTAL","0")
        .env("CARGO_TARGET_DIR",std::env::var_os("CARGO_TARGET_DIR").expect("shared Cargo target")),180);
    support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"true");
    let pack=fs::read_dir(&packs).unwrap().map(|entry|entry.unwrap().path()).find(|path|path.is_dir()).unwrap();
    let license=pack.join("vendor/unicode-ident-1.0.27/LICENSE-MIT");let original=fs::read(&license).unwrap();fs::write(&license,"tampered").unwrap();
    let corrupt=export();assert!(!corrupt.status.success());assert!(String::from_utf8_lossy(&corrupt.stderr).contains("checksum"));assert!(!exported.exists());
    fs::write(license,original).unwrap();
}

#[test]
fn native_build_and_portable_export_link_at_deep_paths() {
    let directory=support::TempDirectory::new();
    let source=directory.0.join("main.tsr");
    fs::write(&source,support::format_round_trip("function main(){console.log(42);}")).unwrap();
    let mut deep=directory.0.join("space and unicode é");
    while deep.as_os_str().len()<270 {deep=deep.join("nested-workspace");}
    fs::create_dir_all(&deep).unwrap();
    let executable=deep.join(format!("direct{}",std::env::consts::EXE_SUFFIX));
    support::success(&support::run(support::cli().arg("build").arg(&source).arg("-o").arg(&executable),120));
    let output=support::run(&mut Command::new(&executable),30);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"42");
    let exported=directory.0.join("exported");
    support::success(&support::run(support::cli().arg("emit").arg(&source).args(["--package","-o"]).arg(&exported),30));
    let target=deep.join("cargo-target");
    support::success(&support::run(Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"])
        .current_dir(&exported).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0"),120));
    let output=support::run(&mut Command::new(target.join(format!("debug/program{}",std::env::consts::EXE_SUFFIX))),30);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout).trim(),"42");
}
