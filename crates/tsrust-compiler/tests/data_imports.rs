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

#[test]
fn data_import_package_builds_offline_after_original_inputs_are_removed() {
    let directory=support::TempDirectory::new();let input=directory.0.join("main.tsr");let data=directory.0.join("settings.json");
    fs::write(&input,support::format_round_trip("import settings from './settings.json';function main():void{console.log(settings.message);}")).unwrap();
    fs::write(&data,r#"{"message":"embedded-package"}"#).unwrap();
    let package=directory.0.join("package");
    support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
    assert!(!package.join("vendor").exists(),"data imports should not link a runtime JSON parser");
    fs::remove_file(input).unwrap();fs::remove_file(data).unwrap();
    let relocated=directory.0.join("relocated");fs::rename(package,&relocated).unwrap();
    let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    support::success(&support::run(Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",directory.0.join("empty-cargo-home")).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0"),60));
    let output=support::run(&mut Command::new(target.join("debug/program")),10);support::success(&output);assert_eq!(output.stdout,b"embedded-package\n");
}

#[test]
fn data_import_scalar_array_and_escaped_property_roots_execute() {
    let directory=support::TempDirectory::new();
    for (name,data) in [("text",r#""雪""#),("integer","42"),("number","-0.5"),("boolean","true"),("absent","null"),("array",r#"[1,"two",null]"#),("keys",r#"{"a-b":7,"na\u006de":"escaped"}"#)] {
        fs::write(directory.0.join(format!("{name}.json")),data).unwrap();
    }
    let entry=directory.0.join("main.tsr");
    fs::write(&entry,support::format_round_trip("import text from './text.json';import integer from './integer.json';import number from './number.json';import boolean from './boolean.json';import absent from './absent.json';import array from './array.json';import keys from './keys.json';function main():void{console.log(text);console.log(integer);console.log(number);console.log(boolean);console.log(absent);for(const item of array){console.log(item);}console.log(keys['a-b']);console.log(keys.name);}")).unwrap();
    let output=support::run(support::cli().arg("run").arg(&entry),60);support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"雪\n42\n-0.5\ntrue\nnull\n1\ntwo\nnull\n7\nescaped\n");
}

#[test]
fn data_import_boundary_test_callbacks_and_cloned_quoted_keys() {
    let directory=support::TempDirectory::new();
    fs::write(directory.0.join("settings.json"),r#"{"a-b":1,"name":"original","values":[1,2]}"#).unwrap();
    let entry=directory.0.join("data.test.tsr");
    fs::write(&entry,support::format_round_trip("import settings from './settings.json';import {test,strictEqual} from 'std:testing';function reader():Function<int>{return ()=>settings['a-b'];}function name():string{return clone(settings.name);}test('immutable imported data',()=>{const owned=clone(settings);owned['a-b']=9;strictEqual(owned['a-b'],9);strictEqual(settings['a-b'],1);const read=reader();strictEqual(read(),1);strictEqual(name(),'original');});")).unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&entry),60);support::success(&output);
    assert!(String::from_utf8(output.stdout).unwrap().contains("1 passed"));
}

#[test]
fn data_import_boundary_rejects_mutations_and_escaping_owned_fields() {
    let directory=support::TempDirectory::new();fs::write(directory.0.join("settings.json"),r#"{"a-b":1,"name":"original","values":[1,2]}"#).unwrap();
    let entry=directory.0.join("main.tsr");
    for body in ["settings['a-b']=9;","settings.values[0]=3;","const aliased=settings.values;aliased.push(3);","const mutate=()=>{settings.values.push(3);};mutate();"] {
        fs::write(&entry,support::format_round_trip(&format!("import settings from './settings.json';function main():void{{{body}}}"))).unwrap();
        let output=support::run(support::cli().arg("check").arg(&entry),20);
        assert!(!output.status.success(),"accepted {body}");
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("data-readonly"),"{body}: {error}");
    }
    fs::write(&entry,support::format_round_trip("import settings from './settings.json';function steal():string{return settings.name;}")).unwrap();
    let output=support::run(support::cli().arg("check").arg(&entry),20);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("data-readonly"));
}
