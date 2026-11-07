mod support;
#[test]
fn owned_lazy_iterators_preserve_retry_ownership_and_finite_shapes(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 for source in [include_str!("fixtures/owned-iterators/contracts.tsr"),include_str!("fixtures/owned-iterators/contracts.test.tsr")]{assert_eq!(support::format_round_trip(source),source);}
 let output=support::run(support::cli().args(["test","crates/tsrust-compiler/tests/fixtures/owned-iterators/contracts.test.tsr","--reporter=json","--profile"]).current_dir(&root),180);
 support::success(&output);
 assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 print!("{}",String::from_utf8_lossy(&output.stdout));
 let rows=std::str::from_utf8(&output.stdout).unwrap().lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);
 for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{assert!(builds[0][field].as_f64().unwrap()>=0.0,"{field}");}
 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),1);assert_eq!(tests[0]["name"],"owned iterator contracts");assert_eq!(tests[0]["status"],"passed");
 assert!(rows.iter().any(|row|row["event"]=="output"&&row["stream"]=="stdout"&&row["text"]=="owned iterators passed\n"));
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);let summary=summaries[0];assert_eq!(rows.last().unwrap()["event"],"summary");
 for (field,value) in [("passed",1),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summary[field],value,"{field}");}
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
 let source=support::format_round_trip("import {alloc,view,length} from 'std:bytes';function main(){const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const read:Owned<Function<int>>=():int=>length(slice);console.log(read());}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"2\n");
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function main(){const values=Iterator.from([1,2]).flatMap((value:int):Iterator<int>=>Iterator.from([value,value]));for(let index=0;index<4;index++){const step=values.next();if(step.done){throw 'early end';}console.log(step.value);}console.log(values.next().done);}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"1\n1\n2\n2\ntrue\n");
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function counter():Owned<Function<int>>{let current:Iterator<int>|undefined=undefined;return ():int=>{if(current===undefined){current=Iterator.from([3,5]);}const step=current.next();if(step.done){current=undefined;return 0;}return step.value;};}function main(){const next=counter();console.log(next(),next(),next(),next());}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"3 5 0 3\n");
 for source in [
  "type Holder={pull:ReadOnly<Function<int>>;};function bad(value:ReadOnly<Holder>|undefined):Owned<Function<int>>{return ():int=>value===undefined?0:value.pull();}function main(){}",
  "import {Iterator} from 'std:iterator';function bad(value:ReadOnly<Iterator<int>>):Owned<Function<int>>{return ():int=>value.toArray().length;}function main(){}",
  "import {Iterator} from 'std:iterator';function bad(value:ReadOnly<Iterator<int>|undefined>):Owned<Function<void>>{return ():void=>{value=Iterator.from([1]);};}function main(){}",
  "import {alloc,view,length,set} from 'std:bytes';function main(){const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const read:ReadOnly<Function<int>>=():int=>length(slice);unwrap(set(bytes,0,1));console.log(read());}",
  "import {alloc,view,length} from 'std:bytes';function bad():Owned<Function<int>>{const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));return ():int=>length(slice);}function main(){const read=bad();console.log(read());}",
  "import {alloc,view,length,set} from 'std:bytes';function main(){const bytes=unwrap(alloc(2));const slice=unwrap(view(bytes,0,2));const read:Owned<Function<int>>=():int=>length(slice);unwrap(set(bytes,0,1));console.log(read());}",
 ]{let source=support::format_round_trip(source);assert!(checked(&source).is_err(),"{source}");}
}

