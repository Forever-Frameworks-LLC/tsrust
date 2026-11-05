//! Runtime ownership witnesses independent of generated source or callback inference.
include!("../src/stdlib/futures/runtime.rs");
mod tsrust_native_threads {
 include!("../src/stdlib/threads/runtime_core.rs");
 include!("../src/stdlib/threads/runtime_async.rs");
}
use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
struct Payload(Arc<AtomicUsize>);
impl Drop for Payload{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
#[test]
fn task_async_lazy_full_closed_and_cancel_before_start_release_each_owner_once(){
 tsrust_native_futures::block_on(async {
  let drops=Arc::new(AtomicUsize::new(0));let pool=tsrust_native_threads::create_pool(1,2).unwrap();
  let payload=Payload(drops.clone());let lazy=tsrust_native_threads::task_async(pool.submission(),move||{drop(payload);Ok::<_,()>(())});
  assert_eq!(pool.stats().outstanding,0);drop(lazy);assert_eq!(drops.load(Ordering::SeqCst),1);
  let(release,wait)=std::sync::mpsc::channel();let(started,observed)=std::sync::mpsc::channel();
  let first=pool.task(move||{started.send(()).unwrap();wait.recv().unwrap();Ok::<_,()>(())}).unwrap();
  observed.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
  let payload=Payload(drops.clone());
  let task=tsrust_native_threads::task_async(pool.submission(),move||{drop(payload);panic!("cancelled callback ran");#[allow(unreachable_code)]Ok::<_,()>(())});
  let mut task=Box::pin(task);
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(task.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
  let payload=Payload(drops.clone());
  assert!(matches!(tsrust_native_threads::task_async(pool.submission(),move||{drop(payload);Ok::<_,()>(())}).await,Err(tsrust_native_threads::AsyncTaskError::Admission(error))if error.code=="full"));
  assert_eq!(drops.load(Ordering::SeqCst),2);drop(task);release.send(()).unwrap();
  assert!(tsrust_native_threads::join(first).is_ok());
  // The async scope owns cleanup for the cancelled accepted callback.
  let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
  while pool.stats().outstanding!=0{assert!(std::time::Instant::now()<deadline);tokio::task::yield_now().await;}
  assert_eq!(drops.load(Ordering::SeqCst),3);
  let capability=pool.submission();drop(pool);
  let payload=Payload(drops.clone());assert!(matches!(tsrust_native_threads::task_async(capability,move||{drop(payload);Ok::<_,()>(())}).await,Err(tsrust_native_threads::AsyncTaskError::Admission(error))if error.code=="closed"));
  assert_eq!(drops.load(Ordering::SeqCst),4);
 });
}
#[test]
fn task_async_completion_moves_results_errors_and_cancellation_waits_actual_work(){
 tsrust_native_futures::block_on(async {
  let pool=tsrust_native_threads::create_pool(1,2).unwrap();
  let value=vec![1,2];let pointer=value.as_ptr();
  let returned=match tsrust_native_threads::task_async(pool.submission(),move||Ok::<_,()>(value)).await{Ok(value)=>value,_=>panic!("completion lost")};assert_eq!(returned.as_ptr(),pointer);
  let error=vec![3,4];let pointer=error.as_ptr();
  match tsrust_native_threads::task_async(pool.submission(),move||Err::<(),_>(error)).await{Err(tsrust_native_threads::AsyncTaskError::Completion(tsrust_native_threads::JoinError::Worker(error)))=>assert_eq!(error.as_ptr(),pointer),_=>panic!("typed failure lost")}
  let finished=Arc::new(AtomicUsize::new(0));let count=finished.clone();let(started,observed)=tokio::sync::oneshot::channel();
  let capability=pool.submission();
  let job=tsrust_native_futures::start(async move{tsrust_native_threads::task_async(capability,move||{let _=started.send(());while !tsrust_native_threads::cancellation_requested(){std::thread::yield_now();}count.store(1,Ordering::SeqCst);Ok::<_,()>(())}).await.map_err(tsrust_native_futures::Failure::Worker)});
  observed.await.unwrap();job.cancel();assert!(matches!(job.join().await,Err(tsrust_native_futures::Failure::Cancelled)));assert_eq!(finished.load(Ordering::SeqCst),1);
 });
}
