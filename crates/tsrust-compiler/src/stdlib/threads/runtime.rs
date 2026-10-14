mod tsrust_native_threads {
    use std::sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::sync::Weak;

    #[derive(Debug)]
    pub struct SpawnError { pub code: &'static str, message: String }
    impl std::fmt::Display for SpawnError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.message.fmt(f) }
    }
    fn failure(code: &'static str, message: impl Into<String>) -> SpawnError { SpawnError { code, message: message.into() } }
    pub enum JoinError<E> { Worker(E), Panic, Cancelled }
    #[derive(Clone, Copy, Default)]
    pub struct Stats { pub workers: usize, pub capacity: usize, pub queued: usize, pub running: usize, pub completed: usize, pub outstanding: usize }
    struct Pool { executor: rayon::ThreadPool, state: Arc<Mutex<Stats>>, idle: Condvar, closed: AtomicBool }
    trait Wake: Send + Sync { fn wake(&self); }
    #[derive(Default)]
    struct Cancellation { requested: AtomicBool, waiters: Mutex<Vec<Weak<dyn Wake>>> }
    impl Cancellation {
        fn load(&self, order: Ordering) -> bool { self.requested.load(order) }
        fn wake(&self) {
            let waiters = self.waiters.lock().unwrap().iter().filter_map(Weak::upgrade).collect::<Vec<_>>();
            for waiter in waiters { waiter.wake(); }
        }
    }
    pub struct TaskContext { cancellation: Arc<Cancellation> }
    impl TaskContext {
        pub fn current() -> Self { Self { cancellation: CANCELLATION.with(|slot| slot.borrow().as_ref().expect("worker context").clone()) } }
        pub fn cancellation_requested(&self) -> bool { self.cancellation.load(Ordering::Acquire) }
    }
    fn watch(waiter: Arc<dyn Wake>) {
        CANCELLATION.with(|slot| {
            if let Some(cancel) = slot.borrow().as_ref() {
                let mut waiters = cancel.waiters.lock().unwrap();
                waiters.retain(|entry| entry.strong_count() > 0);
                if !waiters.iter().any(|entry| entry.ptr_eq(&Arc::downgrade(&waiter))) { waiters.push(Arc::downgrade(&waiter)); }
            }
        });
    }
    thread_local! { static CANCELLATION: RefCell<Option<Arc<Cancellation>>> = const { RefCell::new(None) }; }
    fn build(workers: usize, capacity: usize) -> Result<Arc<Pool>, SpawnError> {
        let executor = rayon::ThreadPoolBuilder::new().num_threads(workers).build().map_err(|e| failure("spawn", e.to_string()))?;
        Ok(Arc::new(Pool { executor, state: Arc::new(Mutex::new(Stats { workers, capacity, ..Stats::default() })), idle: Condvar::new(), closed: AtomicBool::new(false) }))
    }
    fn default_workers() -> usize { std::thread::available_parallelism().map_or(2, |n| n.get().clamp(2, 256)) }
    fn validate_limits(workers: i64, capacity: i64) -> Result<(), SpawnError> {
        if !(1..=256).contains(&workers) || !(1..=1_000_000).contains(&capacity) { return Err(failure("range", "workers must be 1..256 and capacity 1..1000000.")); }
        Ok(())
    }
    pub struct OwnedPool { inner: Arc<Pool> }
    pub fn create_pool(workers: i64, capacity: i64) -> Result<OwnedPool, SpawnError> {
        validate_limits(workers, capacity)?;
        Ok(OwnedPool { inner: build(workers as usize, capacity as usize)? })
    }
    pub fn create_default_pool() -> Result<OwnedPool, SpawnError> { create_pool(default_workers() as i64, 1024) }
    fn close(pool: &Pool) {
        let mut stats = pool.state.lock().unwrap();
        pool.closed.store(true, Ordering::Release);
        while stats.queued + stats.running != 0 { stats = pool.idle.wait(stats).unwrap(); }
    }
    impl OwnedPool {
        pub fn task<T: Send + 'static, E: Send + 'static>(&self, work: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<Thread<T, E>, SpawnError> { submit(self.inner.clone(), work) }
        pub fn stats(&self) -> Stats { *self.inner.state.lock().unwrap() }
        pub fn shutdown(&self) -> Result<(), SpawnError> {
            if self.inner.executor.current_thread_index().is_some() { return Err(failure("worker", "A worker cannot shut down its own pool.")); }
            close(&self.inner); Ok(())
        }
    }
    impl Drop for OwnedPool { fn drop(&mut self) { close(&self.inner); } }
    struct Completion<T, E> { result: Mutex<Option<Result<T, JoinError<E>>>>, ready: Condvar, cancel: Arc<Cancellation>, released: AtomicBool, pool: Option<Arc<Pool>> }
    impl<T, E> Drop for Completion<T, E> {
        fn drop(&mut self) { self.release(); }
    }
    impl<T, E> Completion<T, E> {
        fn release(&self) {
            if self.released.swap(true, Ordering::AcqRel) { return; }
            if let Some(pool) = &self.pool {
                let mut stats = pool.state.lock().unwrap();
                stats.outstanding -= 1; stats.completed -= 1;
            }
        }
    }
    pub struct Thread<T, E> { completion: Arc<Completion<T, E>>, dedicated: Option<std::thread::JoinHandle<()>>, settled: bool }
    impl<T, E> Drop for Thread<T, E> {
        fn drop(&mut self) { if !self.settled { let _ = self.wait(); } }
    }
    impl<T, E> Thread<T, E> {
        fn wait(&mut self) -> Result<T, JoinError<E>> {
            let mut result = self.completion.result.lock().unwrap();
            while result.is_none() {
                // Nested joins must let the same pool execute their queued children.
                if self.completion.pool.as_ref().is_some_and(|p| p.executor.current_thread_index().is_some()) {
                    drop(result);
                    let executed = self.completion.pool.as_ref().unwrap().executor.yield_now();
                    result = self.completion.result.lock().unwrap();
                    if result.is_none() && !matches!(executed, Some(rayon::Yield::Executed)) {
                        result = self.completion.ready.wait_timeout(result, std::time::Duration::from_millis(1)).unwrap().0;
                    }
                } else { result = self.completion.ready.wait(result).unwrap(); }
            }
            let result = result.take().unwrap();
            if let Some(worker) = self.dedicated.take() { let _ = worker.join(); }
            self.completion.release(); self.settled = true; result
        }
    }
    pub fn cancellation_requested() -> bool { CANCELLATION.with(|slot| slot.borrow().as_ref().is_some_and(|flag| flag.load(Ordering::Acquire))) }
    pub fn cancel<T, E>(thread: &Thread<T, E>) -> bool {
        let result = thread.completion.result.lock().unwrap();
        if result.is_some() { return false; }
        let requested = !thread.completion.cancel.requested.swap(true, Ordering::AcqRel);
        drop(result);
        if requested { thread.completion.cancel.wake(); }
        requested
    }
    fn execute<T, E>(completion: &Completion<T, E>, work: impl FnOnce() -> Result<T, E>) {
        if let Some(pool) = &completion.pool { let mut stats = pool.state.lock().unwrap(); stats.queued -= 1; stats.running += 1; }
        let previous = CANCELLATION.with(|slot| slot.replace(Some(completion.cancel.clone())));
        let result = if completion.cancel.load(Ordering::Acquire) { drop(work); Err(JoinError::Cancelled) } else {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {
                Ok(Ok(value)) => Ok(value), Ok(Err(error)) => Err(JoinError::Worker(error)), Err(_) => Err(JoinError::Panic),
            }
        };
        CANCELLATION.with(|slot| { slot.replace(previous); });
        let mut stored = completion.result.lock().unwrap();
        *stored = Some(if completion.cancel.load(Ordering::Acquire) { drop(result); Err(JoinError::Cancelled) } else { result });
        if let Some(pool) = &completion.pool { let mut stats = pool.state.lock().unwrap(); stats.running -= 1; stats.completed += 1; pool.idle.notify_all(); }
        completion.ready.notify_all();
    }
    fn completion<T, E>(pool: Option<Arc<Pool>>) -> Arc<Completion<T, E>> {
        Arc::new(Completion { result: Mutex::new(None), ready: Condvar::new(), cancel: Arc::new(Cancellation::default()), released: AtomicBool::new(false), pool })
    }
    fn submit<T: Send + 'static, E: Send + 'static>(pool: Arc<Pool>, work: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<Thread<T, E>, SpawnError> {
        {
            let mut stats = pool.state.lock().unwrap();
            if pool.closed.load(Ordering::Acquire) { return Err(failure("closed", "The worker pool is shut down.")); }
            if stats.outstanding == stats.capacity { return Err(failure("full", "The pool's outstanding task capacity is full. Join or release a task before submitting more.")); }
            stats.outstanding += 1; stats.queued += 1;
        }
        let completion = completion(Some(pool.clone())); let worker = completion.clone();
        pool.executor.spawn(move || execute(&worker, work));
        Ok(Thread { completion, dedicated: None, settled: false })
    }
    pub fn spawn_dedicated<T: Send + 'static, E: Send + 'static>(work: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<Thread<T, E>, SpawnError> {
        let completion = completion(None); let worker = completion.clone();
        let dedicated = std::thread::Builder::new().spawn(move || execute(&worker, work)).map_err(|e| failure("spawn", e.to_string()))?;
        Ok(Thread { completion, dedicated: Some(dedicated), settled: false })
    }
    pub fn join<T, E>(mut thread: Thread<T, E>) -> Result<T, JoinError<E>> { thread.wait() }
    pub fn detach<T, E>(mut thread: Thread<T, E>) { thread.settled = true; drop(thread.dedicated.take()); }
    pub struct SendError<T> { pub code: &'static str, pub value: T }
    struct ChannelState<T> { queue: VecDeque<T>, capacity: usize, senders: usize, closed: bool, receiver: bool }
    struct ChannelShared<T> { state: Mutex<ChannelState<T>>, changed: Condvar }
    impl<T: Send> Wake for ChannelShared<T> {
        fn wake(&self) {
            // Locking the predicate prevents a cancellation notification from racing the wait.
            let _state = self.state.lock().unwrap();
            self.changed.notify_all();
        }
    }
    pub struct Channel<T> { shared: Arc<ChannelShared<T>> }
    pub struct Sender<T> { shared: Arc<ChannelShared<T>>, open: bool }
    impl<T: Send + 'static> Channel<T> {
        pub fn new(capacity: i64) -> Result<Self, &'static str> {
            if !(1..=1_000_000).contains(&capacity) { return Err("Channel capacity must be 1..1000000."); }
            Ok(Self { shared: Arc::new(ChannelShared { state: Mutex::new(ChannelState { queue: VecDeque::new(), capacity: capacity as usize, senders: 0, closed: false, receiver: true }), changed: Condvar::new() }) })
        }
        pub fn sender(&self) -> Sender<T> {
            let mut state = self.shared.state.lock().unwrap();
            let open = !state.closed;
            if open { state.senders += 1; }
            Sender { shared: self.shared.clone(), open }
        }
        pub fn receive(&self, blocking: bool) -> Result<T, &'static str> {
            watch(self.shared.clone());
            let mut state = self.shared.state.lock().unwrap();
            loop {
                if cancellation_requested() { return Err("cancelled"); }
                if let Some(value) = state.queue.pop_front() { self.shared.changed.notify_all(); return Ok(value); }
                if state.closed || state.senders == 0 { state.closed = true; return Err("closed"); }
                if !blocking { return Err("empty"); }
                state = self.shared.changed.wait(state).unwrap();
            }
        }
        pub fn close(&self) {
            let mut state = self.shared.state.lock().unwrap();
            state.closed = true;
            self.shared.changed.notify_all();
        }
    }
    impl<T> Drop for Channel<T> {
        fn drop(&mut self) {
            let mut state = self.shared.state.lock().unwrap();
            state.receiver = false; state.closed = true;
            let queued = std::mem::take(&mut state.queue);
            self.shared.changed.notify_all();
            drop(state); drop(queued);
        }
    }
    impl<T: Send + 'static> Sender<T> {
        pub fn send(&self, value: T, blocking: bool) -> Result<(), SendError<T>> {
            watch(self.shared.clone());
            let mut state = self.shared.state.lock().unwrap();
            loop {
                let failure = if cancellation_requested() { Some("cancelled") }
                    else if !self.open || state.closed || !state.receiver { Some("closed") }
                    else if state.queue.len() < state.capacity { None }
                    else if !blocking { Some("full") }
                    else { state = self.shared.changed.wait(state).unwrap(); continue; };
                if let Some(code) = failure { return Err(SendError { code, value }); }
                state.queue.push_back(value); self.shared.changed.notify_all(); return Ok(());
            }
        }
        pub fn close(&mut self) {
            if !self.open { return; }
            self.open = false;
            let mut state = self.shared.state.lock().unwrap();
            state.senders -= 1;
            if state.senders == 0 { state.closed = true; }
            self.shared.changed.notify_all();
        }
    }
    impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
            if !self.open { return; }
            let mut state = self.shared.state.lock().unwrap();
            state.senders -= 1;
            if state.senders == 0 { state.closed = true; }
            self.shared.changed.notify_all();
        }
    }
    #[cfg(test)]
    mod channel_tests {
        use super::*;
        use std::sync::atomic::AtomicUsize;
        use std::time::Duration;
        #[test]
        fn backpressure_returns_original_payload_and_disconnect_drains() {
            let channel = Channel::new(1).unwrap(); let sender = channel.sender();
            let first = vec![1, 2, 3]; let pointer = first.as_ptr();
            assert!(sender.send(first, false).is_ok());
            let second = vec![4, 5]; let second_pointer = second.as_ptr();
            let failure = sender.send(second, false).err().unwrap();
            assert_eq!(failure.code, "full"); assert_eq!(failure.value.as_ptr(), second_pointer);
            drop(sender);
            let received = channel.receive(true).unwrap(); assert_eq!(received.as_ptr(), pointer);
            assert_eq!(channel.receive(true).unwrap_err(), "closed");
        }
        #[test]
        fn cancellation_wakes_receive_and_full_send() {
            for sending in [false, true] {
                let channel = Channel::new(1).unwrap(); let sender = channel.sender();
                if sending { assert!(sender.send(1, true).is_ok()); }
                let (started, observed) = std::sync::mpsc::channel();
                let (done, finished) = std::sync::mpsc::channel();
                let worker = if sending {
                    spawn_dedicated(move || { started.send(()).unwrap(); let error = sender.send(2, true).err().unwrap(); assert_eq!(error.code,"cancelled"); assert_eq!(error.value,2); done.send(()).unwrap(); Ok::<_, ()>(()) }).unwrap()
                } else {
                    spawn_dedicated(move || { started.send(()).unwrap(); assert_eq!(channel.receive(true).unwrap_err(),"cancelled"); done.send(()).unwrap(); Ok::<_, ()>(()) }).unwrap()
                };
                observed.recv_timeout(Duration::from_secs(5)).unwrap();
                let deadline=std::time::Instant::now()+Duration::from_secs(5);
                while worker.completion.cancel.waiters.lock().unwrap().is_empty() {
                    assert!(std::time::Instant::now()<deadline,"worker never registered its blocking channel wait");
                    std::thread::yield_now();
                }
                assert!(cancel(&worker));
                finished.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(matches!(join(worker), Err(JoinError::Cancelled)));
            }
        }
        #[test]
        fn endpoint_close_and_context_are_specific_to_their_owner() {
            let channel=Channel::<i64>::new(2).unwrap();
            let mut first=channel.sender(); let second=channel.sender();
            assert_eq!(channel.receive(false).unwrap_err(),"empty");
            first.close(); first.close();
            assert_eq!(first.send(1,false).err().unwrap().code,"closed");
            assert!(second.send(2,false).is_ok());
            channel.close(); channel.close();
            assert_eq!(channel.receive(true).unwrap(),2);
            assert_eq!(channel.receive(true).unwrap_err(),"closed");
            let worker=spawn_dedicated(|| { let context=TaskContext::current(); assert!(!context.cancellation_requested()); Ok::<_,()>(()) }).unwrap();
            assert!(join(worker).is_ok());
        }
        #[test]
        fn queued_and_failed_payloads_drop_once() {
            struct Payload(Arc<AtomicUsize>);
            impl Drop for Payload { fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); } }
            let count=Arc::new(AtomicUsize::new(0));
            let channel=Channel::new(1).unwrap(); let sender=channel.sender();
            assert!(sender.send(Payload(count.clone()),false).is_ok());
            drop(channel);
            assert_eq!(count.load(Ordering::SeqCst),1);
            let error=sender.send(Payload(count.clone()),true).err().unwrap();
            assert_eq!(error.code,"closed"); assert_eq!(count.load(Ordering::SeqCst),1);
            drop(error); drop(sender); assert_eq!(count.load(Ordering::SeqCst),2);
        }
    }
    #[cfg(test)]
    mod pool_tests {
        use super::*;
        use std::sync::mpsc;
        #[test]
        fn two_pool_workers_overlap_before_either_is_joined() {
            let pool = create_pool(2, 2).unwrap();
            let release = Arc::new(AtomicBool::new(false)); let (started, observed) = mpsc::channel();
            let callback = |signal: mpsc::Sender<std::thread::ThreadId>, release: Arc<AtomicBool>| move || {
                signal.send(std::thread::current().id()).unwrap();
                while !release.load(Ordering::Acquire) { std::thread::yield_now(); }
                Ok::<_, ()>(())
            };
            let first = pool.task(callback(started.clone(), release.clone())).unwrap();
            let second = pool.task(callback(started, release.clone())).unwrap();
            let a = observed.recv_timeout(std::time::Duration::from_secs(5));
            let b = observed.recv_timeout(std::time::Duration::from_secs(5));
            release.store(true, Ordering::Release);
            assert!(join(first).is_ok()); assert!(join(second).is_ok());
            assert_ne!(a.unwrap(), b.unwrap());
        }
        #[test]
        fn owned_pools_are_independent_and_release_all_scheduler_references() {
            for _ in 0..32 {
                let first = create_pool(1, 1).unwrap(); let second = create_pool(2, 4).unwrap();
                let first_weak = Arc::downgrade(&first.inner); let second_weak = Arc::downgrade(&second.inner);
                let task = first.task(|| Ok::<_, ()>(42)).unwrap();
                first.shutdown().unwrap(); first.shutdown().unwrap();
                assert_eq!(first.stats().completed, 1);
                assert!(matches!(first.task(|| Ok::<_, ()>(())), Err(SpawnError { code: "closed", .. })));
                let other = second.task(|| Ok::<_, ()>(7)).unwrap();
                drop(first); assert!(matches!(join(task), Ok(42)));
                drop(second); assert!(matches!(join(other), Ok(7)));
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                while first_weak.upgrade().is_some() || second_weak.upgrade().is_some() {
                    assert!(std::time::Instant::now() < deadline, "scheduler leaked"); std::thread::yield_now();
                }
            }
        }
        #[test]
        fn owned_drop_drains_detached_work_and_disposes_results_errors_and_captures() {
            use std::sync::atomic::AtomicUsize;
            struct Payload(Arc<AtomicUsize>);
            impl Drop for Payload { fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); } }
            let disposed = Arc::new(AtomicUsize::new(0));
            let pool = create_pool(1, 4).unwrap(); let weak = Arc::downgrade(&pool.inner);
            let output = Payload(disposed.clone()); let error = Payload(disposed.clone());
            let cancelled = Payload(disposed.clone()); let panicked = Payload(disposed.clone());
            let (release, wait) = mpsc::channel();
            let first = pool.task(move || { wait.recv().unwrap(); Ok::<_, ()>(output) }).unwrap();
            let failure = pool.task(move || Err::<(), _>(error)).unwrap();
            let skip = pool.task(move || { drop(cancelled); Ok::<_, ()>(()) }).unwrap();
            assert!(cancel(&skip));
            let panic = pool.task(move || -> Result<(), ()> { drop(panicked); panic!("witness"); }).unwrap();
            let rejected = Payload(disposed.clone());
            assert!(matches!(pool.task(move || Ok::<_, ()>(rejected)), Err(SpawnError { code: "full", .. })));
            detach(first); detach(failure); detach(skip); detach(panic);
            release.send(()).unwrap(); drop(pool);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while weak.upgrade().is_some() { assert!(std::time::Instant::now() < deadline); std::thread::yield_now(); }
            assert_eq!(disposed.load(Ordering::SeqCst), 5);
        }
        #[test]
        fn defaults_shutdown_and_fresh_instances_are_independent() {
            let pool = create_default_pool().unwrap();
            assert_eq!(pool.stats().workers, default_workers());
            assert_eq!(pool.stats().capacity, 1024);
            assert!(create_pool(0, 2).is_err());
            let worker = pool.task(|| Ok::<_, ()>(8)).unwrap();
            pool.shutdown().unwrap(); assert_eq!(pool.stats().completed, 1);
            assert!(matches!(pool.task(|| Ok::<_, ()>(())), Err(SpawnError { code: "closed", .. })));
            assert!(matches!(join(worker), Ok(8))); assert_eq!(pool.stats().outstanding, 0);
            pool.shutdown().unwrap();
            let fresh = create_default_pool().unwrap();
            assert!(matches!(join(fresh.task(|| Ok::<_, ()>(9)).unwrap()), Ok(9)));
        }
        #[test]
        fn bounded_admission_completion_accounting_and_nested_progress() {
            let pool = build(1, 2).unwrap();
            let (signal, wait) = mpsc::channel();
            let first = submit(pool.clone(), move || { wait.recv().unwrap(); Ok::<_, ()>(1) }).unwrap();
            let second = submit(pool.clone(), || Ok::<_, ()>(2)).unwrap();
            assert!(matches!(submit(pool.clone(), || Ok::<_, ()>(3)), Err(SpawnError { code: "full", .. })));
            signal.send(()).unwrap();
            assert!(matches!(join(first), Ok(1))); assert!(matches!(join(second), Ok(2)));
            assert_eq!(pool.state.lock().unwrap().outstanding, 0);
            let child_pool = pool.clone();
            let parent = submit(pool.clone(), move || {
                let child = submit(child_pool, || Ok::<_, ()>(42)).unwrap();
                Ok::<_, ()>(match join(child) { Ok(value) => value, _ => panic!("child failed") })
            }).unwrap();
            assert!(matches!(join(parent), Ok(42)));
            assert_eq!(pool.state.lock().unwrap().outstanding, 0);
        }
        #[test]
        fn completed_tasks_keep_capacity_until_collected_and_cancel_loses_after_completion() {
            let pool = build(1, 1).unwrap();
            let worker = submit(pool.clone(), || Ok::<_, ()>(7)).unwrap();
            let mut result = worker.completion.result.lock().unwrap();
            while result.is_none() { result = worker.completion.ready.wait(result).unwrap(); }
            drop(result);
            let stats = *pool.state.lock().unwrap();
            assert_eq!((stats.running, stats.completed, stats.outstanding), (0, 1, 1));
            assert!(!cancel(&worker));
            assert!(matches!(submit(pool.clone(), || Ok::<_, ()>(())), Err(SpawnError { code: "full", .. })));
            assert!(matches!(join(worker), Ok(7)));
            assert!(submit(pool, || Ok::<_, ()>(())).is_ok());
        }
        #[test]
        fn queued_cancellation_disposes_capture_once_without_invocation() {
            use std::sync::atomic::AtomicUsize;
            struct Payload(Arc<AtomicUsize>);
            impl Drop for Payload { fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); } }
            let pool = build(1, 2).unwrap();
            let (started, observed) = mpsc::channel(); let (release, wait) = mpsc::channel();
            let first = submit(pool.clone(), move || { started.send(()).unwrap(); wait.recv().unwrap(); Ok::<_, ()>(()) }).unwrap();
            observed.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            let disposed = Arc::new(AtomicUsize::new(0)); let payload = Payload(disposed.clone());
            let second = submit(pool.clone(), move || -> Result<(), ()> { drop(payload); panic!("cancelled callback ran"); }).unwrap();
            assert!(cancel(&second)); assert!(!cancel(&second));
            release.send(()).unwrap(); assert!(join(first).is_ok());
            assert!(matches!(join(second), Err(JoinError::Cancelled)));
            assert_eq!(disposed.load(Ordering::SeqCst), 1); assert_eq!(pool.state.lock().unwrap().outstanding, 0);
        }
        #[test]
        fn nested_cancellation_context_and_detached_capacity_are_preserved() {
            let pool = build(1, 2).unwrap();
            let child_pool = pool.clone(); let (started, observed) = mpsc::channel(); let (release, wait) = mpsc::channel();
            let parent = submit(pool.clone(), move || {
                let child = submit(child_pool, move || {
                    started.send(()).unwrap(); wait.recv().unwrap();
                    assert!(!cancellation_requested()); Ok::<_, ()>(())
                }).unwrap();
                assert!(join(child).is_ok()); assert!(cancellation_requested()); Ok::<_, ()>(())
            }).unwrap();
            observed.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(cancel(&parent)); release.send(()).unwrap();
            assert!(matches!(join(parent), Err(JoinError::Cancelled)));
            let (release, wait) = mpsc::channel();
            let detached = submit(pool.clone(), move || { wait.recv().unwrap(); Ok::<_, ()>(()) }).unwrap();
            detach(detached); assert_eq!(pool.state.lock().unwrap().outstanding, 1);
            release.send(()).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while pool.state.lock().unwrap().outstanding != 0 { assert!(std::time::Instant::now() < deadline); std::thread::yield_now(); }
        }
        #[test]
        fn running_dedicated_and_pooled_cancellation_keep_cleanup() {
            for dedicated in [true, false] {
                let pool = build(1, 1).unwrap(); let (started, observed) = mpsc::channel();
                let work = move || { started.send(()).unwrap(); while !cancellation_requested() { std::thread::yield_now(); } Ok::<_, ()>(99) };
                let worker = if dedicated { spawn_dedicated(work).unwrap() } else { submit(pool.clone(), work).unwrap() };
                observed.recv_timeout(std::time::Duration::from_secs(5)).unwrap(); assert!(cancel(&worker));
                assert!(matches!(join(worker), Err(JoinError::Cancelled)));
                assert_eq!(pool.state.lock().unwrap().outstanding, 0);
            }
            assert!(!cancellation_requested());
        }
    }
}
