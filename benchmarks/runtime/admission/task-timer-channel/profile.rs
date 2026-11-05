// Admission only. Uninstrumented timing and instrumented allocation builds are
// separate; none of these probes is emitted production code or facade evidence.
include!("reference.rs");
use std::{hint::black_box,time::Instant};
#[cfg(profile_weak)] use tsrust_native_threads::task_async_weak_probe as selected_task;
#[cfg(not(profile_weak))] use tsrust_native_threads::task_async_retained_probe as selected_task;
#[cfg(profile_weak)] const CANDIDATE:&str="weak";
#[cfg(not(profile_weak))] const CANDIDATE:&str="strong";

#[cfg(allocation_probe)]
mod allocation {
    include!("../../probes/counting-allocator.rs");
    pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
    pub fn end(baseline:usize)->(usize,usize,usize,i64,i64){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),LIVE.load(Relaxed)as i64-baseline as i64,PEAK.load(Relaxed)as i64-baseline as i64)}
}
fn cpu(seed:u64,steps:usize)->u64{let mut value=black_box(seed);for _ in 0..steps{value=value.wrapping_mul(1664525).wrapping_add(1013904223)&0xffff_ffff;}black_box(value)}
async fn yield_value(value:u64)->Result<u64,Failure<()>>{tokio::task::yield_now().await;Ok(value)}
async fn kernel(name:&str,pool:&tsrust_native_threads::OwnedPool,closed:&tsrust_native_threads::OwnedPool)->u64{
    match name{
        "all_ready"=>all_probe((0..64).map(|value|std::future::ready(Ok::<_,Failure<()>>(value))).collect()).await.unwrap_or_else(|_|panic!("all")).into_iter().sum(),
        "ready_floor"=>{let inputs=(0..64).map(std::future::ready).collect::<Vec<_>>();let mut output=Vec::with_capacity(64);for value in inputs{output.push(value.await);}output.into_iter().sum()},
        "all_yield"=>all_probe((0..64).map(yield_value).collect()).await.unwrap_or_else(|_|panic!("all")).into_iter().sum(),
        "joinset_floor"=>{let mut set=tokio::task::JoinSet::new();for value in 0..64u64{set.spawn(async move{tokio::task::yield_now().await;(value,value)});}let mut output=vec![0;64];while let Some(value)=set.join_next().await{let(index,value)=value.unwrap();output[index as usize]=value;}output.into_iter().sum()},
        "try_direct"=>{let(tx,mut rx)=tokio::sync::mpsc::channel(64);for value in 0..64u64{tx.try_send(value).unwrap();}(0..64).map(|_|rx.try_recv().unwrap()).sum()},
        "try_exclusive"=>{let(tx,rx)=tokio::sync::mpsc::channel(64);let mut rx=tokio::sync::Mutex::new(rx);for value in 0..64u64{tx.try_send(value).unwrap();}(0..64).map(|_|rx.get_mut().try_recv().unwrap()).sum()},
        "async_direct"=>{let(tx,mut rx)=tokio::sync::mpsc::channel(64);for value in 0..64u64{tx.send(value).await.unwrap();}let mut sum=0;for _ in 0..64{sum+=rx.recv().await.unwrap();}sum},
        "async_existing"=>{let channel=tsrust_native_futures::Channel::new(64).unwrap();for value in 0..64u64{channel.send(value).await.unwrap_or_else(|_|panic!("send"));}let mut sum=0;for _ in 0..64{sum+=channel.receive::<()>().await.unwrap_or_else(|_|panic!("receive"));}sum},
        "cap_construct"=>{drop(black_box(selected_task(black_box(pool),||Ok::<_,()>(7))));1},
        "cap_closed_poll"=>{let result=selected_task(black_box(closed),||->Result<u64,()>{panic!("closed work executed")}).await;assert!(matches!(result,Err(tsrust_native_threads::ProbeError::<()>::Admission(error))if error.code=="closed"));1},
        "cpu_async"=>{let mut sum=0;for value in 0..64{sum+=selected_task(pool,move||Ok::<_,()>(cpu(value,4096))).await.unwrap_or_else(|_|panic!("async CPU"));}sum},
        "cpu_join_floor"=>{let mut sum=0;for value in 0..64{let task=pool.task(move||Ok::<_,()>(cpu(value,4096))).unwrap_or_else(|_|panic!("submit"));sum+=tsrust_native_threads::join(task).unwrap_or_else(|_|panic!("join"));}sum},
        _=>panic!("unknown kernel"),
    }
}
async fn capability_contracts(){
    use tsrust_native_threads::{create_pool,pool_alive_probe,ProbeError};
    let pool=create_pool(1,1).unwrap_or_else(|_|panic!("pool"));let alive=pool_alive_probe(&pool);
    let task=selected_task(&pool,||Ok::<_,()>(11));let bytes=std::mem::size_of_val(&task);assert_eq!(pool.stats().outstanding,0);
    let job=tsrust_native_futures::start(async move{task.await.map_err(|_|Failure::Worker(()))});let started_result=job.join().await.unwrap_or_else(|_|panic!("start"));assert_eq!(started_result,11);assert_eq!(pool.stats().outstanding,0);
    let task=selected_task(&pool,||Ok::<_,()>(13));drop(pool);let retained=alive();assert_eq!(retained,CANDIDATE=="strong");
    let closed_after_owner_drop=matches!(task.await,Err(ProbeError::Admission(error))if error.code=="closed");assert!(closed_after_owner_drop);assert!(!alive());
    let pool=create_pool(1,1).unwrap_or_else(|_|panic!("pool"));let drops=Arc::new(AtomicUsize::new(0));
    for busy_after_cancel in [false,true]{
        let(begin,started)=tokio::sync::oneshot::channel();let count=drops.clone();
        let task=selected_task(&pool,move||{let _guard=Count(count);let _=begin.send(());while !tsrust_native_threads::cancellation_requested(){std::thread::yield_now();}if busy_after_cancel{black_box(cpu(19,2_000_000));}Ok::<_,()>(7)});
        let work=async{task.await.map_err(|_|Failure::Worker(()))};let stop=async{started.await.unwrap();Err::<u64,Failure<()>>(Failure::Cancelled)};
        let inputs:Vec<Pin<Box<dyn Future<Output=Result<u64,Failure<()>>>+'_>>>=vec![Box::pin(work),Box::pin(stop)];assert!(matches!(tsrust_native_futures::race(inputs).await,Err(Failure::Cancelled)));assert_eq!(pool.stats().outstanding,0);
    }
    let awaited_cleanup_drops=drops.load(Ordering::SeqCst);assert_eq!(awaited_cleanup_drops,2);
    println!("{{\"kind\":\"contracts\",\"candidate\":\"{CANDIDATE}\",\"future_bytes\":{bytes},\"retains_pool_after_owner_drop\":{retained},\"started_result\":{started_result},\"closed_after_owner_drop\":{closed_after_owner_drop},\"awaited_cleanup_drops\":{awaited_cleanup_drops}}}");
}
fn main(){
    let arguments=std::env::args().collect::<Vec<_>>();let check=arguments.iter().any(|value|value=="--check");let capability_only=arguments.iter().any(|value|value=="--capability-only");
    tsrust_native_futures::block_on(async{
        if check{capability_contracts().await;return;}
        let pool=tsrust_native_threads::create_pool(1,1).unwrap_or_else(|_|panic!("pool"));let closed=tsrust_native_threads::create_pool(1,1).unwrap_or_else(|_|panic!("pool"));closed.shutdown().unwrap_or_else(|_|panic!("close"));
        for(name,rounds)in[("ready_floor",64),("all_ready",64),("joinset_floor",16),("all_yield",16),("try_direct",128),("try_exclusive",128),("async_direct",32),("async_existing",32),("cap_construct",4096),("cap_closed_poll",4096),("cpu_join_floor",1),("cpu_async",1)]{
            if capability_only && !matches!(name,"cap_construct"|"cap_closed_poll"|"cpu_async"){continue;}
            #[cfg(allocation_probe)]{
                for _ in 0..8{black_box(kernel(name,&pool,&closed).await);}
                let baseline=allocation::begin();let checksum=black_box(kernel(name,&pool,&closed).await);let(allocations,reallocations,requested,retained,peak)=allocation::end(baseline);
                println!("{{\"kind\":\"allocations\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"checksum\":{checksum},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak}}}");
            }
            #[cfg(not(allocation_probe))]{
                let mut durations=Vec::with_capacity(31);let mut checksums=Vec::with_capacity(31);
                for sample in 0..39{let started=Instant::now();let mut checksum=0;for _ in 0..rounds{checksum+=black_box(kernel(name,&pool,&closed).await);}let elapsed=started.elapsed().as_nanos();if sample>=8{durations.push(elapsed);checksums.push(checksum);}}
                println!("{{\"kind\":\"timing\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"rounds\":{rounds},\"durations_ns\":{durations:?},\"checksums\":{checksums:?}}}");
            }
        }
    });
}
