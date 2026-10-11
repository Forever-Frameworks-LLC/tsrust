mod support;
use tsrust_compiler::project::{FileSourceProvider,SourceId,SourceProvider,load_project,check_graph};
use std::fs;
#[test]
fn file_provider_resolves_aliases_extensions_indexes_and_canonical_identity() {
 let dir=support::TempDirectory::new();fs::create_dir_all(dir.0.join("src/components")).unwrap();
 fs::write(dir.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"example","build":{"entry":"src/main.tsr","output":"dist/example"},"paths":{"@/*":"./src/*"}}"#).unwrap();
 fs::write(dir.0.join("src/components/index.tsr"),support::format_round_trip("export function greeting():string{return 'index';}")).unwrap();
 fs::write(dir.0.join("src/main.tsr"),support::format_round_trip("import {greeting} from '@/components';function main(){console.log(greeting());}")).unwrap();
 let main=dir.0.join("src/main.tsr");let graph=load_project(&FileSourceProvider,main.to_str().unwrap()).unwrap();check_graph(graph).unwrap();
 let importer=SourceId(main.to_str().unwrap().into());
 let index=FileSourceProvider.resolve(Some(&importer),"@/components").unwrap();
 assert_eq!(index,FileSourceProvider.resolve(Some(&importer),"./components/index.tsr").unwrap());
 fs::write(dir.0.join("src/components.tsr"),support::format_round_trip("export function greeting():string{return 'file';}")).unwrap();
 let file=FileSourceProvider.resolve(Some(&importer),"@/components").unwrap();assert_ne!(file,index);
 assert_eq!(file,FileSourceProvider.resolve(Some(&importer),"./components").unwrap());
 let output=support::run(support::cli().arg("run").current_dir(&dir.0),30);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),"file\n");
 let error=FileSourceProvider.resolve(Some(&importer),"@/missing").unwrap_err();assert!(error.contains("missing.tsr")&&error.contains("index.tsr"),"{error}");
}
#[test]
fn alias_config_errors_and_unmapped_packages_do_not_fall_back() {
 let dir=support::TempDirectory::new();let importer=SourceId(dir.0.join("main.tsr").to_str().unwrap().into());
 fs::write(dir.0.join("tsrust.config.json"),r#"{"paths":{"@/*":"src"}}"#).unwrap();
 assert!(FileSourceProvider.resolve(Some(&importer),"@/x").unwrap_err().contains("wildcard"));
 fs::write(dir.0.join("tsrust.config.json"),"{bad").unwrap();assert!(FileSourceProvider.resolve(Some(&importer),"@/x").unwrap_err().contains("tsrust.config.json"));
 fs::remove_file(dir.0.join("tsrust.config.json")).unwrap();assert!(FileSourceProvider.resolve(Some(&importer),"package").unwrap_err().contains("No path alias"));
}
