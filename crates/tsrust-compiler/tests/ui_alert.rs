mod support;
use std::{fs,path::Path};

#[test]
fn alert_offline_installation_requires_its_compiler_contract_and_preserves_local_edits(){
 let directory=support::TempDirectory::new();let add=||support::run(support::cli().args(["ui","add","alert","--offline","--target","web","--target","native"]).current_dir(&directory.0),30);support::success(&add());
 let path=directory.0.join("src/components/ui/alert.tsrx");let source=fs::read_to_string(&path).unwrap();assert!(source.starts_with("// License: MIT. See ../../../attributions/shadcn-MIT.txt."));for name in ["Alert","AlertTitle","AlertDescription","AlertAction"]{assert!(source.contains(&format!("function {name}(")));}
 let lock:serde_json::Value=serde_json::from_slice(&fs::read(directory.0.join("ui-components.lock.json")).unwrap()).unwrap();let alert=lock["installed"].as_array().unwrap().iter().find(|entry|entry["identity"]["name"]=="alert").unwrap();assert_eq!(alert["compilerApi"],3);assert_eq!(alert["attribution"]["path"],"attributions/shadcn-MIT.txt");
 let edited=format!("{source}\n// Project-owned edit.\n");fs::write(&path,&edited).unwrap();support::success(&add());assert_eq!(fs::read_to_string(&path).unwrap(),edited);
 let entry=directory.0.join("main.tsrx");for (code,expected)in[("import {Alert} from './src/components/ui/alert';function main(){const node=<Alert variant=\"warning\"/>;}","type"),("import {View} from 'std:ui';function main(){const node=<View dataVariant=\"default\"/>;}","Unknown JSX attribute")]{fs::write(&entry,support::format_round_trip(code)).unwrap();let result=support::run(support::cli().arg("check").arg(&entry),30);assert!(!result.status.success());assert!(String::from_utf8_lossy(&result.stderr).contains(expected),"{}",String::from_utf8_lossy(&result.stderr));}
}

#[test]
fn alert_composition_metadata_and_positioning_execute_after_formatting(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");let entry=root.join("src/acceptance/alerts.tsrx");let source=fs::read_to_string(&entry).unwrap();assert_eq!(source,support::format_round_trip(&source));let result=support::run(support::cli().arg("run").arg(entry),180);support::success(&result);assert_eq!(result.stdout,b"Alert composition passed\n");
}
