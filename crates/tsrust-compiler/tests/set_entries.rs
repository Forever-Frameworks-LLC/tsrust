mod support;

#[test]
fn set_entries_generic_temporary_and_result_contracts(){
 for source in ["function main(){new Set<int>().entries(1);}","function main(){const pairs:[string,string][]=new Set<int>([1]).entries();}"]{assert!(tsrust_compiler::check(source).is_err());}
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";enum State{Ready,Busy}function pairs<T>(values:Set<T>):[T,T][]{return values.entries();}function main(){const mixed=new Set<int|string>([1,'one']);const snapshot=pairs(mixed);strictEqual<int|string>(snapshot[1][0],'one');const states=new Set<State>([State.Ready,State.Busy]);const entries=states.entries();strictEqual(entries[1][1],State.Busy);const temporary=new Set<int>([4]).entries();strictEqual(temporary[0][1],4);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn set_entries_copy_ordered_key_pairs_and_survive_source_mutation(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/set-entries/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
