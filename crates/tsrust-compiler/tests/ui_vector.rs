mod support;
use std::{fs,path::Path};
#[test]
fn formatted_vector_markup_contracts_and_failures_execute(){
 let directory=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","tokens","--offline","--target","native"]).current_dir(&directory.0),30));
 let source=fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/src/acceptance/vector-graphics.tsrx")).unwrap();let path=directory.0.join("main.tsrx");fs::write(&path,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert_eq!(output.stdout,b"Vector graphics passed\n");
}
#[test]
fn documented_vector_example_executes_with_the_shown_output(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let entry=catalog["std:ui"].as_array().unwrap().iter().find(|entry|entry["name"]=="Svg").unwrap();let example=&entry["examples"][0];let directory=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","tokens","--offline","--target","native"]).current_dir(&directory.0),30));let path=directory.0.join("main.tsrx");fs::write(&path,support::format_round_trip(example["source"].as_str().unwrap())).unwrap();let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes());
}
