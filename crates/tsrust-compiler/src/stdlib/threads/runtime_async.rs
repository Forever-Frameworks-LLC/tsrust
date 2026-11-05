    #[derive(Default)]
    struct Publication(Mutex<Option<tokio::sync::oneshot::Sender<()>>>);
    impl Publication {fn publish(&self){if let Some(send)=self.0.lock().unwrap().take(){let _=send.send(());}}}
    /// A task retains only admission identity until it is first polled.
    pub struct PoolSubmission { inner: Weak<Pool> }
    impl OwnedPool {pub fn submission(&self)->PoolSubmission{PoolSubmission{inner:Arc::downgrade(&self.inner)}}}
    pub enum AsyncTaskError<E>{Admission(SpawnError),Completion(JoinError<E>)}
    struct Pending<T:Send+'static,E:Send+'static>{thread:Option<Thread<T,E>>,ready:Option<tokio::sync::oneshot::Receiver<()>>}
    fn collect_ready<T,E>(thread:Thread<T,E>)->Result<T,JoinError<E>>{
        // Notification follows result publication. No blocking wait or OS join.
        debug_assert!(thread.completion.result.lock().unwrap().is_some());
        debug_assert!(thread.dedicated.is_none());join(thread)
    }
    impl<T:Send+'static,E:Send+'static> Drop for Pending<T,E>{fn drop(&mut self){
        if let(Some(thread),Some(ready))=(self.thread.take(),self.ready.take()){
            cancel(&thread);
            crate::tsrust_native_futures::cleanup(async move{let _=ready.await;drop(collect_ready(thread));});
        }
    }}
    pub async fn task_async<T:Send+'static,E:Send+'static>(capability:PoolSubmission,work:impl FnOnce()->Result<T,E>+Send+'static)->Result<T,AsyncTaskError<E>>{
        let pool=capability.inner.upgrade().ok_or_else(||AsyncTaskError::Admission(failure("closed","The explicit pool owner has been dropped.")))?;
        let thread=submit(pool,work).map_err(AsyncTaskError::Admission)?;
        let(send,ready)=tokio::sync::oneshot::channel();
        {let result=thread.completion.result.lock().unwrap();
         if result.is_some(){let _=send.send(());}else{*thread.completion.publication.0.lock().unwrap()=Some(send);}}
        let mut guard=Pending{thread:Some(thread),ready:Some(ready)};
        let _=guard.ready.as_mut().unwrap().await;guard.ready.take();
        collect_ready(guard.thread.take().unwrap()).map_err(AsyncTaskError::Completion)
    }
