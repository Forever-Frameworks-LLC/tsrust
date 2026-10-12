//! Backend feasibility controls; these do not expose a Tsrust thread API.
mod support;
use std::process::Command;

fn compile(source:&str)->(support::TempDirectory,std::process::Output,std::path::PathBuf){
 let directory=support::TempDirectory::new();let path=directory.0.join("thread.rs");std::fs::write(&path,source).unwrap();let binary=directory.0.join("thread");
 let output=support::run(support::rustc().args(["--edition=2024","--error-format=short"]).arg(path).arg("-o").arg(&binary),60);
 (directory,output,binary)
}

#[test]
fn owned_payload_moves_to_worker_and_back_without_copying(){
 let (_directory,compiled,binary)=compile(r#"
fn main(){
 let data=vec![3u8,11,23];let pointer=data.as_ptr() as usize;
 let owner=std::thread::current().id();
 let worker=std::thread::spawn(move||{assert_ne!(std::thread::current().id(),owner);data});
 let output=worker.join().unwrap();assert_eq!(output,[3,11,23]);assert_eq!(output.as_ptr() as usize,pointer);
 // Channels transfer owned payloads and return an unsent value on saturation.
 let (send,receive)=std::sync::mpsc::sync_channel(1);
 send.send(output).unwrap();
 let rejected=match send.try_send(vec![42u8]){Err(std::sync::mpsc::TrySendError::Full(value))=>value,_=>panic!("bounded channel accepted overflow")};
 assert_eq!(rejected,[42]);let received=receive.recv().unwrap();assert_eq!(received.as_ptr() as usize,pointer);
 drop(send);assert!(receive.recv().is_err());
 // Backend panic recovery is distinct from a source-language thrown Result.
 let panicked=std::thread::spawn(||std::panic::panic_any(17u32)).join();
 assert_eq!(*panicked.unwrap_err().downcast::<u32>().unwrap(),17);
 println!("thread ownership controls passed");
}
"#);
 support::success(&compiled);let output=support::run(&mut Command::new(binary),10);support::success(&output);
 assert_eq!(output.stdout,b"thread ownership controls passed\n");
}

#[test]
fn backend_rejects_borrowed_and_non_send_captures(){
 for (source,code) in [
  ("fn main(){let text=String::from(\"owner\");let view=text.as_str();let worker=std::thread::spawn(move||view.len());worker.join().unwrap();}","E0597"),
  ("fn main(){let shared=std::rc::Rc::new(7);std::thread::spawn(move||*shared).join().unwrap();}","E0277"),
 ] {
  let (_directory,output,_)=compile(source);assert!(!output.status.success());
  let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains(&format!("error[{code}]")),"{stderr}");
 }
}

fn generated_module(name:&str,source:&str,witness:&str)->String {
 let source=support::format_round_trip(source);
 let generated=tsrust_compiler::check(&source).unwrap().emit_rust();
 // Keep the generated environment, factory and invocation bodies unchanged.
 // The handwritten witness supplies only the not-yet-public thread boundary.
 format!("mod {name} {{\n{generated}\npub fn verify(){{{witness}}}\n}}\n")
}

#[test]
fn generated_owned_closures_cross_threads_and_preserve_results(){
 let mut native=String::from("#![recursion_limit=\"512\"]\n");
 for (name,source,witness) in [
  ("moved","function factory(owner:string):Function<string>{return ():string=>owner;}function main(){}",r#"
   let input=std::string::String::from("owned worker payload");let pointer=input.as_ptr() as usize;
   let callback=function0(String::Owned(input));let caller=std::thread::current().id();
   let result=std::thread::spawn(move||{assert_ne!(std::thread::current().id(),caller);callback.call()}).join().unwrap();
   assert_eq!(result,"owned worker payload");assert_eq!(result.as_ptr() as usize,pointer);
  "#),
  ("read","function factory(owner:string):Function<int>{return ():int=>owner.length;}function main(){}",r#"
   let callback=function0(String::Owned("owned worker payload".to_owned()));
   let result=std::thread::spawn(move||(callback.call(),callback.call())).join().unwrap();
   assert_eq!(result,(20,20));
  "#),
  ("throws","function factory(owner:string):Function<int>{return ():int=>{throw owner;};}function main(){}",r#"
   let input=std::string::String::from("worker failure");let pointer=input.as_ptr() as usize;
   let callback=function0(String::Owned(input));
   // Source throws remain checked errors; they do not become backend panics.
   let result=std::thread::spawn(move||callback.call()).join().expect("source throw must not panic");
   match result {Err(TsrClosureError0::V0(value))=>{assert_eq!(value,"worker failure");assert_eq!(value.as_ptr() as usize,pointer);},_=>panic!("lost source throw")}
  "#),
 ] {native.push_str(&generated_module(name,source,witness));}
 native.push_str("fn main(){moved::verify();read::verify();throws::verify();}");
 let (_directory,compiled,binary)=compile(&native);support::success(&compiled);
 let output=support::run(&mut Command::new(binary),10);support::success(&output);assert!(output.stdout.is_empty());
}

#[test]
fn generated_borrowed_closures_cannot_escape_on_threads(){
 for (source,witness) in [
  ("function main(){const owner='bytes';const callback=():int=>owner.length;callback();}",r#"
   let owner=String::Owned("borrowed".to_owned());let callback=Closure0{capture0:&owner};
   std::thread::spawn(move||callback.call()).join().unwrap();
  "#),
  ("function main(){let owner=42;const callback=():int=>{owner+=1;return owner;};callback();}",r#"
   let mut owner=42;let mut callback=Closure0{capture0:&mut owner};
   std::thread::spawn(move||callback.call()).join().unwrap();
  "#),
 ] {
  let native=generated_module("borrowed",source,witness)+"fn main(){borrowed::verify();}";
  let (_directory,output,_)=compile(&native);assert!(!output.status.success());
  let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("error[E0597]"),"{stderr}");
 }
}
