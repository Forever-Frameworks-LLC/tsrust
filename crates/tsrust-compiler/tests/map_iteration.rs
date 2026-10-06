mod support;
#[test]
fn map_iteration_borrows_owned_entries_and_preserves_order(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/map-iteration/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn map_iteration_rejects_mutation_and_transfers_of_borrowed_entries(){
 for source in [
  "function main(){const map=new Map<int,string>([[1,'x']]);for(const entry of map){map.clear();}}",
  "function main(){const map=new Map<int,string>([[1,'x']]);const output:string[]=[];for(const [key,value] of map){output.push(value);}}",
  "function main(){const map=new Map<int,string>([[1,'x']]);for(const entry of map){entry[1]='changed';}}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe Map loop accepted: {source}");}
}
