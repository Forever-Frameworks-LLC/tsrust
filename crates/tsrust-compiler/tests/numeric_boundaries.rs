mod support;
#[test]
fn numeric_boundaries_preserve_integer_identity_and_reject_loss(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/numeric-boundaries/checked.tsr"));
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);
 support::success(&output);assert_eq!(output.stdout,b"numeric boundaries exact\n");
}
#[test]
fn numeric_documentation_examples(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 for entry in catalog["std:numeric"].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(example["source"].as_str().unwrap());
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),90);support::success(&output);
 assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes(),"{}",entry["name"]);
 }}
}
