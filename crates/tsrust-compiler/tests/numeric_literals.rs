mod support;
#[test]
fn separators_and_radices_preserve_contextual_numeric_values(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/numeric-literals/values.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn malformed_separators_and_radix_digits_are_rejected(){
 for literal in ["1__0","1_","0_1","00_1","1_.0","1._0","1e_2","1e+_2","1e2_","0x_FF","0b_1","0o_7","0xFF_","0b102","0o8","0xGG","0b","0o","0x"]{
  let source=format!("function main(){{const value={literal};}}");let error=tsrust_compiler::check(&source).err().unwrap_or_else(||panic!("malformed literal accepted: {literal}"));assert_eq!(error.code,"syntax","{literal}: {error:?}");
 }
}
#[test]
fn radices_preserve_int_limits_source_ranges_and_large_number_contexts(){
 let source="function main(){const value=0x8000_0000_0000_0000;}";let error=tsrust_compiler::check(source).err().expect("out of range literal");assert_eq!(error.code,"integer-range");assert_eq!(&source[error.span.start..error.span.end],"0x8000_0000_0000_0000");
 let directory=support::TempDirectory::new();let source=directory.source("import {strictEqual} from \"std:testing\";function main(){const maximum=0x7fff_ffff_ffff_ffff;const minimum=-0x8000_0000_0000_0000;strictEqual(String(maximum),\"9223372036854775807\");strictEqual(String(minimum),\"-9223372036854775808\");}");let output=support::run(support::cli().arg("run").arg(source),60);support::success(&output);
}
#[test]
fn radix_number_rounding_matches_independent_node_cases(){
 let directory=support::TempDirectory::new();let source=directory.source(include_str!("fixtures/numeric-literals/number-boundaries.tsr"));let output=support::run(support::cli().arg("run").arg(source),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
