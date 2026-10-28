mod support;
use std::fs;

#[test]
fn unprepared_exports_and_failed_setup_preserve_owner_files_and_leave_no_partial_pack() {
    let directory=support::TempDirectory::new();let packs=directory.0.join("packs");
    let source=directory.0.join("main.tsr");let output=directory.0.join("export");
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    let mut export=support::cli();export.current_dir(&directory.0).arg("emit").arg(&source).args(["--package","-o"]).arg(&output)
        .env("TSRUST_SOURCE_PACK_DIR",&packs).env("PATH",directory.0.join("no-tools")).env("RUSTUP_HOME",directory.0.join("no-rustup"));
    let failed=support::run(&mut export,20);assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("tsrust setup --portable"));assert!(!output.exists());
    fs::create_dir(&output).unwrap();fs::write(output.join("owner"),"keep").unwrap();
    assert!(!support::run(&mut export,20).status.success());assert_eq!(fs::read(output.join("owner")).unwrap(),b"keep");
    let failed=support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"])
        .env("TSRUST_SOURCE_PACK_DIR",&packs).env("CARGO_HOME",directory.0.join("empty-cargo")),40);
    assert!(!failed.status.success());assert!(String::from_utf8_lossy(&failed.stderr).contains("Portable source preparation failed"));
    assert_eq!(fs::read_dir(packs).unwrap().count(),0,"failed staging must be removed");
}

#[test]
fn prepared_pack_metadata_tampering_is_rejected_without_overwrite() {
    let directory=support::TempDirectory::new();let packs=directory.0.join("packs");
    let mut setup=support::cli();setup.current_dir(&directory.0).args(["setup","--portable","--offline"]).env("TSRUST_SOURCE_PACK_DIR",&packs);
    support::success(&support::run(&mut setup,120));
    let pack=fs::read_dir(&packs).unwrap().next().unwrap().unwrap().path();
    let license=pack.join("vendor/serde-1.0.229/LICENSE-MIT");let original=fs::read(&license).unwrap();
    fs::write(&license,b"changed vendor bytes").unwrap();
    let failed=support::run(&mut setup,30);assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("Vendor checksum changed"));
    assert_eq!(fs::read(&license).unwrap(),b"changed vendor bytes");fs::write(&license,original).unwrap();
    let manifest=pack.join("Cargo.toml");fs::write(&manifest,"owner changed metadata").unwrap();
    let failed=support::run(&mut setup,30);assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("metadata changed"));
    assert_eq!(fs::read(&manifest).unwrap(),b"owner changed metadata");
    let source=directory.0.join("main.tsr");let output=directory.0.join("export");
    fs::write(&source,support::format_round_trip("import {fromInt} from 'std:json';function main(){console.log(JSON.stringify(fromInt(7)));}")).unwrap();
    let failed=support::run(support::cli().current_dir(&directory.0).arg("emit").arg(source).args(["--package","-o"]).arg(&output)
        .env("TSRUST_SOURCE_PACK_DIR",&packs).env("PATH",directory.0.join("no-tools")),30);
    assert!(!failed.status.success());assert!(!output.exists());
}
