#![allow(dead_code)]
#![forbid(unsafe_code)]
include!("reference.rs");
include!("../../../../crates/tsrust-compiler/src/stdlib/futures/runtime.rs");
mod tsrust_native_threads{
 include!("../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_core.rs");
 include!("../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_async.rs");
}
use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
use std::future::Future;
use std::task::Poll;
struct Owner{bytes:Vec<u8>,drops:Arc<AtomicUsize>}
impl Drop for Owner{fn drop(&mut self){self.drops.fetch_add(1,Ordering::SeqCst);}}
async fn poll_pending<F:Future>(future:std::pin::Pin<&mut F>){let mut future=future;std::future::poll_fn(|context|{assert!(future.as_mut().poll(context).is_pending());Poll::Ready(())}).await;}
async fn contracts(){
 let scope=Arc::new(tsrust_native_futures::Scope::default());
 tsrust_native_futures::in_scope(scope.clone(),async{
  let pool=tsrust_native_threads::create_pool(1,2).unwrap();
  let lazy_drops=Arc::new(AtomicUsize::new(0));let payload=Owner{bytes:input(16384,true,7),drops:lazy_drops.clone()};
  let lazy=tsrust_native_threads::task_async(pool.submission(),move||{let result=compress(Format::Gzip,&payload.bytes,-1,64,Strategy::Writer);drop(payload);result});
  assert_eq!(pool.stats().outstanding,0);drop(lazy);assert_eq!(lazy_drops.load(Ordering::SeqCst),1);
  let active_drops=Arc::new(AtomicUsize::new(0));let payload=Owner{bytes:input(16384,false,7),drops:active_drops.clone()};
  let(started,observed)=tokio::sync::oneshot::channel();let(release,wait)=std::sync::mpsc::channel();
  let mut active=Box::pin(tsrust_native_threads::task_async(pool.submission(),move||{let _=started.send(());wait.recv().unwrap();let result=compress(Format::Gzip,&payload.bytes,-1,64,Strategy::Writer);drop(payload);result}));
  poll_pending(active.as_mut()).await;observed.await.unwrap();
  let queued_drops=Arc::new(AtomicUsize::new(0));let queued_runs=Arc::new(AtomicUsize::new(0));let payload=Owner{bytes:input(16384,true,7),drops:queued_drops.clone()};let runs=queued_runs.clone();
  let mut queued=Box::pin(tsrust_native_threads::task_async(pool.submission(),move||{runs.fetch_add(1,Ordering::SeqCst);let result=compress(Format::Zlib,&payload.bytes,-1,64,Strategy::Writer);drop(payload);result}));
  poll_pending(queued.as_mut()).await;assert_eq!(pool.stats().outstanding,2);
  let full=tsrust_native_threads::task_async(pool.submission(),||Ok::<_,io::Error>(Vec::<u8>::new())).await;
  assert!(matches!(full,Err(tsrust_native_threads::AsyncTaskError::Admission(error))if error.code=="full"));
  drop(queued);drop(active);
  // Cancellation requests cleanup; it cannot claim started CPU work was aborted.
  assert_eq!(active_drops.load(Ordering::SeqCst),0);assert_eq!(pool.stats().running,1);
  release.send(()).unwrap();scope.drain(false).await;
  assert_eq!(active_drops.load(Ordering::SeqCst),1);assert_eq!(queued_drops.load(Ordering::SeqCst),1);assert_eq!(queued_runs.load(Ordering::SeqCst),0);assert_eq!(pool.stats().outstanding,0);
  let encoded=tsrust_native_threads::task_async(pool.submission(),||compress(Format::Gzip,b"owned result",-1,1,Strategy::Writer)).await;
  let Ok(encoded)=encoded else{panic!("pool compression failed")};assert_eq!(decompress(Format::Gzip,&encoded,1,None).unwrap(),b"owned result");
  let capability=pool.submission();drop(pool);
  assert!(matches!(tsrust_native_threads::task_async(capability,||Ok::<_,io::Error>(())).await,Err(tsrust_native_threads::AsyncTaskError::Admission(error))if error.code=="closed"));
 }).await;
 println!("{{\"kind\":\"pool-contracts\",\"lazyAdmission\":true,\"queuedOwnerDropOnce\":true,\"startedWorkDrained\":true,\"fullClosedTyped\":true,\"ownedOutput\":true}}");
}
fn main(){tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(contracts());}
