//! Backend controls only; no source async scheduling or cancellation policy.
mod support;
use std::process::Command;

fn compile(source:&str)->(support::TempDirectory,std::process::Output,std::path::PathBuf){
 let directory=support::TempDirectory::new();let path=directory.0.join("async.rs");std::fs::write(&path,source).unwrap();let binary=directory.0.join("async");
 let output=support::run(support::rustc().args(["--edition=2024","--error-format=short"]).arg(path).arg("-o").arg(&binary),60);
 (directory,output,binary)
}

#[test]
fn suspended_owned_and_borrowed_state_has_deterministic_drop(){
 let (_directory,compiled,binary)=compile(r#"
use std::{future::Future,pin::Pin,sync::{Arc,atomic::{AtomicUsize,Ordering}},task::{Context,Poll,Wake,Waker}};
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {fn wake(self:Arc<Self>){self.0.fetch_add(1,Ordering::SeqCst);}}
struct YieldOnce(bool);
impl Future for YieldOnce {
 type Output=();
 fn poll(mut self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<()> {
  if self.0 {Poll::Ready(())} else {self.0=true;cx.waker().wake_by_ref();Poll::Pending}
 }
}
struct Resource(Arc<AtomicUsize>);
impl Drop for Resource {fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
async fn operation(resource:Resource,text:String,fail:bool)->Result<String,String>{
 // This reference into the future's owned storage survives a real suspension.
 let view=text.as_str();YieldOnce(false).await;assert_eq!(view,"payload");
 drop(resource);if fail {Err(text)} else {Ok(text)}
}
fn main(){
 let wake=Arc::new(WakeCount(AtomicUsize::new(0)));let waker=Waker::from(wake.clone());let mut cx=Context::from_waker(&waker);
 for fail in [false,true] {
  let drops=Arc::new(AtomicUsize::new(0));let text="payload".to_owned();let pointer=text.as_ptr() as usize;
  let mut future=Box::pin(operation(Resource(drops.clone()),text,fail));
  assert!(future.as_mut().poll(&mut cx).is_pending());assert_eq!(drops.load(Ordering::SeqCst),0);
  let result=match future.as_mut().poll(&mut cx){Poll::Ready(value)=>value,Poll::Pending=>panic!("second poll must complete")};
  assert_eq!(result.is_err(),fail);let text=match result{Ok(text)|Err(text)=>text};assert_eq!(text.as_ptr() as usize,pointer);
  assert_eq!(drops.load(Ordering::SeqCst),1);drop(future);assert_eq!(drops.load(Ordering::SeqCst),1);
 }
 for poll_first in [false,true] {
  let drops=Arc::new(AtomicUsize::new(0));let mut future=Box::pin(operation(Resource(drops.clone()),"payload".to_owned(),false));
  if poll_first {assert!(future.as_mut().poll(&mut cx).is_pending());}
  assert_eq!(drops.load(Ordering::SeqCst),0);drop(future);assert_eq!(drops.load(Ordering::SeqCst),1);
 }
 assert_eq!(wake.0.load(Ordering::SeqCst),3);
}
"#);
 support::success(&compiled);let output=support::run(&mut Command::new(binary),10);support::success(&output);assert!(output.stdout.is_empty());
}

#[test]
fn backend_rejects_invalid_suspended_loans_and_thread_transfer(){
 for (source,code) in [
  ("async fn read(value:&str)->usize{std::future::pending::<()>().await;value.len()}fn main(){let owner=String::from(\"owner\");let future=read(&owner);drop(owner);drop(future);}","E0505"),
  ("fn require_send<T:Send>(_:T){}fn main(){let future=async{let value=std::rc::Rc::new(7);std::future::pending::<()>().await;*value};require_send(future);}","cannot be sent between threads safely"),
 ] {
  let (_directory,output,_)=compile(source);assert!(!output.status.success());
  let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains(code),"{stderr}");
 }
}
