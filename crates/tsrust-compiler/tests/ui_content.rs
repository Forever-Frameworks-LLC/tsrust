mod support;
#[test]
fn semantic_content_sources_execute_natively_and_offline_install_as_editable_files(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-content");
 let output=support::run(support::cli().arg("run").arg(root.join("inspect.tsrx")),120);support::success(&output);assert_eq!(output.stdout,b"Semantic content passed\n");
 let dir=support::TempDirectory::new();let output=support::run(support::cli().args(["ui","add","kbd","separator","--target","native","--offline"]).current_dir(&dir.0),30);support::success(&output);
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/semantic-content-1.0.0.json")).unwrap();
 for item in catalog.as_array().unwrap(){let name=item["name"].as_str().unwrap();let source=support::format_round_trip(item["files"][0]["content"].as_str().unwrap());assert_eq!(std::fs::read_to_string(dir.0.join(format!("src/components/ui/{name}.tsrx"))).unwrap(),source);assert_eq!(std::fs::read_to_string(root.join(format!("src/components/ui/{name}.tsrx"))).unwrap(),source);}
 let lock=std::fs::read(dir.0.join("ui-components.lock.json")).unwrap();let output=support::run(support::cli().args(["ui","add","kbd","separator"]).current_dir(&dir.0),30);support::success(&output);assert_eq!(std::fs::read(dir.0.join("ui-components.lock.json")).unwrap(),lock);
}
#[test]
fn separator_children_unknown_orientation_and_unsupported_props_reject(){
 let dir=support::TempDirectory::new();let output=support::run(support::cli().args(["ui","add","kbd","separator","--target","native"]).current_dir(&dir.0),30);support::success(&output);
 for (body,reason) in [("const n=<Separator orientation='diagonal'/>;","Expected"),("const n=<Separator>Invalid</Separator>;","children"),("const n=<Kbd render={()=>{return <View/>;}}>Invalid</Kbd>;","render")] {
  let source=format!("import {{Separator}} from './components/ui/separator';import {{Kbd}} from './components/ui/kbd';import {{View}} from 'std:ui';function main(){{{body}}}");let path=dir.0.join("src/reject.tsrx");std::fs::write(&path,support::format_round_trip(&source)).unwrap();for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(reason),"{}",String::from_utf8_lossy(&output.stderr));}
 }
}
