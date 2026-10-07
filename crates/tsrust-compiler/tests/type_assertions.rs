mod support;

#[test]
fn assignable_assertions_contextualize_without_losing_satisfies_type(){
 let directory=support::TempDirectory::new();
 let formatted=support::format_round_trip(include_str!("fixtures/type-assertions/assignable.tsr"));
 let path=directory.source(&formatted);
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert!(output.stdout.is_empty());
 assert!(output.stderr.is_empty());
}

#[test]
fn assertions_reject_unassignable_values_and_union_narrowing(){
 for source in [
  "function main(){const value='text' as int;}",
  "function narrow(value:int|string):int{return value as int;}function main(){}",
  "function main(){const values=[1] satisfies string[];}",
 ]{
  let error=tsrust_compiler::check(source).err().expect("unsafe assertion accepted");
  assert!(matches!(error.code,"type-assertion"|"type"),"{}: {}",error.code,error.message);
 }
}

#[test]
fn satisfies_preserves_owned_transfer_rules(){
 let source="function main(){const values=[1];const transferred=values satisfies int[];values.push(2);}";
 let error=tsrust_compiler::check(source).err().expect("owned value reused after transfer");
 assert!(matches!(error.code,"ownership"|"moved"|"consumed"|"use-after-move"|"transferred"),"{}: {}",error.code,error.message);
}

#[test]
fn assertions_preserve_borrows_and_depth_limits() {
    for operator in ["as", "satisfies"] {
        let source = format!("function main(){{const values=[1];const first=values[0] {operator} int;values.push(2);console.log(first);}}");
        // Scalar copies carry no owning element loan.
        tsrust_compiler::check(&source).unwrap();
        let source = format!("function main(){{const values=[\"one\"];const first=values[0] {operator} string;values.push(\"two\");console.log(first);}}");
        let error = tsrust_compiler::check(&source).err().expect("assertion lost borrow");
        assert_ne!(error.code, "borrowed-element", "initial borrow was rejected: {operator}");
    }
    let ordinary = format!("function main(){{const value=1{};}}", " as int".repeat(100));
    tsrust_compiler::check(&ordinary).unwrap();
    let source = format!("function main(){{const value=1{};}}", " as int".repeat(140));
    let error = tsrust_compiler::check(&source).err().expect("unbounded assertion chain");
    assert_eq!(error.code, "unsupported");
}

#[test]
fn assertion_types_do_not_consume_comparison_operators() {
    tsrust_compiler::check("function main(){const other=2;const less=1 as int < other;const greater=3 satisfies number > other;}").unwrap();
}

#[test]
fn narrowing_assertions_explain_the_safe_alternative() {
    let error = tsrust_compiler::check("function narrow(value:int|string):int{return value as int;}function main(){}").err().unwrap();
    assert!(error.message.contains("typeof"), "{}", error.message);
}
