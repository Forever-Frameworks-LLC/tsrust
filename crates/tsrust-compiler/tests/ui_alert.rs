mod support;
use std::{fs,path::Path};

#[test]
fn alert_offline_installation_requires_its_compiler_contract_and_preserves_local_edits(){
 let directory=support::TempDirectory::new();let add=||support::run(support::cli().args(["ui","add","alert","--offline","--target","web","--target","native"]).current_dir(&directory.0),30);support::success(&add());
 let path=directory.0.join("src/components/ui/alert.tsrx");let source=fs::read_to_string(&path).unwrap();assert!(source.starts_with("// License: MIT. See ../../../attributions/shadcn-MIT.txt."));for name in ["Alert","AlertTitle","AlertDescription","AlertAction"]{assert!(source.contains(&format!("function {name}(")));}
 let lock:serde_json::Value=serde_json::from_slice(&fs::read(directory.0.join("ui-components.lock.json")).unwrap()).unwrap();let alert=lock["installed"].as_array().unwrap().iter().find(|entry|entry["identity"]["name"]=="alert").unwrap();assert_eq!(alert["compilerApi"],4);assert_eq!(alert["attribution"]["path"],"attributions/shadcn-MIT.txt");
 let edited=format!("{source}\n// Project-owned edit.\n");fs::write(&path,&edited).unwrap();support::success(&add());assert_eq!(fs::read_to_string(&path).unwrap(),edited);
 let entry=directory.0.join("main.tsrx");for (code,expected)in[("import {Alert} from './src/components/ui/alert';function main(){const node=<Alert variant=\"warning\"/>;}","type"),("import {View} from 'std:ui';function main(){const node=<View dataVariant=\"default\"/>;}","Unknown field \"dataVariant\" on ViewProps")]{fs::write(&entry,support::format_round_trip(code)).unwrap();let result=support::run(support::cli().arg("check").arg(&entry),30);assert!(!result.status.success());assert!(String::from_utf8_lossy(&result.stderr).contains(expected),"{}",String::from_utf8_lossy(&result.stderr));}
}

#[test]
fn alert_composition_metadata_and_positioning_execute_after_formatting(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");let entry=root.join("src/acceptance/alerts.tsrx");let source=fs::read_to_string(&entry).unwrap();assert_eq!(source,support::format_round_trip(&source));let result=support::run(support::cli().arg("run").arg(entry),180);support::success(&result);assert_eq!(result.stdout,b"Alert composition passed\n");
}

#[test]
fn reactive_alert_installation_preserves_prior_source_and_imports(){
 let directory=support::TempDirectory::new();let old=include_str!("../../../docs/src/components/ui/alert.tsrx");fs::create_dir_all(directory.0.join("src/components/ui")).unwrap();fs::write(directory.0.join("src/components/ui/alert.tsrx"),old).unwrap();let application="import {Alert} from './src/components/ui/alert';function main(){const node=<Alert/>;}";fs::write(directory.0.join("main.tsrx"),application).unwrap();
 support::success(&support::run(support::cli().args(["ui","add","alert","--offline","--target","web","--target","native"]).current_dir(&directory.0),30));assert_eq!(fs::read_to_string(directory.0.join("src/components/ui/alert.tsrx")).unwrap(),old);assert_eq!(fs::read_to_string(directory.0.join("main.tsrx")).unwrap(),application);
 let current=fs::read_to_string(directory.0.join("src/components/ui/alert-v2.tsrx")).unwrap();assert!(current.contains("ReactiveValue<'default' | 'destructive'>")&&current.contains("useProp<string>"));
 let source="import {Alert,AlertTitle} from './src/components/ui/alert-v2';import {signal,View} from 'std:ui';function main(){const active=signal(false);const node=<Alert variant={active.get()?'destructive':'default'} render={(props)=><View {...props} as=\"section\"/>}><AlertTitle>Live</AlertTitle></Alert>;}";let path=directory.0.join("reactive.tsrx");fs::write(&path,support::format_round_trip(source)).unwrap();support::success(&support::run(support::cli().arg("check").arg(path),30));
}
