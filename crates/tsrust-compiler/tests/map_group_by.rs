mod support;
#[test]
fn map_group_by_retains_order_copies_and_callback_completion(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/map-group-by/values.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn map_group_by_rejects_source_mutation_and_noncopyable_values(){
 for source in [
 "function main(){const values=[1,2];Map.groupBy(values,()=>{values.push(3);return 0;});}",
 "function main(){const values=[1];Map.groupBy(values,(value:Mutable<int>)=>{return value;});}",
 "type Row={action:Once<Function<int>>;};function main(){const rows:Row[]=[];Map.groupBy(rows,()=>{return 0;});}",
 "function main(){Map.groupBy([1],(value:int,index:int,source:int[])=>{return 0;});}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe grouping accepted: {source}");}
}

#[test]
fn map_group_by_copies_owning_records_independently(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/map-group-by/owned-copy.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
