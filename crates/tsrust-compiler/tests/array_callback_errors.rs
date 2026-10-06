mod support;

#[test]
fn array_callback_errors_propagate_and_leave_receivers_valid(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/array-callback-errors/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn readonly_abstract_callbacks_propagate_errors(){
 let directory=support::TempDirectory::new();
 let path=directory.source(r#"
import {strictEqual} from "std:testing";
function apply(values:int[], action:ReadOnly<Function<int,[int]>>):int[]{return values.map(action);}
function main(){
 const values=[1,2];let caught=0;
 const action=(value:int):int=>{if(value===2){throw "readonly map";}return value;};
 try{apply(values,action);}catch(error){if(typeof error==="string"){strictEqual(error,"readonly map");caught=1;}}
 strictEqual(caught,1);
}
"#);
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