#[test]
fn owned_iterator_mixed_union_returns_keep_their_physical_alternative(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function values(flag:boolean):int[]|Iterator<int>{if(flag){return [1,2];}return Iterator.from([3,4]).map(value=>value+1);}function main(){const source=Iterator.from([false,true]).flatMap((flag:boolean):int[]|Iterator<int>=>values(flag));console.log(source.toArray().join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"4,5,1,2\n");
 for source in ["import {Iterator} from 'std:iterator';function wrong(flag:boolean):int[]|Iterator<string>{return Iterator.from([1]);}function main(){}","import {Iterator} from 'std:iterator';function wrong():int[]|Iterator<int>{return 'wrong';}function main(){}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
}

#[test]
fn readonly_mixed_union_observations_preserve_the_selected_iterator_owner(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function observe(value:ReadOnly<int[]|Iterator<int>>):boolean{return Array.isArray(value);}function main(){const value:int[]|Iterator<int>=Iterator.from([2,4]);if(!Array.isArray(value)){console.log(observe(value),observe(value));console.log(value.toArray().join(','));}const array:int[]|Iterator<int>=[5,6];if(Array.isArray(array)){console.log(observe(array),observe(array));console.log(array.join(','));}}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"false false\n2,4\ntrue true\n5,6\n");
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function observe(value:ReadOnly<int[]|Iterator<int>>):boolean{return Array.isArray(value);}function main(){const value:int[]|Iterator<int>=Iterator.from([2,4]);if(!Array.isArray(value)){const moved=value;console.log(observe(value));}}");assert!(checked(&source).is_err());
}

#[test]
fn generic_callback_records_and_legacy_owned_mixed_unions_keep_their_contracts(){
 let source=support::format_round_trip("type Holder<T>={value:T;};function relay<T>(value:Owned<Holder<T>>):Holder<T>{return value;}type Reactive<T>={evaluate:Owned<Function<T>>;};type Value<T>=Reactive<T>|T;function snapshot(value:Mutable<Value<string|undefined>|undefined>):string{if(value===undefined){return '';}if(typeof value==='string'){return clone(value);}return value.evaluate()??'';}function invoke(value:Value<string|undefined>|undefined=''):string{return snapshot(value);}function main(){const holder:Holder<Function<int>>={value:():int=>7};const again=relay(holder);console.log(again.value());console.log(invoke());}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"7\n\n");
 for source in ["type Holder<T>={value:T;};function take(value:Owned<Holder<Function<string>>>):void{}function main(){const h:Holder<Function<int>>={value:():int=>7};take(h);}","import {Iterator} from 'std:iterator';function wrong():string[]|Iterator<string>{return Iterator.from([1]);}function main(){}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
}

#[test]
fn concrete_owned_closure_records_inject_into_nullable_storage() {
 let source=support::format_round_trip("type Holder={pull:Owned<Function<int>>;};function main(){let current:Holder|undefined=undefined;const inner:Holder={pull:():int=>7};current=inner;if(current!==undefined){console.log(current.pull());}}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"7\n");
}

#[test]
fn named_nullable_consumers_keep_raw_owned_closure_record_shapes() {
 let source=support::format_round_trip("type Holder={pull:Owned<Function<int>>;};function use(value:Owned<Holder|undefined>):int{if(value===undefined){return 0;}return value.pull();}function read(value:ReadOnly<Holder|undefined>):boolean{return value!==undefined;}function main(){const item:Holder|undefined={pull:():int=>7};console.log(read(item),read(item),use(item));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"true true 7\n");
}

#[test]
fn mapped_started_jobs_transfer_through_iterator_collection(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';import {all,Job} from 'std:futures';async function value():Task<int>{return 3;}async function main():Task<void>{const jobs=[value().start(),value().start()];const values=await all(Iterator.from(jobs).map((work:Job<int,undefined>):Job<int,undefined>=>work).toArray());console.log(values.join(','));}");
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),120);support::success(&output);assert_eq!(output.stdout,b"3,3\n");
}

#[test]
fn late_iterator_factories_finish_after_an_unnamed_catch(){
 let original=include_str!("fixtures/owned-iterators/contracts.tsr");
 let (before,after)=original.rsplit_once("catch (error)").unwrap();
 let source=support::format_round_trip(&format!("{before}catch{}",after.replace("strictEqual(error.code, 'selected');","strictEqual(terminalErrors, 1);")));
 let directory=support::TempDirectory::new();let rust=directory.0.join("program.rs");
 support::success(&support::run(support::cli().arg("emit").arg(directory.source(&source)).arg("-o").arg(&rust),30));
 assert!(rust.is_file());
}
