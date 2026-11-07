mod support;
#[path="support/iterator_catalog_examples.rs"] mod iterator_catalog_examples;
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
 iterator_catalog_examples::assert_sources();
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
fn temporary_iterator_result_discriminators_evaluate_once_and_keep_ownership_guards(){
 let prefix="type Left={done:false;value:string;};type Right={done:true;value:string;};function make():Left|Right{const value:Left={done:false,value:'kept'};return value;}";
 assert!(checked(&support::format_round_trip(&format!("{prefix}function main(){{}}"))).unwrap().is_empty(),"Factory must check independently of ownership negatives");
 for(body,code)in [("make().done=true;","unsupported"),("const value=make().value;","union-field-ownership"),("const value=make();value.done=true;","union-field-ownership")]{
  let source=format!("{prefix}function main(){{{body}}}");
  assert_eq!(checked(&support::format_round_trip(&source)).unwrap_err(),code,"{body}");
 }
}

#[test]
fn cursor_payloads_use_owning_callback_storage_without_clone(){
 for source in ["import {Iterator} from 'std:iterator';function main(){const wrong:Iterator<string>=Iterator.from([1]);}","import {Iterator} from 'std:iterator';function main(){const wrong:Iterator<Function<string>>=Iterator.from([():int=>1]);}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
}

#[test]
fn flat_map_array_and_iterator_alternatives_keep_order(){
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
fn owned_nullable_capture_slots_preserve_state_and_borrowed_guards(){
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
 for source in ["import {Iterator} from 'std:iterator';function wrong(flag:boolean):int[]|Iterator<string>{return Iterator.from([1]);}function main(){}","import {Iterator} from 'std:iterator';function wrong():int[]|Iterator<int>{return 'wrong';}function main(){}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
}

#[test]
fn readonly_mixed_union_observations_preserve_the_selected_iterator_owner(){
 let source=support::format_round_trip("import {Iterator} from 'std:iterator';function observe(value:ReadOnly<int[]|Iterator<int>>):boolean{return Array.isArray(value);}function main(){const value:int[]|Iterator<int>=Iterator.from([2,4]);if(!Array.isArray(value)){const moved=value;console.log(observe(value));}}");assert!(checked(&source).is_err());
}

#[test]
fn generic_callback_records_and_legacy_owned_mixed_unions_keep_their_contracts(){
 for source in ["type Holder<T>={value:T;};function take(value:Owned<Holder<Function<string>>>):void{}function main(){const h:Holder<Function<int>>={value:():int=>7};take(h);}","import {Iterator} from 'std:iterator';function wrong():string[]|Iterator<string>{return Iterator.from([1]);}function main(){}"]{let source=support::format_round_trip(source);assert!(checked(&source).is_err());}
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
