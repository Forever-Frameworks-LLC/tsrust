// Admission prototype, not a facade implementation or an accepted feature gate.
include!(env!("TSRUST_B06_FUTURES_SOURCE"));
include!(env!("TSRUST_B06_THREADS_SOURCE"));
use std::{future::Future,pin::Pin,sync::{Arc,atomic::{AtomicUsize,Ordering}},task::Poll,time::Duration};
use tsrust_native_futures::{Failure,Scope};
type Work=Pin<Box<dyn Future<Output=Result<i64,Failure<()>>>+Send>>;
struct Count(Arc<AtomicUsize>);
impl Drop for Count{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
async fn all_probe<F:Future<Output=Result<T,Failure<E>>>,T,E>(inputs:Vec<F>)->Result<Vec<T>,Failure<E>>{
    let scope=Scope::group();
    let result=tsrust_native_futures::in_scope(scope.clone(),async move{
        let mut inputs=inputs.into_iter().map(|value|Some(Box::pin(value))).collect::<Vec<_>>();
        let mut outputs=(0..inputs.len()).map(|_|None).collect::<Vec<_>>();
        std::future::poll_fn(|cx|{
            let mut complete=true;
            for(input,output)in inputs.iter_mut().zip(&mut outputs){if let Some(value)=input{match value.as_mut().poll(cx){
                Poll::Ready(Ok(value))=>{*output=Some(value);*input=None;},
                Poll::Ready(Err(error))=>return Poll::Ready(Err(error)),Poll::Pending=>complete=false,
            }}}
            if complete{Poll::Ready(Ok(outputs.iter_mut().map(|value|value.take().unwrap()).collect()))}else{Poll::Pending}
        }).await
    }).await;
    scope.drain(false).await;result
}
async fn tasks()->(Vec<i64>,usize,usize,usize){
    let inputs:Vec<Work>=vec![Box::pin(async{tokio::task::yield_now().await;Ok(3)}),Box::pin(async{Ok(5)})];
    let ordered=all_probe(inputs).await.unwrap_or_else(|_|panic!("all failed"));assert_eq!(ordered,[3,5]);
    let empty:Vec<std::future::Ready<Result<(),Failure<()>>>>=Vec::new();
    let empty_len=all_probe(empty).await.unwrap_or_else(|_|panic!("empty failed")).len();assert_eq!(empty_len,0);
    let voids=vec![std::future::ready(Ok::<_,Failure<()>>(())),std::future::ready(Ok(()))];
    let void_len=all_probe(voids).await.unwrap_or_else(|_|panic!("void failed")).len();assert_eq!(void_len,2);
    let drops=Arc::new(AtomicUsize::new(0));let guard=Count(drops.clone());
    let(ready,started)=tokio::sync::oneshot::channel();
    let sibling=tsrust_native_futures::start(async move{let _guard=guard;let _=ready.send(());std::future::pending::<()>().await;Ok::<_,Failure<()>>(9)});
    started.await.unwrap();
    let inputs:Vec<Work>=vec![Box::pin(std::future::ready(Err(Failure::Worker(())))),Box::pin(sibling.join())];
    assert!(matches!(all_probe(inputs).await,Err(Failure::Worker(()))));assert_eq!(drops.load(Ordering::SeqCst),1);
    (ordered,void_len,empty_len,drops.load(Ordering::SeqCst))
}
async fn timers()->Vec<String>{
    use tokio::time::{Instant,MissedTickBehavior,interval_at};
    let period=Duration::from_millis(20);let mut observed=Vec::new();
    for behavior in[MissedTickBehavior::Burst,MissedTickBehavior::Delay,MissedTickBehavior::Skip]{
        let start=Instant::now();let mut timer=interval_at(start,period);timer.set_missed_tick_behavior(behavior);
        assert_eq!(timer.tick().await,start);tokio::time::sleep(Duration::from_millis(70)).await;
        assert_eq!(timer.tick().await,start+period);let next=timer.tick().await;
        match behavior{
            MissedTickBehavior::Burst=>assert_eq!(next,start+period*2),
            MissedTickBehavior::Delay=>assert!(next>=start+Duration::from_millis(70)),
            MissedTickBehavior::Skip=>assert!(next>=start+Duration::from_millis(60)&&next.duration_since(start).as_nanos()%period.as_nanos()==0),
        }
        observed.push(format!("{behavior:?}").to_lowercase());
    }
    observed
}
async fn tokens()->bool{
    let parent=tokio_util::sync::CancellationToken::new();let shared=parent.clone();let child=parent.child_token();
    child.cancel();assert!(!parent.is_cancelled());assert!(!shared.is_cancelled());child.cancelled().await;
    parent.cancel();shared.cancelled().await;assert!(parent.child_token().is_cancelled());
    let independent=tokio_util::sync::CancellationToken::new();let retained=independent.clone();drop(independent);assert!(!retained.is_cancelled());
    child.is_cancelled()&&shared.is_cancelled()&&!retained.is_cancelled()
}
async fn channels()->bool{
    use tokio::sync::mpsc::{channel,error::{TryRecvError,TrySendError}};
    let(owner,mut receiver)=channel::<Vec<u8>>(1);let sender=owner.clone();let peer=sender.clone();drop(sender);
    assert!(matches!(receiver.try_recv(),Err(TryRecvError::Empty)));
    let first=vec![1,2];let pointer=first.as_ptr();peer.try_send(first).unwrap();
    let second=vec![3,4];let returned_pointer=second.as_ptr();
    let returned=match owner.try_send(second){Err(TrySendError::Full(value))=>value,_=>panic!("full")};assert_eq!(returned.as_ptr(),returned_pointer);
    let received=receiver.try_recv().unwrap();assert_eq!(received.as_ptr(),pointer);assert_eq!(received,[1,2]);
    receiver.close();let returned=match peer.try_send(returned){Err(TrySendError::Closed(value))=>value,_=>panic!("closed")};assert_eq!(returned.as_ptr(),returned_pointer);
    let retained=received.as_ptr()==pointer&&returned.as_ptr()==returned_pointer;
    assert!(matches!(receiver.try_recv(),Err(TryRecvError::Disconnected)));
    let(owner,mut receiver)=channel(1);let peer=owner.clone();drop(peer);owner.try_send(7).unwrap();assert_eq!(receiver.try_recv().unwrap(),7);
    drop(owner);assert!(matches!(receiver.try_recv(),Err(TryRecvError::Disconnected)));
    retained
}
async fn pools()->(usize,usize,usize){
    use tsrust_native_threads::{create_pool,task_async_probe,ProbeError};
    let pool=create_pool(1,1).unwrap_or_else(|error|panic!("{error}"));let drops=Arc::new(AtomicUsize::new(0));let guard=Count(drops.clone());
    let unpolled=task_async_probe(&pool,move||{drop(guard);Ok::<_,()>(3)});drop(unpolled);
    assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(pool.stats().outstanding,0);
    let unpolled_slots=pool.stats().outstanding;
    assert!(matches!(task_async_probe(&pool,||Ok::<_,()>(7)).await,Ok(7)));assert_eq!(pool.stats().outstanding,0);
    let(release,wait)=std::sync::mpsc::channel();let(begin,started)=tokio::sync::oneshot::channel();
    let mut occupied=Box::pin(task_async_probe(&pool,move||{let _=begin.send(());wait.recv().unwrap();Ok::<_,()>(0)}));
    std::future::poll_fn(|cx|match occupied.as_mut().poll(cx){Poll::Pending=>Poll::Ready(()),Poll::Ready(_)=>panic!("held worker")}).await;
    started.await.unwrap();let rejected=Count(drops.clone());
    assert!(matches!(task_async_probe(&pool,move||{drop(rejected);Ok::<_,()>(0)}).await,Err(ProbeError::Admission(error))if error.code=="full"));
    assert_eq!(drops.load(Ordering::SeqCst),2);release.send(()).unwrap();assert!(occupied.await.is_ok());
    for cooperative in[true,false]{
        let(begin,started)=tokio::sync::oneshot::channel();let count=drops.clone();
        let work=task_async_probe(&pool,move||{let _guard=Count(count);let _=begin.send(());
            if cooperative{while !tsrust_native_threads::cancellation_requested(){std::thread::yield_now();}}
            else{std::thread::sleep(Duration::from_millis(40));}Ok::<_,()>(9)});
        let completion=async{work.await.map_err(|_|Failure::Worker(()))};
        let cancellation=async{started.await.unwrap();Err::<i64,Failure<()>>(Failure::Cancelled)};
        let inputs:Vec<Pin<Box<dyn Future<Output=Result<i64,Failure<()>>>+'_>>>=vec![Box::pin(completion),Box::pin(cancellation)];
        assert!(matches!(tsrust_native_futures::race(inputs).await,Err(Failure::Cancelled)));assert_eq!(pool.stats().outstanding,0);
    }
    assert_eq!(drops.load(Ordering::SeqCst),4);
    pool.shutdown().unwrap_or_else(|error|panic!("{error}"));
    assert!(matches!(task_async_probe(&pool,||Ok::<_,()>(0)).await,Err(ProbeError::Admission(error))if error.code=="closed"));
    (unpolled_slots,pool.stats().outstanding,drops.load(Ordering::SeqCst))
}
async fn retained_pool_capability()->(bool,bool){
    use tsrust_native_threads::{create_pool,task_async_retained_probe,ProbeError};
    let pool=create_pool(1,1).unwrap_or_else(|error|panic!("{error}"));
    let task=task_async_retained_probe(&pool,||Ok::<_,()>(11));
    assert_eq!(pool.stats().outstanding,0);
    let job=tsrust_native_futures::start(async move{task.await.map_err(|_|Failure::Worker(()))});
    let started=matches!(job.join().await,Ok(11));assert!(started);
    assert_eq!(pool.stats().outstanding,0);
    drop(pool);

    let pool=create_pool(1,1).unwrap_or_else(|error|panic!("{error}"));
    let task=task_async_retained_probe(&pool,||Ok::<_,()>(13));
    assert_eq!(pool.stats().outstanding,0);
    drop(pool);
    let closed=matches!(task.await,Err(ProbeError::Admission(error))if error.code=="closed");assert!(closed);
    (started,closed)
}
#[cfg(not(profile_probe))]
fn main(){
    let(tasks,policies,token,channel,pool,retained)=tsrust_native_futures::block_on(async{(tasks().await,timers().await,tokens().await,channels().await,pools().await,retained_pool_capability().await)});
    println!("{{\"all_order\":{:?},\"all_void_len\":{},\"all_empty_len\":{},\"sibling_drops\":{},\"timer_policies\":{:?},\"token_direction\":{},\"send_owner_retained\":{},\"pool_unpolled_slots\":{},\"pool_cleanup_slots\":{},\"pool_drops\":{},\"pool_retained_start\":{},\"pool_owner_drop_closes\":{}}}",tasks.0,tasks.1,tasks.2,tasks.3,policies,token,channel,pool.0,pool.1,pool.2,retained.0,retained.1);
}
