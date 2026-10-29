//! Frozen legacy UI coverage; runs only in legacy-ui builds.
#![cfg(feature = "legacy-ui")]
mod support;
use std::{fs,path::Path};

#[test]
fn empty_installs_current_source_and_executes_the_formatted_paired_gallery(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 let installed=support::run(support::cli().args(["ui","add","empty","--offline"]).current_dir(&root),30);support::success(&installed);
 for relative in ["src/examples/empty-states.tsrx","src/acceptance/empty-states.tsr","src/acceptance/empty-composition.tsrx","src/components/ui/empty.tsrx"]{let source=fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let output=support::run(support::cli().arg("run").arg(root.join("src/acceptance/empty-states.tsr")),180);support::success(&output);assert_eq!(output.stdout,b"Empty component passed\n");
 let directory=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","empty","--offline","--target","native","--target","web"]).current_dir(&directory.0),30));
 let payload=fs::read_to_string(directory.0.join("src/components/ui/empty.tsrx")).unwrap();let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/empty-1.0.0.json")).unwrap();assert!(payload.ends_with(catalog[0]["files"][0]["content"].as_str().unwrap()));
 let repeated=support::run(support::cli().args(["ui","add","empty","--offline"]).current_dir(&directory.0),30);support::success(&repeated);assert_eq!(fs::read_to_string(directory.0.join("src/components/ui/empty.tsrx")).unwrap(),payload);
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let example=&catalog["std:ui"].as_array().unwrap().iter().find(|entry|entry["name"]=="ThemeTypography").unwrap()["examples"][0];let path=directory.0.join("main.tsr");fs::write(&path,support::format_round_trip(example["source"].as_str().unwrap())).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes());
}
