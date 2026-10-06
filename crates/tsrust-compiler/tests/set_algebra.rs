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
