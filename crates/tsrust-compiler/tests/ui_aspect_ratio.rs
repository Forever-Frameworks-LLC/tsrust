//! Frozen legacy UI coverage; runs only in legacy-ui builds.
#![cfg(feature = "legacy-ui")]
mod support;
#[test]
fn aspect_ratio_offline_source_and_formatted_native_construction_have_checked_failures(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 let fixture=root.join("src/acceptance/aspect-ratio.tsrx");let original=std::fs::read_to_string(&fixture).unwrap();assert_eq!(original,support::format_round_trip(&original));
 let output=support::run(support::cli().arg("run").arg(&fixture),120);support::success(&output);assert_eq!(output.stdout,b"Aspect ratio construction passed\n");
 let dir=support::TempDirectory::new();let output=support::run(support::cli().args(["ui","add","aspect-ratio","--offline","--target","web","--target","native"]).current_dir(&dir.0),30);support::success(&output);
 let path=dir.0.join("src/components/ui/aspect-ratio.tsrx");let installed=std::fs::read_to_string(&path).unwrap();assert!(installed.starts_with("// License: MIT. See ../../../attributions/shadcn-MIT.txt."));assert!(dir.0.join("attributions/shadcn-MIT.txt").is_file());
 let expected=std::fs::read_to_string(root.join("src/components/ui/aspect-ratio.tsrx")).unwrap();assert_eq!(installed.split_once("import ").unwrap().1,expected.split_once("import ").unwrap().1);
 let lock=std::fs::read(dir.0.join("ui-components.lock.json")).unwrap();let output=support::run(support::cli().args(["ui","add","aspect-ratio","--offline"]).current_dir(&dir.0),30);support::success(&output);assert_eq!(std::fs::read(dir.0.join("ui-components.lock.json")).unwrap(),lock);
}
