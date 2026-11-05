// Prototype-only extension inserted inside the existing native thread module.
// The driver adds a oneshot publication hook; production sources are untouched.
pub enum ProbeError<E>{Admission(SpawnError),Completion(JoinError<E>)}
struct Pending<T:Send+'static,E:Send+'static>{thread:Option<Thread<T,E>>,ready:Option<tokio::sync::oneshot::Receiver<()>>}
fn collect_ready<T,E>(thread:Thread<T,E>)->Result<T,JoinError<E>>{
    assert!(thread.completion.result.lock().unwrap().is_some());
    assert!(thread.dedicated.is_none());
    // Result publication precedes notification: no Condvar wait or OS join.
    join(thread)
}
impl<T:Send+'static,E:Send+'static> Drop for Pending<T,E>{fn drop(&mut self){
    if let(Some(thread),Some(ready))=(self.thread.take(),self.ready.take()){
        cancel(&thread);
        crate::tsrust_native_futures::cleanup(async move{ready.await.expect("completion");drop(collect_ready(thread));});
    }
}}
pub async fn task_async_probe<T:Send+'static,E:Send+'static>(pool:&OwnedPool,work:impl FnOnce()->Result<T,E>+Send+'static)->Result<T,ProbeError<E>>{
    // Admission occurs at first poll; an unpolled future owns only its callback.
    task_async_arc(pool.inner.clone(),work).await
}
pub fn task_async_retained_probe<T:Send+'static,E:Send+'static>(pool:&OwnedPool,work:impl FnOnce()->Result<T,E>+Send+'static)->impl std::future::Future<Output=Result<T,ProbeError<E>>>+Send+'static{
    // Retain the existing submission Arc, not another close-on-drop OwnedPool.
    // This clones at creation but admits no work before the first poll.
    let capability=pool.inner.clone();
    async move{task_async_arc(capability,work).await}
}
pub fn task_async_weak_probe<T:Send+'static,E:Send+'static>(pool:&OwnedPool,work:impl FnOnce()->Result<T,E>+Send+'static)->impl std::future::Future<Output=Result<T,ProbeError<E>>>+Send+'static{
    // Both candidates own work immediately and admit only when first polled.
    // Weak retains the Arc allocation header, not Pool or its idle executor.
    // Original OwnedPool Drop/shutdown still closes admission. An expired Weak
    // or a surviving but closed Pool both return the existing closed error.
    // Once accepted, the normal Completion owns its strong Pool and Pending
    // cancellation still awaits actual publication before releasing work.
    let capability=Arc::downgrade(&pool.inner);
    async move{
        let pool=capability.upgrade().ok_or_else(||ProbeError::Admission(failure("closed","The explicit pool owner has been dropped.")))?;
        task_async_arc(pool,work).await
    }
}
pub fn pool_alive_probe(pool:&OwnedPool)->impl Fn()->bool+Send+'static{
    let observer=Arc::downgrade(&pool.inner);
    move||observer.strong_count()!=0
}
async fn task_async_arc<T:Send+'static,E:Send+'static>(pool:Arc<Pool>,work:impl FnOnce()->Result<T,E>+Send+'static)->Result<T,ProbeError<E>>{
    let thread=submit(pool,work).map_err(ProbeError::Admission)?;
    let(send,ready)=tokio::sync::oneshot::channel();
    {let result=thread.completion.result.lock().unwrap();
     if result.is_some(){let _=send.send(());}else{*thread.completion.probe_done.lock().unwrap()=Some(send);}}
    let mut guard=Pending{thread:Some(thread),ready:Some(ready)};
    guard.ready.as_mut().unwrap().await.expect("completion");guard.ready.take();
    collect_ready(guard.thread.take().unwrap()).map_err(ProbeError::Completion)
}
