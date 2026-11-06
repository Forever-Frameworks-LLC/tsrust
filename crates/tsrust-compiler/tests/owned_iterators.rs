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

#[test]
fn array_cursor_results_follow_each_declared_item_type(){
 let source="import {Iterator,IteratorResult} from 'std:iterator';function main(){const left:IteratorResult<int> = Iterator.from([1]).next();const right:IteratorResult<string> = Iterator.from(['x']).next();console.log(left.done,right.done);}";
 assert!(checked(source).is_ok());
 assert!(checked(&source.replace("const right:IteratorResult<string>","const right:IteratorResult<int>")).is_err());
}

#[test]
fn iterator_physical_carriers_survive_owned_returns_captures_and_joins(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function owned(source:Owned<Iterator<int>>):Iterator<int>{return source;}function choose(flag:boolean):Iterator<int>{if(flag){return Iterator.from([2,4]);}return Iterator.from([3,5]).map(value=>value+1);}function capture(source:Owned<Iterator<int>>):Once<Function<int[]>>{return ():int[]=>source.toArray();}function main(){const action=capture(owned(choose(true)));console.log(action().join(','));console.log(owned(choose(false)).toArray().join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"2,4\n4,6\n");
}

#[test]
fn explicit_iterator_closure_results_and_conditional_shapes_execute(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function conditional(flag:boolean):Iterator<int>{return flag?Iterator.from([8]):Iterator.from([3]).map(value=>value+1);}function main(){const factory=():Iterator<int>=>Iterator.from([2,4]);console.log(factory().toArray().join(','));console.log(conditional(true).toArray().join(','));console.log(conditional(false).toArray().join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"2,4\n8\n4\n");
}
