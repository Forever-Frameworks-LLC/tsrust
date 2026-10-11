mod support;
use std::fs;
use tsrust_compiler::{project::{load_project,check_graph,FileSourceProvider},native_runtime::separate};
#[test]
fn separated_runtime_preserves_native_views_files_and_paths() {
 let dir=support::TempDirectory::new();
 for (name,source) in [
  ("text","function main(){const text=' hi ';console.log(text.trim().toUpperCase(),String(7));}"),
  ("views","import {fromArray,view,at} from 'std:bytes';function main(){match(fromArray([7,8])){Ok(bytes)=>{match(view(bytes,0,1)){Ok(part)=>{match(at(part,0)){Ok(value)=>{console.log(value);},Err(error)=>{throw error.message;}}},Err(error)=>{throw error.message;}}},Err(error)=>{throw error.message;}}}"),
  ("files","import {readFile} from 'std:fs';function main(){match(readFile('missing-runtime-test-file')){Ok(bytes)=>{console.log('unexpected');},Err(error)=>{console.log(error.code);}}}"),
  ("paths","import {basename} from 'std:path';function main(){console.log(basename('/a/b.txt'));}"),
 ] {
  let entry=dir.0.join(format!("{name}.tsr"));fs::write(&entry,support::format_round_trip(source)).unwrap();
  let rust=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap().emit_rust();
  let split=separate(&rust);let runtime=split.runtime.expect("native support must split");
  assert!(!split.application.contains("fn tsr_string_html("),"string support must be separate");
  assert!(!split.application.contains("enum TsrConsoleValue"),"console support must be separate");
  assert!(!runtime.contains("struct Record"));assert!(!split.application.contains("mod tsrust_native_"));
  fs::write(&entry,support::format_round_trip(&source.replace("function main(){","function main(){const extra:int=1;"))).unwrap();
  let edited=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap().emit_rust();
  assert_ne!(edited,rust);assert_eq!(separate(&edited).runtime.as_deref(),Some(runtime.as_str()),"application edits must preserve runtime identity");
  fs::write(dir.0.join("runtime.rs"),runtime).unwrap();fs::write(dir.0.join("app.rs"),split.application).unwrap();
  for opt in ["0","3"] {
   let library=dir.0.join("libtsrust_runtime.rlib");
   support::success(&support::run(support::rustc().args(["--edition=2024","--crate-type=rlib","--crate-name=tsrust_runtime","-C",&format!("opt-level={opt}"),"-C","overflow-checks=on"]).arg(dir.0.join("runtime.rs")).arg("-o").arg(&library),60));
   if name=="views" && opt=="0" {
    fs::write(dir.0.join("bad.rs"),"fn invalid()->&'static [u8]{let bytes=vec![1u8];match tsrust_runtime::tsrust_native_bytes::view(&bytes,0,1){Ok(view)=>view,Err(_)=>panic!()}} fn main(){}").unwrap();
    let rejected=support::run(support::rustc().args(["--edition=2024","--extern"]).arg(format!("tsrust_runtime={}",library.display())).arg(dir.0.join("bad.rs")).arg("-o").arg(dir.0.join("bad")),60);
    assert!(!rejected.status.success());assert!(String::from_utf8_lossy(&rejected.stderr).contains("E0515"));
   }
   let binary=dir.0.join("app");
   support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={opt}"),"-C","overflow-checks=on","--extern"]).arg(format!("tsrust_runtime={}",library.display())).arg(dir.0.join("app.rs")).arg("-o").arg(&binary),60));
   let output=support::run(std::process::Command::new(binary).current_dir(&dir.0),20);support::success(&output);
   assert_eq!(String::from_utf8(output.stdout).unwrap(),match name {"text"=>"HI 7\n","views"=>"7\n","paths"=>"b.txt\n",_=>"not-found\n"});
  }
 }
 let simple=tsrust_compiler::check("function main(){}").unwrap().emit_rust();let split=separate(&simple);assert!(split.runtime.is_none());assert_eq!(split.application,simple);
}
