mod support;
#[test]
fn attribution_migration_is_offline_readable_and_preserves_notices_in_exported_packages() {
 use std::{fs,path::Path};
 let dir=support::TempDirectory::new();
 let output=support::run(support::cli().args(["ui","add","kbd","separator","--target","web","--offline"]).current_dir(&dir.0),30);support::success(&output);
 let lock_path=dir.0.join("ui-components.lock.json");let mut lock:serde_json::Value=serde_json::from_slice(&fs::read(&lock_path).unwrap()).unwrap();
 let shadcn=fs::read(dir.0.join("attributions/shadcn-MIT.txt")).unwrap();let component=dir.0.join("src/components/ui/kbd.tsrx");
 let original=fs::read_to_string(&component).unwrap();let body=original.split_once('\n').unwrap().1.to_owned()+"\n// Owner edit is retained.\n";
 fs::write(&component,format!("/*\n{}*/\n{body}",String::from_utf8_lossy(&shadcn))).unwrap();
 for entry in lock["installed"].as_array_mut().unwrap(){let path=entry["attribution"]["path"].as_str().unwrap();let notice=fs::read_to_string(dir.0.join(path)).unwrap();entry.as_object_mut().unwrap().remove("attribution");entry["licenseText"]=notice.into();}
 fs::write(&lock_path,serde_json::to_vec_pretty(&lock).unwrap()).unwrap();fs::remove_dir_all(dir.0.join("attributions")).unwrap();let previous=fs::read(&component).unwrap();let previous_lock=fs::read(&lock_path).unwrap();
 let output=support::run(support::cli().args(["ui","attributions","--dry-run","--json"]).current_dir(&dir.0),20);support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("shared license notice"));assert_eq!(fs::read(&component).unwrap(),previous);assert_eq!(fs::read(&lock_path).unwrap(),previous_lock);assert!(!dir.0.join("attributions").exists());
 let output=support::run(support::cli().args(["ui","attributions","--offline"]).current_dir(&dir.0),20);support::success(&output);let updated=fs::read_to_string(&component).unwrap();assert_eq!(updated.split_once('\n').unwrap().1,body);assert_eq!(fs::read(dir.0.join("attributions/shadcn-MIT.txt")).unwrap(),shadcn);
 let lock=fs::read(&lock_path).unwrap();let output=support::run(support::cli().args(["ui","attributions"]).current_dir(&dir.0),20);support::success(&output);assert_eq!(fs::read(&lock_path).unwrap(),lock);
 let package=dir.0.join("exported");let output=support::run(support::cli().args(["emit","src/main.tsr","--package","-o"]).arg(&package).current_dir(&dir.0),30);support::success(&output);assert_eq!(fs::read(package.join("attributions/shadcn-MIT.txt")).unwrap(),shadcn);assert!(package.join(Path::new("attributions/tsrust-MIT.txt")).is_file());
}
