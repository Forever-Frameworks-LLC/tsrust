#![allow(dead_code)]
include!(env!("TSRUST_TASK_GROUP_RUNTIME"));
include!(env!("TSRUST_TASK_GROUP_FLOOR_READY"));
include!(env!("TSRUST_TASK_GROUP_FLOOR_YIELD"));
include!(env!("TSRUST_TASK_GROUP_FLOOR_CHANNEL"));
use std::{future::Future,pin::Pin,task::Poll,hint::black_box,sync::{Arc,atomic::{AtomicUsize,Ordering}},cell::{Cell,RefCell},rc::Rc};
use tsrust_native_futures::Failure;
#[cfg(candidate_stack)]const CANDIDATE:&str="stack-four";
#[cfg(candidate_single)]const CANDIDATE:&str="single-box";
#[cfg(not(any(candidate_stack,candidate_single)))]const CANDIDATE:&str="double-box";
#[cfg(allocation_probe)]mod allocation {
    include!("../../probes/counting-allocator.rs");
    pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
    pub fn end(baseline:usize)->(usize,usize,usize,i64,i64){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),LIVE.load(Relaxed)as i64-baseline as i64,PEAK.load(Relaxed)as i64-baseline as i64)}
}
async fn group<'a,A,B,C,D,T:'a,E:'a>(a:A,b:B,c:C,d:D)->Result<Vec<T>,Failure<E>>
where A:Future<Output=Result<T,Failure<E>>>+'a,B:Future<Output=Result<T,Failure<E>>>+'a,C:Future<Output=Result<T,Failure<E>>>+'a,D:Future<Output=Result<T,Failure<E>>>+'a {
    #[cfg(candidate_stack)]{tsrust_native_futures::all_four(a,b,c,d).await}
    #[cfg(not(candidate_stack))]{
        let inputs:Vec<Pin<Box<dyn Future<Output=Result<T,Failure<E>>>+'a>>>=vec![Box::pin(a),Box::pin(b),Box::pin(c),Box::pin(d)];
        #[cfg(candidate_single)]{tsrust_native_futures::all_once(inputs).await}
        #[cfg(not(candidate_single))]{tsrust_native_futures::all(inputs).await}
    }
}
async fn owned_group<A,B,C,D,T,E>(a:A,b:B,c:C,d:D)->Result<Vec<T>,Failure<E>>
where A:Future<Output=Result<T,Failure<E>>>+Send+'static,B:Future<Output=Result<T,Failure<E>>>+Send+'static,C:Future<Output=Result<T,Failure<E>>>+Send+'static,D:Future<Output=Result<T,Failure<E>>>+Send+'static,T:Send+'static,E:Send+'static {
    #[cfg(candidate_stack)]{tsrust_native_futures::all_four(a,b,c,d).await}
    #[cfg(not(candidate_stack))]{
        let inputs:Vec<Pin<Box<dyn Future<Output=Result<T,Failure<E>>>+Send>>>=vec![Box::pin(a),Box::pin(b),Box::pin(c),Box::pin(d)];
        #[cfg(candidate_single)]{tsrust_native_futures::all_once(inputs).await}
        #[cfg(not(candidate_single))]{tsrust_native_futures::all(inputs).await}
    }
}
async fn value(value:i64,yielding:bool)->Result<i64,Failure<()>>{if yielding{tsrust_native_futures::yield_now::<()>().await?;}Ok(value)}
fn fold(sum:i64,value:i64)->i64{(sum*131+value)%1000000007}
async fn kernel(name:&str,n:i64,seed:i64)->i64{
    match name {
        "ready-floor"=>floor_ready::run(n,seed).await,
        "yield-floor"=>floor_yield::run(n,seed).await,
        "channel-floor"=>floor_channel::run(n,seed).await,
        "channel-runtime"=>{
            let mut queue=tsrust_native_futures::Channel::new(4).unwrap();let mut sender=queue.sender();let mut sum=0;
            for i in 0..n{for j in 0..4{assert!(sender.send(seed+i+j).await.is_ok());}for _ in 0..4{sum=fold(sum,queue.receive::<()>().await.ok().expect("receive"));}}
            sender.close();queue.close();sum
        },
        "ready"|"yield"=>{let mut sum=0;let yielding=name=="yield";for i in 0..n{
            let values=owned_group(async move{value(seed+i,yielding).await},async move{value(seed+i+1,yielding).await},async move{value(seed+i+2,yielding).await},async move{value(seed+i+3,yielding).await}).await.ok().expect("group");
            for value in values{sum=fold(sum,value);}
        }sum},
        _=>panic!("unknown task group kernel"),
    }
}

