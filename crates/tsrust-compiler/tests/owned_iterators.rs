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

#[test]
fn generic_array_shape_probes_preserve_scalar_nonclone_and_task_contracts(){
 let identity="function keep<T>(values:Owned<T[]>):T[]{return values;}";
 for body in ["function main(){const values=keep([1,2]);console.log(values.length);}","import {trackedResource,closeResource} from 'std:testing';function main(){const values=keep([trackedResource()]);closeResource(values.pop()!);}","import {all} from 'std:futures';async function value():Task<int>{return 1;}async function main():Task<void>{await all(keep([value()]));}","import {all} from 'std:futures';async function value():Task<int>{return 1;}async function main():Task<void>{const task=value();await all(keep([task]));}"]{assert!(checked(&format!("{identity}{body}")).is_ok());}
 assert!(checked(&format!("{identity}function main(){{const wrong=keep<int>(['x']);}}")).is_err());
 assert!(checked("import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([1,2]).map(value=>value+1);console.log(values.toArray());}").is_ok());
 assert!(checked("import {Iterator} from 'std:iterator';import {trackedResource,closeResource} from 'std:testing';function main(){const values=Iterator.from([trackedResource()]).map(value=>{closeResource(value);return 1;});console.log(values.toArray());}").is_ok());
 assert!(checked("function bad(value:Owned<int>):int{return value;}function main(){bad(1);}").is_err());
 assert!(checked("function need<T,U>(action:Owned<Function<U,[value:Owned<T>]>>):void{}function main(){need(value=>value);}").is_err());
}

#[test]
fn borrowed_and_owned_record_parameters_execute_in_both_orders(){
 for borrowed_first in [true,false]{
  let calls=if borrowed_first{"console.log(useHolder(borrowed));console.log(useHolder(copied));console.log(useOwned(owned));"}else{"console.log(useOwned(owned));console.log(useHolder(copied));console.log(useHolder(borrowed));"};
  let source=format!("import {{alloc,view,length}} from 'std:bytes';import {{trackedResource,resourceId,releaseCount,closeResource}} from 'std:testing';type Holder={{pull:ReadOnly<Function<int>>;}};type OwnedHolder={{pull:Once<Function<int>>;}};function useHolder(value:ReadOnly<Holder>):int{{return value.pull();}}function useOwned(value:Owned<OwnedHolder>):int{{return value.pull();}}function main(){{const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const borrowed:Holder={{pull:():int=>length(slice)}};const number=7;const copied:Holder={{pull:():int=>number}};const resource=trackedResource();const id=resourceId(resource);const owned:OwnedHolder={{pull:():int=>{{closeResource(resource);return 9;}}}};{calls}console.log(releaseCount(id));}}");
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(&source))),120);support::success(&output);assert_eq!(output.stdout,if borrowed_first{b"2\n7\n9\n1\n"}else{b"9\n7\n2\n1\n"});
 }
}

#[test]
fn temporary_iterator_result_discriminators_evaluate_once_and_keep_ownership_guards(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([4]);console.log(values.next().done);console.log(values.next().done);}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"false\ntrue\n");
 let prefix="type Left={done:false;value:string;};type Right={done:true;value:string;};function make():Left|Right{const value:Left={done:false,value:'kept'};return value;}";
 assert!(checked(&support::format_round_trip(&format!("{prefix}function main(){{}}"))).unwrap().is_empty(),"Factory must check independently of ownership negatives");
 for(body,code)in [("make().done=true;","unsupported"),("const value=make().value;","union-field-ownership"),("const value=make();value.done=true;","union-field-ownership")]{
  let source=format!("{prefix}function main(){{{body}}}");
  assert_eq!(checked(&support::format_round_trip(&source)).unwrap_err(),code,"{body}");
 }
}

