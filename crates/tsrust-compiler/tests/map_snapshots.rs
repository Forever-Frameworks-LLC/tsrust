mod support;

#[test]
fn only_value_copying_snapshots_require_cloneable_values(){
 let declaration="type Row={action:Once<Function<int>>;};function main(){const values=new Map<int,Row>();values.set(1,{action:():int=>{return 7;}});";
 for method in ["values","entries"] {
  let source=format!("{declaration}values.{method}();}}");
  let error=tsrust_compiler::check(&source).err().expect("non-cloneable values copied");
  assert_eq!(error.code,"clone-type","{}",error.message);
 }
 let directory=support::TempDirectory::new();
 let path=directory.source(&format!("import {{strictEqual}} from \"std:testing\";{declaration}const keys=values.keys();strictEqual(keys[0],1);strictEqual(values.size,1);}}"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn map_snapshots_check_arity_result_types_and_receiver_evaluation(){
 for source in [
  "function main(){const values=new Map<int,int>();values.keys(1);}",
  "function main(){const values=new Map<int,int>();values.values(1);}",
  "function main(){const values=new Map<int,int>();values.entries(1);}",
  "function main(){const values=new Map<string,int>();const pairs:[int,int][]=values.entries();}",
 ] {assert!(tsrust_compiler::check(source).is_err(),"unsafe snapshot accepted: {source}");}
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function make(trace:int[]):Map<int,string>{trace.push(1);return new Map<int,string>([[2,'two']]);}function copy<T>(values:Map<int,T>):T[]{return values.values();}function main(){const trace:int[]=[];const pairs=make(trace).entries();strictEqual(trace.length,1);strictEqual(pairs[0][1],'two');const values=new Map<int,int>([[1,4]]);const snapshot=copy(values);values.clear();strictEqual(snapshot[0],4);}");
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
}

#[test]
fn map_snapshots_preserve_order_types_and_survive_source_mutation(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/map-snapshots/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn map_value_snapshots_explicitly_copy_owned_arrays(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function main(){const values=new Map<int,int[]>();values.set(1,[2,3]);const snapshot=values.values();snapshot[0].push(4);{const original=values.get(1);strictEqual(original?.length??0,2);}values.clear();strictEqual(snapshot[0].length,3);}");
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
}
