mod support;
#[test]
fn array_from_copies_collections_and_maps_borrowed_entries(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/array-from-collections/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn array_from_rejects_receiver_mutation_transfers_and_noncloneable_copies(){
 for source in [
  "function main(){const values=new Set<int>([1]);Array.from(values,()=>{values.clear();return 0;});}",
  "function main(){const values=new Map<int,string>([[1,'x']]);Array.from(values,(entry)=>{values.clear();return 0;});}",
  "function main(){const values=new Map<int,string>([[1,'x']]);Array.from(values,(entry)=>{return entry[1];});}",
  "type Row={action:Once<Function<int>>;};function main(){const values=new Map<int,Row>();Array.from(values);}",
  "function main(){Array.from(new Set<int>(),(value:int,index:int,all:ReadOnly<Set<int>>)=>{return value;});}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe Array.from accepted: {source}");}
}
