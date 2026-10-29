//! Frozen legacy UI coverage; runs only in legacy-ui builds.
#![cfg(feature = "legacy-ui")]
mod support;
use std::{fs,path::Path};

#[test]
fn disclosure_gallery_and_standalone_id_documentation_execute_after_formatting(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 for relative in ["src/examples/disclosure-semantics.tsrx","src/acceptance/disclosure-semantics.tsr"]{let source=fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let output=support::run(support::cli().arg("run").arg(root.join("src/acceptance/disclosure-semantics.tsr")),180);support::success(&output);assert_eq!(output.stdout,b"Disclosure semantics passed\n");
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let entry=catalog["std:ui"].as_array().unwrap().iter().find(|entry|entry["name"]=="useId").unwrap();let example=&entry["examples"][0];let directory=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","tokens","--offline","--target","native"]).current_dir(&directory.0),30));let path=directory.0.join("main.tsr");fs::write(&path,support::format_round_trip(example["source"].as_str().unwrap())).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes());
 let path=directory.0.join("failure.tsrx");
 for source in ["import {Button} from 'std:ui';function main(){const node=<Button aria-expanded='yes'/>;}","import {Button} from 'std:ui';function main(){const node=<Button aria-controls={42}/>;}","import {useId} from 'std:ui';function main(){const id=useId(1);}"]{fs::write(&path,source).unwrap();let output=support::run(support::cli().arg("check").arg(&path),30);assert!(!output.status.success(),"{source}");}
}
