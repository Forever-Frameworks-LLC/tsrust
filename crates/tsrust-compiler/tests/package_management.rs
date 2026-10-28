mod support;
use std::{fs,path::Path};
fn package(root:&Path,command:&str,args:&[&str])->std::process::Output{
 let mut cli=support::cli();cli.arg(command).arg("--project").arg(root).arg("--offline").args(args);support::run(&mut cli,180)
}
#[test]
fn package_changes_validate_tests_and_preserve_failed_or_preview_generations(){
 let dir=support::TempDirectory::new();
 fs::write(dir.0.join("native.rs"),"pub fn answer()->i64{42}").unwrap();
 fs::write(dir.0.join("main.tsr"),support::format_round_trip("import {answer} from './native.bindings.json';function main(){console.log(answer());}")).unwrap();
 fs::write(dir.0.join("answer.test.tsr"),support::format_round_trip("import {test,strictEqual} from 'std:testing';import {answer} from './native.bindings.json';test('answer',()=>{strictEqual(answer(),42);});")).unwrap();
 // proc-macro2 is already in the pinned compiler toolchain's Cargo cache.
 let output=package(&dir.0,"add",&["proc-macro2@1.0.107"]);support::success(&output);
 let manifest=dir.0.join("native.bindings.json");let accepted=fs::read(&manifest).unwrap();let value:serde_json::Value=serde_json::from_slice(&accepted).unwrap();assert_eq!(value["schemaVersion"],2);assert!(value["lockfile"].as_str().unwrap().contains("proc-macro2"));assert!(!dir.0.join("native.bindings.lock").exists());
 assert!(!dir.0.join("target/tsrust-native/bindings").exists(),"ordinary package changes must not vendor sources");
 assert!(!value["lockfile"].as_str().unwrap().contains("name = \"vello\""),"new binding graphs must exclude retained UI dependencies");
 let output=package(&dir.0,"features",&["proc-macro2","--features","span-locations","--dry-run"]);support::success(&output);assert_eq!(fs::read(&manifest).unwrap(),accepted);
 assert!(!dir.0.join("target/tsrust-native/bindings").exists(),"dry runs must not create vendor generations");
 fs::write(dir.0.join("answer.test.tsr"),"import {test,strictEqual} from 'std:testing';test('fails',()=>{strictEqual(1,2);});").unwrap();
 let output=package(&dir.0,"features",&["proc-macro2","--enable","span-locations"]);assert!(!output.status.success());assert_eq!(fs::read(&manifest).unwrap(),accepted);
 fs::remove_file(dir.0.join("answer.test.tsr")).unwrap();
 let output=package(&dir.0,"features",&["proc-macro2","--enable","span-locations","--no-default-features"]);support::success(&output);
 let value:serde_json::Value=serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();assert_eq!(value["dependencies"]["proc-macro2"]["features"],serde_json::json!(["span-locations"]));assert_eq!(value["dependencies"]["proc-macro2"]["defaultFeatures"],false);
 support::success(&package(&dir.0,"features",&["proc-macro2","--disable","span-locations","--default-features"]));
 support::success(&package(&dir.0,"packages",&[]));
 let before=fs::read(&manifest).unwrap();assert!(!package(&dir.0,"update",&["proc-macro2","--to","999.0.0"]).status.success());assert_eq!(fs::read(&manifest).unwrap(),before);
 fs::write(dir.0.join("native.rs"),"pub fn answer()->i64{missing_function()}").unwrap();
 assert!(!package(&dir.0,"update",&["proc-macro2","--to","1.0.107"]).status.success());assert_eq!(fs::read(&manifest).unwrap(),before);
}
#[test]
fn reserved_dependencies_fail_without_creating_project_configuration(){
 let dir=support::TempDirectory::new();for dependency in ["serde@1.0.229","tokio@1.53.3","rusqlite@0.40.2","libsqlite3-sys@0.38.2","bigdecimal@0.4.11","chrono@0.4.45","chrono-tz@0.10.4","uuid@1.28.0","getrandom@0.4.3"]{assert!(!package(&dir.0,"add",&[dependency]).status.success());}assert!(!dir.0.join("native.bindings.json").exists());assert!(!dir.0.join("native.rs").exists());
}
#[test]
fn curated_regex_add_requires_no_json_and_real_bindings_execute_in_tests(){
 let dir=support::TempDirectory::new();
 fs::write(dir.0.join("regex.test.tsr"),support::format_round_trip("import {test,strictEqual} from 'std:testing';import {Search,SearchResultFound,SearchResultMissing,search} from './native.bindings.json';test('real regex',()=>{const request:Search={pattern:'[0-9]+',text:'Order 314 ready'};match(unwrap(search(request))){SearchResultFound(hit)=>strictEqual(hit.value.text,'314'),SearchResultMissing(_)=>{throw 'no match';}}});")).unwrap();
 let output=package(&dir.0,"add",&["regex@1.12.2"]);support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("real regex"));
 let source=fs::read(dir.0.join("native.rs")).unwrap();assert!(String::from_utf8_lossy(&source).contains("regex::Regex::new"));
 let manifest=dir.0.join("native.bindings.json");let before=fs::read(&manifest).unwrap();
 // Removing a used crate must fail in the native adapter and leave the generation intact.
 assert!(!package(&dir.0,"remove",&["regex"]).status.success());assert_eq!(fs::read(&manifest).unwrap(),before);assert_eq!(fs::read(dir.0.join("native.rs")).unwrap(),source);
}
#[test]
fn explicit_cached_version_updates_are_pinned_and_custom_sources_survive(){
 let dir=support::TempDirectory::new();fs::write(dir.0.join("native.rs"),"pub fn valid()->bool{unicode_ident::is_xid_start('a')}").unwrap();
 support::success(&package(&dir.0,"add",&["unicode-ident@1.0.26"]));
 support::success(&package(&dir.0,"update",&["unicode-ident","--to","1.0.27"]));
 let config:serde_json::Value=serde_json::from_slice(&fs::read(dir.0.join("native.bindings.json")).unwrap()).unwrap();assert_eq!(config["dependencies"]["unicode-ident"]["version"],"=1.0.27");
 assert_eq!(fs::read_to_string(dir.0.join("native.rs")).unwrap(),"pub fn valid()->bool{unicode_ident::is_xid_start('a')}");
}
#[test]
fn concurrent_feature_changes_serialize_without_losing_updates(){
 let dir=support::TempDirectory::new();fs::write(dir.0.join("native.rs"),"pub fn answer()->i64{42}").unwrap();support::success(&package(&dir.0,"add",&["proc-macro2@1.0.107"]));
 let mut first=support::cli().args(["features","proc-macro2","--enable","span-locations","--offline","--project"]).arg(&dir.0).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn().unwrap();
 let mut second=support::cli().args(["features","proc-macro2","--no-default-features","--offline","--project"]).arg(&dir.0).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn().unwrap();
 assert!(first.wait().unwrap().success());assert!(second.wait().unwrap().success());let value:serde_json::Value=serde_json::from_slice(&fs::read(dir.0.join("native.bindings.json")).unwrap()).unwrap();assert_eq!(value["dependencies"]["proc-macro2"]["features"],serde_json::json!(["span-locations"]));assert_eq!(value["dependencies"]["proc-macro2"]["defaultFeatures"],false);
}
#[test]
fn latest_add_repeat_add_caller_failure_and_unused_removal_keep_contracts(){
 let dir=support::TempDirectory::new();fs::write(dir.0.join("native.rs"),"pub fn answer()->i64{42}").unwrap();
 support::success(&package(&dir.0,"add",&["unicode-ident"]));let path=dir.0.join("native.bindings.json");let first:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();assert!(first["dependencies"]["unicode-ident"]["version"].as_str().unwrap().starts_with('='));
 support::success(&package(&dir.0,"add",&["unicode-ident"]));assert_eq!(serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap(),first);
 let before=fs::read(&path).unwrap();fs::write(dir.0.join("main.tsr"),"import {answer} from './native.bindings.json';function main(){console.log(answer().toUpperCase());}").unwrap();assert!(!package(&dir.0,"update",&["unicode-ident"]).status.success());assert_eq!(fs::read(&path).unwrap(),before);
 fs::remove_file(dir.0.join("main.tsr")).unwrap();support::success(&package(&dir.0,"remove",&["unicode-ident"]));let value:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();assert_eq!(value["dependencies"],serde_json::json!({}));
}
#[test]
fn explicit_project_does_not_consult_an_unrelated_broken_current_project(){
 let broken=support::TempDirectory::new();fs::write(broken.0.join("tsrust.config.json"),"{").unwrap();
 let project=support::TempDirectory::new();fs::write(project.0.join("native.rs"),"pub fn answer()->i64{42}").unwrap();
 let output=support::run(support::cli().args(["add","unicode-ident@1.0.26","--offline","--project"]).arg(&project.0).current_dir(&broken.0),180);support::success(&output);assert!(project.0.join("native.bindings.json").exists());
}
