use std::sync::{Arc, atomic::{AtomicUsize,Ordering}};
struct DropCount(Arc<AtomicUsize>);
impl Drop for DropCount{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
#[test]
fn terminal_test_boundary_drives_a_borrowed_non_send_future(){
    let value=std::rc::Rc::new(std::cell::Cell::new(0));
    testing_scope(||testing_await(async {
        assert!(yield_now::<()>().await.is_ok());
        value.set(7);
    }));
    assert_eq!(value.get(),7);
}
#[test]
fn cancel_waits_for_nested_owned_resources_and_preserves_completed_results(){
    block_on(async {
        let drops=Arc::new(AtomicUsize::new(0));let count=drops.clone();
        let (ready,started)=tokio::sync::oneshot::channel();
        let parent=start(async move {
            let guard=DropCount(count);let child=start(async move {let _guard=guard;let _=ready.send(());std::future::pending::<()>().await;Ok::<_,Failure<()>>(())});
            child.join().await
        });
        started.await.unwrap();assert!(parent.cancel());assert!(!parent.cancel());
        assert!(matches!(parent.join().await,Err(Failure::Cancelled)));
        assert_eq!(drops.load(Ordering::SeqCst),1);
        let done=start(async {Ok::<_,Failure<()>>(42)});
        assert_eq!(done.join().await.ok(),Some(42));
    });
}
#[test]
fn timeout_and_race_drain_losers_and_channels_apply_backpressure(){
    block_on(async {
        let drops=Arc::new(AtomicUsize::new(0));let guard=DropCount(drops.clone());
        let (ready,started)=tokio::sync::oneshot::channel();
        let job=start(async move {let _guard=guard;let _=ready.send(());std::future::pending::<()>().await;Ok::<_,Failure<()>>(())});
        started.await.unwrap();assert!(matches!(timeout(job.join(),0).await,Err(Failure::Timeout)));assert_eq!(drops.load(Ordering::SeqCst),1);
        let channel=Channel::new(1).unwrap();channel.send(1).await.ok().unwrap();
        let sender=channel.sender();let send=start(async move {sender.send(2).await.map_err(Failure::Worker)});
        assert_eq!(channel.receive::<()>().await.ok(),Some(1));assert!(send.join().await.is_ok());assert_eq!(channel.receive::<()>().await.ok(),Some(2));
        assert!(matches!(race::<std::future::Ready<Result<(),Failure<()>>>,_,_>(vec![]).await,Err(Failure::Empty)));
        let results=all_settled(vec![std::future::ready(Ok::<_,Failure<()>>(1)),std::future::ready(Err(Failure::Cancelled))]).await;
        assert_eq!(results.len(),2);assert!(matches!(results[0],Ok(1)));assert!(matches!(results[1],Err(Failure::Cancelled)));
    });
}
#[test]
fn unpolled_join_and_shutdown_release_once(){
    let drops=Arc::new(AtomicUsize::new(0));let count=drops.clone();
    block_on(async move {let guard=DropCount(count);let job=start(async move {let _guard=guard;std::future::pending::<()>().await;Ok::<_,Failure<()>>(())});drop(job.join());});
    assert_eq!(drops.load(Ordering::SeqCst),1);
}
#[test]
fn timers_closed_channels_and_worker_errors_keep_their_contracts(){
    block_on(async {
        assert!(delay::<()>(0).await.is_ok());
        assert!(matches!(delay::<()>(-1).await,Err(Failure::Range)));
        yield_now::<()>().await.unwrap_or_else(|_|panic!("yield failed"));
        let mut channel=Channel::new(1).unwrap();
        channel.send(7).await.unwrap_or_else(|_|panic!("send failed"));
        channel.close();
        assert!(matches!(channel.send(8).await,Err(SendError{code:"closed",value:8})));
        assert!(matches!(channel.receive::<()>().await,Ok(7)));
        assert!(matches!(channel.receive::<()>().await,Err(Failure::Closed)));
        let failed=start(async {Err::<(),_>(Failure::Worker(9))});
        assert!(matches!(failed.join().await.map_err(|error|error.map(|value|value+1)),Err(Failure::Worker(10))));
    });
}

#[test]
fn cancelling_parent_drains_children_inside_pending_groups(){
    block_on(async {
        let drops=Arc::new(AtomicUsize::new(0));let guard=DropCount(drops.clone());
        let (ready,started)=tokio::sync::oneshot::channel();
        let parent=start(async move {
            timeout(async move {
                let child=start(async move {let _guard=guard;let _=ready.send(());std::future::pending::<()>().await;Ok::<_,Failure<()>>(())});
                child.join().await
            },60000).await
        });
        started.await.unwrap();parent.cancel();
        assert!(matches!(parent.join().await,Err(Failure::Cancelled)));
        assert_eq!(drops.load(Ordering::SeqCst),1);
    });
}
#[test]
fn workers_overlap_and_repeated_completion_releases_owned_inputs(){
    block_on(async {
        let barrier=Arc::new(tokio::sync::Barrier::new(3));
        let left_gate=barrier.clone();let right_gate=barrier.clone();
        let left=start(async move{left_gate.wait().await;Ok::<_,Failure<()>>(1)});
        let right=start(async move{right_gate.wait().await;Ok::<_,Failure<()>>(2)});
        tokio::time::timeout(std::time::Duration::from_secs(5),barrier.wait()).await.expect("both workers must run before either is joined");
        assert!(matches!(left.join().await,Ok(1)));assert!(matches!(right.join().await,Ok(2)));
        let drops=Arc::new(AtomicUsize::new(0));
        for _ in 0..1000 {let guard=DropCount(drops.clone());let job=start(async move{let _guard=guard;Ok::<_,Failure<()>>(())});assert!(job.join().await.is_ok());}
        assert_eq!(drops.load(Ordering::SeqCst),1000);
    });
}
#[test]
fn returned_jobs_transfer_cleanup_responsibility_instead_of_being_cancelled(){
    block_on(async {
        let parent=start(async {Ok::<_,Failure<()>>(start(async {std::future::pending::<()>().await;Ok::<_,Failure<()>>(())}))});
        let child=parent.join().await.unwrap_or_else(|_|panic!("parent failed"));
        assert!(child.cancel());assert!(matches!(child.join().await,Err(Failure::Cancelled)));
        let child=timeout(async {Ok::<_,Failure<()>>(start(async {std::future::pending::<()>().await;Ok::<_,Failure<()>>(())}))},1000).await.unwrap_or_else(|_|panic!("deadline failed"));
        assert!(child.cancel());assert!(matches!(child.join().await,Err(Failure::Cancelled)));
    });
}
#[test]
fn race_failure_drains_started_loser_and_cancelled_send_releases_message(){
    block_on(async {
        let drops=Arc::new(AtomicUsize::new(0));let guard=DropCount(drops.clone());
        let loser=start(async move{let _guard=guard;std::future::pending::<()>().await;Ok::<_,Failure<()>>(1)});
        let inputs:Vec<std::pin::Pin<Box<dyn std::future::Future<Output=Result<i32,Failure<()>>>>>>=vec![Box::pin(std::future::ready(Err(Failure::Worker(())))),Box::pin(loser.join())];
        assert!(matches!(race(inputs).await,Err(Failure::Worker(()))));
        assert_eq!(drops.load(Ordering::SeqCst),1);
        let mut queue=Channel::new(1).unwrap();
        assert!(queue.send(DropCount(drops.clone())).await.is_ok());
        let sender=queue.sender();let message=DropCount(drops.clone());
        let (ready,started)=tokio::sync::oneshot::channel();
        let blocked=start(async move{let _=ready.send(());sender.send(message).await.map_err(Failure::Worker)});
        started.await.unwrap();blocked.cancel();assert!(matches!(blocked.join().await,Err(Failure::Cancelled)));
        assert_eq!(drops.load(Ordering::SeqCst),2);
        queue.close();drop(queue.receive::<()>().await);assert_eq!(drops.load(Ordering::SeqCst),3);
    });
}

#[test]
fn resource_cleanup_finishes_before_cancelled_job_join(){
 use std::sync::{Arc,atomic::{AtomicBool,Ordering}};
 let released=Arc::new(AtomicBool::new(false));let observed=released.clone();
 block_on(async move{
  let (ready,started)=tokio::sync::oneshot::channel();
  let worker=start(async move{
   cleanup(async move{tokio::time::sleep(std::time::Duration::from_millis(20)).await;released.store(true,Ordering::Release);});
   ready.send(()).unwrap();
   delay::<()>(60000).await
  });
  started.await.unwrap();worker.cancel();let _=worker.join().await;
  assert!(observed.load(Ordering::Acquire));
 });
}

#[test]
fn cancelled_nested_drain_retains_cleanup(){
 use tsrust_native_futures::*;
 block_on(async {
  let outer=Scope::group();
  let (release,released)=tokio::sync::oneshot::channel::<()>();
  in_scope(outer.clone(),async {
   let inner=Scope::group();
   in_scope(inner,async {cleanup(async move {released.await.unwrap();});}).await;
  }).await;
  let mut first=Box::pin(outer.drain(false));
  std::future::poll_fn(|cx| {assert!(std::future::Future::poll(first.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
  drop(first);
  let mut second=Box::pin(outer.drain(false));
  std::future::poll_fn(|cx| {assert!(std::future::Future::poll(second.as_mut(),cx).is_pending(),"nested cleanup was forgotten");std::task::Poll::Ready(())}).await;
  release.send(()).unwrap();second.await;
 });
}

#[test]
fn bounded_queue_pending_sends_preserve_identity_and_drop_only_the_cancelled_owner(){
 use std::{future::Future,task::Poll};
 block_on(async {
  struct Payload{bytes:Box<[u8]>,drops:Arc<AtomicUsize>}
  impl Drop for Payload{fn drop(&mut self){self.drops.fetch_add(1,Ordering::SeqCst);}}
  let drops=Arc::new(AtomicUsize::new(0));
  let mut queue=Channel::new(1).unwrap();
  let first=Payload{bytes:vec![1,2,3].into_boxed_slice(),drops:drops.clone()};
  let identity=first.bytes.as_ptr();
  assert!(queue.send(first).await.is_ok());
  let sender=queue.sender();
  let mut blocked=Box::pin(sender.send(Payload{bytes:vec![4].into_boxed_slice(),drops:drops.clone()}));
  assert!(std::future::poll_fn(|cx|Poll::Ready(matches!(blocked.as_mut().poll(cx),Poll::Pending))).await);
  assert_eq!(drops.load(Ordering::SeqCst),0);
  drop(blocked);
  assert_eq!(drops.load(Ordering::SeqCst),1);
  queue.close();
  let received=queue.receive::<()>().await.unwrap_or_else(|_|panic!("queued owner lost"));
  assert_eq!(received.bytes.as_ptr(),identity);
  assert_eq!(&*received.bytes,&[1,2,3]);
  drop(received);
  assert_eq!(drops.load(Ordering::SeqCst),2);
  assert!(matches!(queue.receive::<()>().await,Err(Failure::Closed)));
  let rejected=Payload{bytes:vec![5].into_boxed_slice(),drops:drops.clone()};
  assert!(matches!(sender.send(rejected).await,Err(SendError{code:"closed",..})));
  assert_eq!(drops.load(Ordering::SeqCst),3);
 });
}

#[test]
fn all_orders_unit_and_non_unpin_borrowed_results_and_drains_failure_siblings(){
 block_on(async {
  let cell=std::rc::Rc::new(std::cell::Cell::new(0));
  let value=&cell;
  let local=async {yield_now::<()>().await?;value.set(1);Ok::<_,Failure<()>>(7)};
  assert_eq!(all(vec![local]).await.ok(),Some(vec![7]));
  assert_eq!(cell.get(),1);
  assert_eq!(all::<std::future::Ready<Result<(),Failure<()>>>,_,_>(vec![]).await.ok(),Some(vec![]));
  assert_eq!(all(vec![std::future::ready(Ok::<_,Failure<()>>(())),std::future::ready(Ok(()))]).await.ok(),Some(vec![(),()]));
  let drops=Arc::new(AtomicUsize::new(0));let guard=DropCount(drops.clone());
  let(ready,started)=tokio::sync::oneshot::channel();
  let sibling=start(async move{let _guard=guard;let _=ready.send(());std::future::pending::<()>().await;Ok::<_,Failure<i64>>(1_i64)});
  started.await.unwrap();
  let sibling=Box::pin(sibling.join()) as std::pin::Pin<Box<dyn std::future::Future<Output=Result<i64,Failure<i64>>>>>;
  let failed=Box::pin(std::future::ready(Err(Failure::Worker(9_i64)))) as std::pin::Pin<Box<dyn std::future::Future<Output=Result<i64,Failure<i64>>>>>;
  assert!(matches!(all(vec![sibling,failed]).await,Err(Failure::Worker(9))));
  assert_eq!(drops.load(Ordering::SeqCst),1);
 });
}
#[test]
fn try_and_awaited_send_retain_exact_unsent_owners_and_sender_close_is_local(){
 block_on(async {
  let mut queue=Channel::new(1).unwrap();let mut sender=queue.sender();let other=queue.sender();
  assert!(matches!(queue.try_receive(),Err("empty")));
  let first=vec![1,2];let first_pointer=first.as_ptr();assert!(sender.try_send(first).is_ok());
  let full=vec![3,4];let full_pointer=full.as_ptr();let full=other.try_send(full).err().unwrap();
  assert_eq!(full.code,"full");assert_eq!(full.value.as_ptr(),full_pointer);
  let received=queue.try_receive().ok().unwrap();assert_eq!(received.as_ptr(),first_pointer);
  sender.close();sender.close();
  let closed=vec![5,6];let pointer=closed.as_ptr();let closed=sender.send(closed).await.err().unwrap();
  assert_eq!(closed.code,"closed");assert_eq!(closed.value.as_ptr(),pointer);
  assert!(other.try_send(vec![7]).is_ok());queue.close();assert_eq!(queue.try_receive().ok(),Some(vec![7]));
  assert!(matches!(queue.try_receive(),Err("closed")));
  let closed=vec![8];let pointer=closed.as_ptr();let closed=other.send(closed).await.err().unwrap();
  assert_eq!(closed.code,"closed");assert_eq!(closed.value.as_ptr(),pointer);
 });
}
#[test]
fn interval_constructor_checks_context_and_ticks_are_cancel_safe(){
 assert!(matches!(Interval::new(1,"burst"),Err("runtime")));
 block_on(async {
  assert!(matches!(Interval::new(0,"burst"),Err("range")));
  assert!(matches!(Interval::new(-1,"burst"),Err("range")));
  assert!(matches!(Interval::new(1,"unknown"),Err("range")));
  for policy in ["burst","delay","skip"]{
   let mut interval=Interval::new(60_000,policy).unwrap();
   assert!(timeout::<_,_,()>(async{interval.tick().await;Ok(())},1000).await.is_ok());
   let mut waiting=Box::pin(interval.tick());
   std::future::poll_fn(|cx|{assert!(std::future::Future::poll(waiting.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
   drop(waiting);
  }
 });
}

#[test]
fn actual_deadline_and_capacity_ranges_replace_old_policy_ceilings(){
 block_on(async {
  assert!(matches!(timeout(delay::<()>(31_536_000_001),0).await,Err(Failure::Timeout)));
  assert!(Channel::<()>::new(1_000_001).is_ok());
  assert!(Channel::<()>::new(0).is_err());assert!(Channel::<()>::new(-1).is_err());
  let above=tokio::sync::Semaphore::MAX_PERMITS.checked_add(1).and_then(|value|i64::try_from(value).ok());
  if let Some(above)=above{assert!(Channel::<()>::new(above).is_err());}
 });
}

#[test]
fn cancelled_drain_retains_abandoned_child_until_its_gated_cleanup_finishes(){
 struct ReleaseOnDrop(Option<tokio::sync::oneshot::Sender<()>>);
 impl ReleaseOnDrop {fn release(&mut self){if let Some(sender)=self.0.take(){let _=sender.send(());}}}
 impl Drop for ReleaseOnDrop {fn drop(&mut self){self.release();}}
 block_on(async {
  let outer=Scope::group();let(release,released)=tokio::sync::oneshot::channel();
  let(started,cleanup_started)=tokio::sync::oneshot::channel();
  in_scope(outer.clone(),async {
   let inner=Scope::group();
   in_scope(inner,async {
    let child=start(async move {
     cleanup(async move {let _=started.send(());let _=tokio::time::timeout(std::time::Duration::from_secs(5),released).await;});
     std::future::pending::<()>().await;Ok::<_,Failure<()>>(())
    });
    tokio::time::timeout(std::time::Duration::from_secs(5),cleanup_started).await.expect("child cleanup start deadline").unwrap();
    drop(child);
   }).await;
  }).await;
  let mut release=ReleaseOnDrop(Some(release));
  let mut first=Box::pin(outer.drain(true));
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(first.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
  drop(first);
  let mut second=Box::pin(outer.drain(true));
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(second.as_mut(),cx).is_pending(),"cancelled drain lost its abandoned child");std::task::Poll::Ready(())}).await;
  release.release();tokio::time::timeout(std::time::Duration::from_secs(5),second).await.expect("child drain cleanup deadline");
 });
}
#[test]
fn interval_wrapper_distinguishes_burst_delay_and_skip_after_missed_ticks(){
 let runtime=tokio::runtime::Builder::new_current_thread().enable_time().start_paused(true).build().unwrap();
 runtime.block_on(async {
  async fn ready(interval:&mut Interval)->bool{
   let mut tick=Box::pin(interval.tick());
   std::future::poll_fn(|cx|std::task::Poll::Ready(std::future::Future::poll(tick.as_mut(),cx).is_ready())).await
  }
  let mut burst=Interval::new(10,"burst").unwrap();let mut delay=Interval::new(10,"delay").unwrap();let mut skip=Interval::new(10,"skip").unwrap();
  burst.tick().await;delay.tick().await;skip.tick().await;
  tokio::time::advance(std::time::Duration::from_millis(35)).await;
  // Each consumes the overdue 10 ms tick; next deadlines become 20, 45 and 40 ms.
  let late=tokio::time::Instant::now();
  burst.tick().await;delay.tick().await;skip.tick().await;assert_eq!(tokio::time::Instant::now(),late);
  burst.tick().await;assert_eq!(tokio::time::Instant::now(),late);assert!(!ready(&mut delay).await);assert!(!ready(&mut skip).await);
  tokio::time::advance(std::time::Duration::from_millis(5)).await;
  let aligned=tokio::time::Instant::now();skip.tick().await;assert_eq!(tokio::time::Instant::now(),aligned);assert!(!ready(&mut delay).await);
  tokio::time::advance(std::time::Duration::from_millis(5)).await;
  let delayed=tokio::time::Instant::now();delay.tick().await;assert_eq!(tokio::time::Instant::now(),delayed);
 });
}
