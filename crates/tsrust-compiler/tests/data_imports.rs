mod support;
use std::{fs, process::Command};
#[test]
fn typed_data_import_is_embedded_and_aliases_share_one_owner() {
    let directory=support::TempDirectory::new();
    fs::create_dir_all(directory.0.join("data")).unwrap();
    fs::write(directory.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"data-test","paths":{"@data/*":"./data/*"}}"#).unwrap();
    let data=directory.0.join("data/settings.jsonc");
    fs::write(&data,"{/* config */\"host\":\"local\",\"port\":8080,}").unwrap();
    let entry=directory.0.join("main.tsr");
    fs::write(&entry,support::format_round_trip("import settings from '@data/settings.jsonc'; import same from './data/settings.jsonc'; function main(): void { console.log(settings.host); console.log(same.port); const owned = clone(settings); owned.port = 9; console.log(owned.port); console.log(same.port); }")).unwrap();
    let emitted=support::run(support::cli().arg("emit").arg(&entry),30);support::success(&emitted);
    assert_eq!(String::from_utf8(emitted.stdout).unwrap().matches("static TSR_DATA").count(),1);
    let binary=directory.0.join("program");
    support::success(&support::run(support::cli().arg("build").arg(&entry).arg("-o").arg(&binary),60));
    fs::remove_file(data).unwrap();
    let output=support::run(&mut Command::new(binary),10);support::success(&output);
    assert_eq!(output.stdout,b"local\n8080\n9\n8080\n");
}
