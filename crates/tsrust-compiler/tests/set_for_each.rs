mod support;

#[test]
fn readonly_abstract_for_each_callbacks_use_read_invocation(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function visitSet(values:Set<int>,action:ReadOnly<Function<void,[int]>>):void{values.forEach(action);}function visitArray(values:int[],action:ReadOnly<Function<void,[int]>>):void{values.forEach(action);}function inspect(value:int):void{strictEqual(value>0,true);}function main(){const values=new Set<int>([1,2]);visitSet(values,inspect);visitArray([1,2],inspect);let caught=0;try{visitSet(new Set<int>([-1]),inspect);}catch(error){caught=1;}strictEqual(caught,1);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn set_for_each_accepts_named_and_abstract_reusable_callbacks(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function visit(values:Set<int>,action:Mutable<Function<void,[int]>>):void{values.forEach(action);}function main(){const values=new Set<int>([1,2]);let total=0;{const action=(value:int)=>{total=total+value;};visit(values,action);values.forEach(action);}strictEqual(total,6);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn set_for_each_default_parameters_and_return_discard_keep_supplied_counts(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function main(){const values=new Set<int>([2,3]);let total=0;values.forEach((value:int,key:int,source:Set<int>,extra:int=7)=>{total=total+value+key+source.size+extra;return String(value);});strictEqual(total,28);let extras=0;values.forEach((value:int,key:int,source:Set<int>,...tail:int[])=>{extras=extras+tail.length;});strictEqual(extras,0);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn array_for_each_callback_errors_reach_the_surrounding_catch(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function main(){let caught=0;try{[1,2].forEach((value)=>{if(value===2){throw 'stop';}});}catch(error){if(typeof error==='string'){strictEqual(error,'stop');caught=1;}}strictEqual(caught,1,'Array.forEach callback error must reach catch');}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn contextual_generic_record_callback_types_reuse_checked_names(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";type Box<T>={value:T;};function main(){const rows:Box<int>[]=[{value:3}];let total=0;rows.forEach((row,index,source)=>{total=total+row.value+source[index].value;});strictEqual(total,6);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn set_for_each_rejects_receiver_mutation_and_owned_input_transfers(){
 for source in [
  "function main(){const values=new Set<int>([1]);values.forEach(()=>{values.clear();});}",
  "function main(){const values=new Set<int>([1]);values.forEach((value,key,source)=>{source.clear();});}",
  "function main(){const values=new Set<string>(['x']);const output:string[]=[];values.forEach((value)=>{output.push(value);});}",
  "function main(){const values=new Set<string>(['x']);values.forEach((value:Owned<string>)=>{});}",
  "function main(){new Set<int>().forEach(()=>{},1);}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe Set callback accepted: {source}");}
}

#[test]
fn set_for_each_borrows_keys_runs_once_in_order_and_propagates_errors(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/set-for-each/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
