use std::sync::{Arc, atomic::{AtomicUsize,Ordering}};
struct DropCount(Arc<AtomicUsize>);
impl Drop for DropCount{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
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
        let channel=Channel::new(1).unwrap();channel.send::<()>(1).await.ok().unwrap();
        let sender=channel.sender();let send=start(async move {sender.send::<()>(2).await});
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
        channel.send::<()>(7).await.unwrap_or_else(|_|panic!("send failed"));
        channel.close();
        assert!(matches!(channel.send::<()>(8).await,Err(Failure::Closed)));
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
        assert!(queue.send::<()>(DropCount(drops.clone())).await.is_ok());
        let sender=queue.sender();let message=DropCount(drops.clone());
        let (ready,started)=tokio::sync::oneshot::channel();
        let blocked=start(async move{let _=ready.send(());sender.send::<()>(message).await});
        started.await.unwrap();blocked.cancel();assert!(matches!(blocked.join().await,Err(Failure::Cancelled)));
        assert_eq!(drops.load(Ordering::SeqCst),2);
        queue.close();drop(queue.receive::<()>().await);assert_eq!(drops.load(Ordering::SeqCst),3);
    });
}
