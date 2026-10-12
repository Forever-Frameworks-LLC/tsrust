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
