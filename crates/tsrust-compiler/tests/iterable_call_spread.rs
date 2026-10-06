mod support;
#[test]
fn iterable_call_spread_preserves_contracts_and_source_order(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/iterable-call-spread/values.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn iterable_call_spread_rejects_unsafe_collection_key_access_and_owner_reuse(){
 for source in [
 "function take(value:Owned<string>):string{return value;}function main(){const values=new Set<string>(['x']);take(...values);values.clear();}",
 "function take(value:Owned<string>):string{return value;}function copy(values:ReadOnly<Set<string>>):string{return take(...values);}function main(){}",
 "function touch(value:Mutable<string>){value='changed';}function main(){const values=new Set<string>(['x']);touch(...values);}",
 "function touch(value:Mutable<[int,int]>){value[0]=3;}function main(){const values=new Map<int,int>([[1,2]]);touch(...values);}",
 "function main(){console.log(...{value:1});}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe iterable call spread accepted: {source}");}
}

#[test]
fn dynamic_spread_arity_errors_are_catchable_and_run_finally(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/iterable-call-spread/arity-recovery.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
