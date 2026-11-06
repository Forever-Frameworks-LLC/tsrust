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
    // Private shared admission/ready setup; ordinary task_async keeps its API.
    fn pending_task<T:Send+'static,E:Send+'static>(capability:PoolSubmission,work:impl FnOnce()->Result<T,E>+Send+'static)->Result<Pending<T,E>,SpawnError>{
        let pool=capability.inner.upgrade().ok_or_else(||failure("closed","The explicit pool owner has been dropped."))?;
        let thread=submit(pool,work)?;
        let(send,ready)=tokio::sync::oneshot::channel();
        {let result=thread.completion.result.lock().unwrap();
         if result.is_some(){let _=send.send(());}else{*thread.completion.publication.0.lock().unwrap()=Some(send);}}
        Ok(Pending{thread:Some(thread),ready:Some(ready)})
    }
    pub async fn task_async<T:Send+'static,E:Send+'static>(capability:PoolSubmission,work:impl FnOnce()->Result<T,E>+Send+'static)->Result<T,AsyncTaskError<E>>{
        let mut guard=pending_task(capability,work).map_err(AsyncTaskError::Admission)?;
        let _=guard.ready.as_mut().unwrap().await;guard.ready.take();
        collect_ready(guard.thread.take().unwrap()).map_err(AsyncTaskError::Completion)
    }
    // Internal resource steps retain the caller-owned pool only weakly.
    pub(crate) fn repeat_submission(capability:&PoolSubmission)->PoolSubmission{PoolSubmission{inner:capability.inner.clone()}}
    struct Published<T:Send+'static,E:Send+'static,P:FnOnce(Result<T,AsyncTaskError<E>>)+Send+'static>{pending:Option<Pending<T,E>>,publish:Option<P>}
    impl<T:Send+'static,E:Send+'static,P:FnOnce(Result<T,AsyncTaskError<E>>)+Send+'static> Drop for Published<T,E,P>{fn drop(&mut self){
        if let(Some(mut pending),Some(publish))=(self.pending.take(),self.publish.take()){
            cancel(pending.thread.as_ref().unwrap());
            crate::tsrust_native_futures::cleanup(async move{
                let _=pending.ready.as_mut().unwrap().await;pending.ready.take();
                let result=collect_ready(pending.thread.take().unwrap()).map_err(AsyncTaskError::Completion);
                // collect_ready releases capacity before restoring a reusable resource.
                publish(result);
            });
        }
    }}
    pub(crate) async fn task_async_publish<T:Send+'static,E:Send+'static>(capability:PoolSubmission,work:impl FnOnce()->Result<T,E>+Send+'static,publish:impl FnOnce(Result<T,AsyncTaskError<E>>)+Send+'static){
        let pending=match pending_task(capability,work){Ok(pending)=>pending,Err(error)=>{publish(Err(AsyncTaskError::Admission(error)));return;}};
        let mut guard=Published{pending:Some(pending),publish:Some(publish)};
        let _=guard.pending.as_mut().unwrap().ready.as_mut().unwrap().await;
        let mut pending=guard.pending.take().unwrap();pending.ready.take();
        let result=collect_ready(pending.thread.take().unwrap()).map_err(AsyncTaskError::Completion);
        guard.publish.take().unwrap()(result);
    }
