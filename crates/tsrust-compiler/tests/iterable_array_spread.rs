mod support;
#[test]
fn iterable_array_spread_preserves_order_and_transfers_noncloneable_owners(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/iterable-array-spread/values.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn iterable_array_spread_rejects_owner_reuse_and_borrowed_transfers(){
 for source in [
  "function main(){const values=new Set<string>(['x']);const moved=[...values];values.clear();}",
  "function main(){const values=new Map<int,int>([[1,2]]);const moved=[...values];values.clear();}",
  "function copy(values:ReadOnly<Set<string>>):string[]{return [...values];}function main(){const values=new Set<string>(['x']);copy(values);}",
  "function copy(values:ReadOnly<Map<int,int>>):[int,int][]{return [...values];}function main(){const values=new Map<int,int>([[1,2]]);copy(values);}",
  "function main(){const result=[...{value:1}];}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe iterable spread accepted: {source}");}
}
