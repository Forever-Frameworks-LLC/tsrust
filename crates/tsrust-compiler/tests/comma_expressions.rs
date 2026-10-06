mod support;
#[test]
fn comma_expressions_preserve_order_final_values_and_completion(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/comma-expressions/values.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn comma_prefixes_keep_explicit_moves_and_receiver_loans(){
 for source in [
 "function main(){let values:int[][]=[[1]];values[0].push((values=[],2));}",
 "function main(){const values=[String('x')];const first=(0,values[0]);values.push('y');console.log(first);}",
 "function main(){const values=[String('x')];const first=(0,values)[0];values.push('y');console.log(first);}",
 "function take(value:Owned<string>):string{return value;}function main(){const word=String('x');const result=(take(word),0);console.log(word);}",
 "function main(){const values=[String('x')];const first=values[0];const selected=(values.push('y'),0);console.log(first);}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe comma operands accepted: {source}");}
}
#[test]
fn comma_final_guard_keeps_narrowing_after_prefix_effects(){
 tsrust_compiler::check("function use(value:int|string):int{if(0,typeof value===\"string\"){return value.length;}return value;}function main(){use(3);}").unwrap();
}
#[test]
fn wide_comma_expression_keeps_stack_bounded_and_separators_distinct(){
 let operands=std::iter::repeat_n("0",512).collect::<Vec<_>>().join(",");
 tsrust_compiler::check(&format!("function main(){{const selected=({operands});console.log(selected);}}")).unwrap();
 for source in ["function main(){const a=1,b=2;const values=[1,2];console.log(a,b,values.length);}","function main(){const values:int[]=(0,[]);values.push(3);}"]{tsrust_compiler::check(source).unwrap();}
}