// Interior state allows a genuine !Unpin future without unsafe projection.
struct Borrowed<'a>{value:&'a Rc<Cell<i64>>,remaining:Cell<usize>,events:Rc<RefCell<Vec<i64>>>,_pin:std::marker::PhantomPinned}
impl Future for Borrowed<'_>{type Output=Result<i64,Failure<()>>;fn poll(self:Pin<&mut Self>,cx:&mut std::task::Context<'_>)->Poll<Self::Output>{let this=self.as_ref().get_ref();if this.remaining.get()>0{this.remaining.set(this.remaining.get()-1);cx.waker().wake_by_ref();Poll::Pending}else{let value=this.value.get();this.events.borrow_mut().push(value);Poll::Ready(Ok(value))}}}
impl Drop for Borrowed<'_>{fn drop(&mut self){self.events.borrow_mut().push(-self.value.get());}}
struct Count(Arc<AtomicUsize>);impl Drop for Count{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
struct CleanupOnDrop(Arc<AtomicUsize>);impl Drop for CleanupOnDrop{fn drop(&mut self){let count=self.0.clone();tsrust_native_futures::cleanup(async move{tokio::task::yield_now().await;count.fetch_add(1,Ordering::SeqCst);});}}
async fn never<T>()->Result<T,Failure<()>>{std::future::pending().await}
async fn contracts(){
    // Borrowed Rc inputs prove non-Send and !Unpin support, reverse completion,
    // exact ordered output and immediate completed-future destruction.
    let values=[Rc::new(Cell::new(11)),Rc::new(Cell::new(22)),Rc::new(Cell::new(33)),Rc::new(Cell::new(44))];let events=Rc::new(RefCell::new(Vec::new()));
    let make=|index,delay|Borrowed{value:&values[index],remaining:Cell::new(delay),events:events.clone(),_pin:std::marker::PhantomPinned};
    let output=group(make(0,3),make(1,1),make(2,2),make(3,0)).await.ok().expect("borrowed group");assert_eq!(output,[11,22,33,44]);assert_eq!(*events.borrow(),[44,-44,22,-22,33,-33,11,-11]);
    let output_drops=Arc::new(AtomicUsize::new(0));let sibling_drops=Arc::new(AtomicUsize::new(0));let cleanup=Arc::new(AtomicUsize::new(0));
    let output=Count(output_drops.clone());let pending=Count(sibling_drops.clone());let cleanup_guard=CleanupOnDrop(cleanup.clone());let unpolled_drops=Arc::new(AtomicUsize::new(0));let unpolled=Count(unpolled_drops.clone());
    let result=group(async move{Ok::<_,Failure<()>>(output)},async move{let _guard=pending;let _cleanup=cleanup_guard;never::<Count>().await},async{Err::<Count,_>(Failure::Worker(()))},async move{let _guard=unpolled;panic!("polled after the first observed failure");#[allow(unreachable_code)]never::<Count>().await}).await;
    assert!(matches!(result,Err(Failure::Worker(()))));assert_eq!(output_drops.load(Ordering::SeqCst),1);assert_eq!(sibling_drops.load(Ordering::SeqCst),1);assert_eq!(unpolled_drops.load(Ordering::SeqCst),1);assert_eq!(cleanup.load(Ordering::SeqCst),1);
    // Failure must cancel and await an already-started nested sibling and cleanup.
    let drops=Arc::new(AtomicUsize::new(0));let cleaned=Arc::new(AtomicUsize::new(0));let guard=Count(drops.clone());let clean=cleaned.clone();let(send,started)=tokio::sync::oneshot::channel();
    let result=group(async{started.await.expect("nested child started");Err::<i64,_>(Failure::Worker(()))},async move{
        let parent=tsrust_native_futures::start(async move{let child=tsrust_native_futures::start(async move{let _guard=guard;tsrust_native_futures::cleanup(async move{tokio::task::yield_now().await;clean.fetch_add(1,Ordering::SeqCst);});let _=send.send(());never::<i64>().await});child.join().await});parent.join().await
    },never(),never()).await;
    assert!(matches!(result,Err(Failure::Worker(()))));assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(cleaned.load(Ordering::SeqCst),1);
    // Returned Job handles escape the group and remain live under their parent.
    let drops=Arc::new(AtomicUsize::new(0));
    let spawn=||{let guard=Count(drops.clone());async move{Ok::<_,Failure<()>>(tsrust_native_futures::start(async move{let _guard=guard;never::<i64>().await}))}};
    let jobs=group(spawn(),spawn(),spawn(),spawn()).await.ok().expect("escaped jobs");assert_eq!(drops.load(Ordering::SeqCst),0);for job in jobs{assert!(job.cancel());assert!(matches!(job.join().await,Err(Failure::Cancelled)));}assert_eq!(drops.load(Ordering::SeqCst),4);
    // Send/static inputs must retain Send through the complete group future.
    let drops=Arc::new(AtomicUsize::new(0));let cleaned=Arc::new(AtomicUsize::new(0));let guard=Count(drops.clone());let clean=cleaned.clone();let(send,started)=tokio::sync::oneshot::channel();
    let worker=tsrust_native_futures::start(async move{owned_group(async move{let child=tsrust_native_futures::start(async move{let _guard=guard;tsrust_native_futures::cleanup(async move{tokio::task::yield_now().await;clean.fetch_add(1,Ordering::SeqCst);});let _=send.send(());never::<i64>().await});child.join().await},never(),never(),never()).await});
    started.await.expect("owned child started");assert!(worker.cancel());assert!(matches!(worker.join().await,Err(Failure::Cancelled)));assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(cleaned.load(Ordering::SeqCst),1);
    // A drain cancelled by its caller leaves cleanup reachable through parent scope.
    let outer=tsrust_native_futures::Scope::group();let(release,released)=tokio::sync::oneshot::channel();
    tsrust_native_futures::in_scope(outer.clone(),async {
        let mut pending=Box::pin(group(async move{tsrust_native_futures::cleanup(async move{released.await.unwrap();});Ok::<_,Failure<()>>(1)},async{Ok(2)},async{Ok(3)},async{Ok(4)}));
        std::future::poll_fn(|cx|{assert!(pending.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;drop(pending);
    }).await;
    let mut draining=Box::pin(outer.drain(false));std::future::poll_fn(|cx|{assert!(draining.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;release.send(()).unwrap();draining.await;
    println!("{{\"kind\":\"contracts\",\"candidate\":\"{CANDIDATE}\",\"ordered\":[11,22,33,44],\"borrowedNonSendNotUnpin\":true,\"immediateDrops\":true,\"failfastOutputDrops\":1,\"failfastSiblingDrops\":1,\"unpolledSiblingDrops\":1,\"nestedSiblingDrops\":1,\"escapedJobDrops\":4,\"cancelledGroupDrops\":1,\"cleanupAfterCancelledDrain\":true}}");
}
fn main(){
    let check=std::env::args().any(|arg|arg=="--check");
    tsrust_native_futures::block_on(async{
        if check{contracts().await;for name in ["ready","yield","ready-floor","yield-floor","channel-runtime","channel-floor"]{if CANDIDATE!="double-box" && !matches!(name,"ready"|"yield"){continue;}for(n,seed)in[(0,0),(1,7),(2,19),(17,3),(2048,7)]{let checksum=kernel(name,n,seed).await;println!("{{\"kind\":\"correctness\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"n\":{n},\"seed\":{seed},\"checksum\":{checksum}}}");}}return;}
        for name in ["ready","yield","ready-floor","yield-floor","channel-runtime","channel-floor"]{
            if CANDIDATE!="double-box" && !matches!(name,"ready"|"yield"){continue;}
            const N:i64=2048;const SEED:i64=7;
            for _ in 0..8{black_box(kernel(black_box(name),black_box(N),black_box(SEED)).await);}
            #[cfg(allocation_probe)]{
                let baseline=allocation::begin();let checksum=black_box(kernel(black_box(name),black_box(N),black_box(SEED)).await);let(allocations,reallocations,requested,retained,peak)=allocation::end(baseline);
                println!("{{\"kind\":\"allocation\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"n\":{N},\"seed\":{SEED},\"checksum\":{checksum},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak}}}");
            }
            #[cfg(not(allocation_probe))]{
                let mut durations=Vec::with_capacity(31);let mut checksums=Vec::with_capacity(31);
                for _ in 0..31{let start=std::time::Instant::now();let checksum=black_box(kernel(black_box(name),black_box(N),black_box(SEED)).await);durations.push(start.elapsed().as_nanos());checksums.push(checksum);}
                println!("{{\"kind\":\"timing\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"n\":{N},\"seed\":{SEED},\"durations_ns\":{durations:?},\"checksums\":{checksums:?}}}");
            }
        }
    });
}
