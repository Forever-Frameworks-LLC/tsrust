mod support;
#[test]
fn bounded_full_file_access_preserves_binary_content_and_reports_limits() {
 let directory=support::TempDirectory::new();
 std::fs::write(directory.0.join("output.bin"),b"old trailing data").unwrap();
 let source=support::format_round_trip(r#"
import {readFile,writeFile} from "std:fs";
import {fromArray,view,length,at} from "std:bytes";
import {strictEqual} from "std:testing";
function main(){
 match(fromArray([0,255,65,66])){
  Ok(owner)=>{
   match(view(owner,0,3)){
    Ok(bytes)=>{match(writeFile("output.bin",bytes)){Ok(_)=>{},Err(error)=>{throw error.message;}}},
    Err(error)=>{throw error.message;}
   }
  },Err(error)=>{throw error.message;}
 }
 match(readFile("output.bin")){
  Ok(bytes)=>{strictEqual(length(bytes),3);match(at(bytes,1)){Ok(value)=>{strictEqual(value,255);},Err(error)=>{throw error.message;}}},
  Err(error)=>{throw error.message;}
 }
 match(readFile("output.bin",3)){Ok(_)=>{},Err(error)=>{throw error.message;}}
 match(readFile("output.bin",2)){Ok(_)=>{throw "limit ignored";},Err(error)=>{strictEqual(error.code,"length-range");strictEqual(error.operation,"readFile");}}
 match(readFile("missing.bin",-1)){Ok(_)=>{throw "negative limit ignored";},Err(error)=>{strictEqual(error.code,"length-range");}}
}
"#);
 let file=directory.source(&source);
 support::success(&support::run(support::cli().arg("run").arg(file).current_dir(&directory.0),60));
 assert_eq!(std::fs::read(directory.0.join("output.bin")).unwrap(),[0,255,65]);
}