#[test]
fn cursor_payloads_use_owning_callback_storage_without_clone(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';import {all} from 'std:futures';async function task():Task<int>{return 3;}async function main():Task<void>{const number=7;const callbacks=Iterator.from([():int=>number]).toArray();console.log(callbacks[0]());const values=await all(Iterator.from([task()]).toArray());console.log(values.join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"7\n3\n");
 for source in ["import {Iterator} from 'std:iterator';function main(){const wrong:Iterator<string>=Iterator.from([1]);}","import {Iterator} from 'std:iterator';function main(){const wrong:Iterator<Function<string>>=Iterator.from([():int=>1]);}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
}

#[test]
fn flat_map_array_and_iterator_alternatives_keep_order(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([1,2]).flatMap((value:int):int[]|Iterator<int>=>value===1?[value,value]:Iterator.from([value]));console.log(values.toArray().join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"1,1,2\n");
 assert!(checked(&support::format_round_trip("type Holder={value:int;};function count(value:int[]|Holder):int{if(Array.isArray(value)){return value.length;}return value.value;}function main(){console.log(count([1]),count({value:3}));}")).is_ok());
 assert!(checked(&support::format_round_trip("type Holder={value:int;};function wrong(value:int[]|Holder):int{if(Array.isArray(value)){return value.value;}return 0;}function main(){}")).is_err());
 for source in [
  "type Holder={value:int;};function count(value:int[]|Holder):int{if(!Array.isArray(value)){return value.value;}return value.length;}function main(){}",
  "type Holder={value:int;};function main(){let value:int[]|Holder=[1];while(Array.isArray(value)){value={value:2};}console.log(value.value);}",
 ]{assert!(checked(&support::format_round_trip(source)).is_ok(),"{source}");}
 for source in [
  "type Holder={value:int;};type Row={value:int[]|Holder;};function wrong(row:Row):int{if(Array.isArray(row.value)){row.value={value:2};return row.value.length;}return 0;}function main(){}",
  "type Holder={value:int;};type Row={value:int[]|Holder;};function change(row:Row){row.value={value:2};}function wrong(row:Row):int{if(Array.isArray(row.value)){change(row);return row.value.length;}return 0;}function main(){}",
  "function main(){const value=JSON.parse('[1]');if(Array.isArray(value)){console.log(value.length);}}",
 ]{assert!(checked(&support::format_round_trip(source)).is_err(),"{source}");}
}

#[test]
fn nested_iterator_receiver_preparation_keeps_chains_and_callback_items(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function main(){console.log(Iterator.from([1,2,3]).take(2).toArray().join(','));const number=7;const callbacks=Iterator.from([():int=>number]).toArray();console.log(callbacks[0]());}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"1,2\n7\n");
}

#[test]
fn owned_nullable_capture_slots_preserve_state_and_borrowed_guards(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([1,2]).flatMap((value:int):Iterator<int>=>Iterator.from([value,value]));for(let index=0;index<4;index++){const step=values.next();if(step.done){throw 'early end';}console.log(step.value);}console.log(values.next().done);}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"1\n1\n2\n2\ntrue\n");
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function counter():Owned<Function<int>>{let current:Iterator<int>|undefined=undefined;return ():int=>{if(current===undefined){current=Iterator.from([3,5]);}const step=current.next();if(step.done){current=undefined;return 0;}return step.value;};}function main(){const next=counter();console.log(next(),next(),next(),next());}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"3 5 0 3\n");
 for source in [
  "type Holder={pull:ReadOnly<Function<int>>;};function bad(value:ReadOnly<Holder>|undefined):Owned<Function<int>>{return ():int=>value===undefined?0:value.pull();}function main(){}",
  "import {Iterator} from 'std:iterator';function bad(value:ReadOnly<Iterator<int>>):Owned<Function<int>>{return ():int=>value.toArray().length;}function main(){}",
  "import {Iterator} from 'std:iterator';function bad(value:ReadOnly<Iterator<int>|undefined>):Owned<Function<void>>{return ():void=>{value=Iterator.from([1]);};}function main(){}",
  "import {alloc,view,length,set} from 'std:bytes';function main(){const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const read:ReadOnly<Function<int>>=():int=>length(slice);unwrap(set(bytes,0,1));console.log(read());}",
  "import {alloc,view,length} from 'std:bytes';function main(){const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const read:Owned<Function<int>>=():int=>length(slice);console.log(read());}",
 ]{let source=support::format_round_trip(source);assert!(checked(&source).is_err(),"{source}");}
}
