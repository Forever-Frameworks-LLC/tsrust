mod support;
#[test]
fn owned_lazy_iterators_preserve_retry_ownership_and_finite_shapes(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("fixtures/owned-iterators/contracts.tsr"));
 let file=directory.source(&source);
 let output=support::run(support::cli().arg("run").arg(file).current_dir(&directory.0),120);
 support::success(&output);
 assert!(String::from_utf8_lossy(&output.stdout).contains("owned iterators passed"));
}
fn checked(source:&str)->Result<Vec<String>,String>{
 use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
 let directory=support::TempDirectory::new();let path=directory.source(source);
 let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
 check_graph(graph).map(|checked|checked.warnings().into_iter().map(|warning|warning.code.to_owned()).collect()).map_err(|error|error.code.to_owned())
}
#[test]
fn lazy_and_started_responsibilities_remain_visible_inside_iterator_state(){
 let header="import {Iterator} from 'std:iterator';import {all,Job} from 'std:futures';async function value():Task<int>{return 3;}";
 let lazy=format!("{header}async function main():Task<void>{{const pending=Iterator.from([value()]);}}");
 assert!(checked(&lazy).unwrap().iter().any(|code|code=="unused-task"));
 let started=format!("{header}async function main():Task<void>{{const pending=Iterator.from([value().start()]);}}");
 assert_eq!(checked(&started).unwrap_err(),"unsettled-job");
 for source in [
  "const pending=Iterator.from([value()]);await all(pending.toArray());",
  "const pending=Iterator.from([value().start()]);await all(pending.toArray());",
 ]{let source=format!("{header}async function main():Task<void>{{{source}}}");assert!(checked(&source).unwrap().is_empty());}
}

#[test]
fn owned_iterator_catalog_examples_execute_and_hide_private_cursor(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let mut count=0;
 for receiver in ["std:iterator","IteratorConstructor","Iterator"]{for entry in catalog[receiver].as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){let directory=support::TempDirectory::new();let source=support::format_round_trip(example["source"].as_str().unwrap());let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,example["output"].as_str().unwrap().as_bytes(),"{}",entry["name"]);count+=1;}}}assert_eq!(count,15);
 for source in [
  "import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([1]);values._pull();}",
  "import {Iterator} from 'std:iterator';function __tsrust_private_iterator_next(source:Mutable<Iterator<int>>):void{source._pull();}function main(){const values=Iterator.from([1]);__tsrust_private_iterator_next(values);}",
  "import {Iterator} from 'std:iterator';function __tsrust_private_iterator_next(source:Mutable<Iterator<int>>):void{const pull=source._pull;}function main(){const values=Iterator.from([1]);__tsrust_private_iterator_next(values);}",
 ]{assert_eq!(checked(source).unwrap_err(),"iterator-member","{source}");}
}
