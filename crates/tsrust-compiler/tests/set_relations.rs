mod support;
#[test]
fn set_relations_compare_current_key_types_without_mutation(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/set-relations/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn set_relations_reject_wrong_arguments_results_and_conflicting_evaluation(){
 for source in [
  "function main(){new Set<int>().isSubsetOf();}",
  "function main(){new Set<int>().isSupersetOf(new Set<int>(),new Set<int>());}",
  "function main(){new Set<int>().isDisjointFrom([1]);}",
  "function main(){const result:int=new Set<int>().isSubsetOf(new Set<int>());}",
  "function clear(values:Mutable<Set<int>>):Set<int>{values.clear();return new Set<int>();}function main(){const values=new Set<int>([1]);values.isSubsetOf(clear(values));}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"invalid relation accepted: {source}");}
}

#[test]
fn set_relations_preserve_exact_int_and_enum_identity_rules(){
 let directory=support::TempDirectory::new();let path=directory.source(r#"
import {strictEqual} from "std:testing";
enum Left{Ready}enum Right{Ready}
function main(){
 const exact=new Set<int>([9007199254740992,9007199254740993]);strictEqual(exact.size,2);
 strictEqual(new Set<int>([9007199254740993]).isSubsetOf(new Set<int>([9007199254740992])),false);
 strictEqual(exact.isSubsetOf(new Set<number>([9007199254740992.0])),true);
 strictEqual(new Set<Left>([Left.Ready]).isDisjointFrom(new Set<Right>([Right.Ready])),true);
}
"#);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
