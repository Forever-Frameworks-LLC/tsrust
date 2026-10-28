mod support;
use std::fs;

#[test]
fn nested_and_new_optional_cargo_configuration_cannot_reuse_a_stale_executable() {
    let directory=support::TempDirectory::new();let cargo=directory.0.join(".cargo");fs::create_dir(&cargo).unwrap();
    let source=directory.0.join("main.tsr");let binary=directory.0.join(format!("program{}",std::env::consts::EXE_SUFFIX));
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    fs::write(cargo.join("config.toml"),"include = ['flags.toml', { path = 'optional.toml', optional = true }]\n").unwrap();
    fs::write(cargo.join("flags.toml"),"[[include]]\npath = 'deeper.toml'\n").unwrap();
    fs::write(cargo.join("deeper.toml"),"[build]\nrustflags = []\n").unwrap();
    let build=||support::run(support::cli().current_dir(&directory.0).arg("build").arg(&source)
        .arg("-o").arg(&binary).env("TSRUST_CACHE_DIR",directory.0.join("native"))
        .env("TSRUST_BUILD_TIMINGS","1"),180);
    support::success(&build());
    let cached=build();support::success(&cached);assert!(String::from_utf8_lossy(&cached.stderr).contains("cache=hit"));
    let missing=directory.0.join("tsrust-test-wrapper-that-does-not-exist");
    let invalid_wrapper=format!("[build]\nrustc-wrapper = {}\n",serde_json::to_string(&missing.to_string_lossy()).unwrap());
    fs::write(cargo.join("deeper.toml"),&invalid_wrapper).unwrap();
    let changed=build();assert!(!changed.status.success(),"A changed included wrapper must not reuse the old executable");
    fs::write(cargo.join("deeper.toml"),"[build]\nrustflags = []\n").unwrap();
    support::success(&build());
    fs::write(cargo.join("optional.toml"),invalid_wrapper).unwrap();
    let appeared=build();assert!(!appeared.status.success(),"A newly created optional include must invalidate artifacts");
    assert!(binary.is_file(),"Failed reconfiguration must preserve the previous working executable");
}
