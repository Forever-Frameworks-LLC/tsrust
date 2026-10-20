mod support;
use std::{fs,path::Path};
#[test]
fn offline_badge_source_and_formatted_composition_execute(){
 let directory=support::TempDirectory::new();
 support::success(&support::run(support::cli().args(["ui","add","badge","--offline","--target","native"]).current_dir(&directory.0),30));
 let source=fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/src/acceptance/badge.tsrx")).unwrap().replace("../components/ui/badge","./src/components/ui/badge");
 let path=directory.0.join("main.tsrx");fs::write(&path,support::format_round_trip(&source)).unwrap();
 let output=support::run(support::cli().arg("run").arg(path).current_dir(&directory.0),180);support::success(&output);assert_eq!(output.stdout,b"Badge composition passed\n");
 let installed=directory.0.join("src/components/ui/badge.tsrx");let before=fs::read(&installed).unwrap();support::success(&support::run(support::cli().args(["ui","add","badge","--offline"]).current_dir(&directory.0),30));assert_eq!(fs::read(installed).unwrap(),before);
}
