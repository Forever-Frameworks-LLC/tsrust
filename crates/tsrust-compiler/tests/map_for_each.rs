mod support;
#[test]
fn map_for_each_borrows_values_and_keys_and_propagates_errors(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/map-for-each/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn map_for_each_rejects_receiver_mutation_and_input_transfers(){
 for source in [
  "function main(){const values=new Map<int,string>();values.forEach(()=>{values.clear();});}",
  "function main(){const values=new Map<int,string>();values.forEach((value,key,source)=>{source.clear();});}",
  "function main(){const values=new Map<int,string>([[1,'x']]);const output:string[]=[];values.forEach((value)=>{output.push(value);});}",
  "function main(){const values=new Map<string,int>([['x',1]]);const output:string[]=[];values.forEach((value,key)=>{output.push(key);});}",
  "function main(){new Map<int,string>().forEach((value:Owned<string>)=>{});}",
  "function main(){new Map<int,string>().forEach(()=>{},1);}",
 ]{assert!(tsrust_compiler::check(source).is_err(),"unsafe Map callback accepted: {source}");}
}
