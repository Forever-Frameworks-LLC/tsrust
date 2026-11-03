mod support;
#[test]
fn set_algebra_copies_results_and_preserves_node_order(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/set-algebra/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn set_algebra_rejects_wrong_arguments_results_and_conflicting_evaluation(){
 for source in [
  "function main(){new Set<int>().union();}",
  "function main(){new Set<int>().intersection([1]);}",
  "function main(){new Set<int>().difference(new Set<int>(),new Set<int>());}",
  "function main(){const result:boolean=new Set<int>().symmetricDifference(new Set<int>());}",
  "function clear(values:Mutable<Set<int>>):Set<int>{values.clear();return new Set<int>();}function main(){const values=new Set<int>([1]);values.union(clear(values));}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"invalid algebra accepted: {source}");}
}

#[test]
fn intersection_keeps_every_exact_receiver_key_under_numeric_overlap(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/set-algebra/precision.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn cross_key_queries_hash_only_indexed_storage_and_preserve_collisions() {
 let source=support::format_round_trip("function main(){const values=new Set<int>();console.log(values.size);}");
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();
 assert!(!emitted.contains("unsafe"));
 let assertions=r#"
static HASH_CALLS: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
#[derive(Clone)]struct Counted(i64);
impl TsrSetKey for Counted {
 fn tsr_hash(&self)->u64{HASH_CALLS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);0}
 fn tsr_same(&self,other:&Self)->bool{self.0==other.0}
}
fn main(){
 source_main();
 let same=|a:&Counted,b:&Counted|a.0==b.0;
 for count in [0,1,8,9,17] {
  let mut values=TsrSet::from_values((0..count).map(Counted));
  HASH_CALLS.store(0,std::sync::atomic::Ordering::Relaxed);
  assert_eq!(values.has_with(&Counted(0),&same),count>0);
  assert_eq!(values.matches_with(&Counted(0),&same).count(),usize::from(count>0));
  assert!(!values.has_with(&Counted(99),&same));
  assert_eq!(values.matches_with(&Counted(99),&same).count(),0);
  assert_eq!(HASH_CALLS.load(std::sync::atomic::Ordering::Relaxed),if count<=8{0}else{4});
  if count>8 {
   for key in 1..count{assert!(values.delete(&Counted(key)));}
   assert_eq!(values.len(),1);
   HASH_CALLS.store(0,std::sync::atomic::Ordering::Relaxed);
   assert!(values.has_with(&Counted(0),&same));
   assert_eq!(values.matches_with(&Counted(0),&same).count(),1);
   assert_eq!(HASH_CALLS.load(std::sync::atomic::Ordering::Relaxed),2);
  }
  values.clear();
  HASH_CALLS.store(0,std::sync::atomic::Ordering::Relaxed);
  assert!(!values.has_with(&Counted(0),&same));
  assert_eq!(values.matches_with(&Counted(0),&same).count(),0);
  assert_eq!(HASH_CALLS.load(std::sync::atomic::Ordering::Relaxed),0);
 }
}
"#;
 let directory=support::TempDirectory::new();let path=directory.0.join("cross-key.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+assertions).unwrap();
 for level in ["0","3"] {
  let executable=directory.0.join(format!("cross-key-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),60));
  let output=support::run(&mut std::process::Command::new(executable),30);support::success(&output);
  assert_eq!(output.stdout,b"0\n");assert!(output.stderr.is_empty());
 }
}
