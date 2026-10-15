mod tsrust_native_futures {
    use std::{future::Future, sync::{Arc, Mutex, atomic::{AtomicU8, Ordering}}, task::Poll, time::Duration};
    use tokio::{sync::{Notify, oneshot, mpsc}, task::AbortHandle};
    tokio::task_local! { static SCOPE: Arc<Scope>; }
    #[derive(Default)]
    pub(crate) struct Scope {
        children: Mutex<std::collections::BTreeMap<usize,Arc<Control>>>,
        cleanup: Mutex<Vec<Arc<Cleanup>>>,
        groups: Mutex<std::collections::BTreeMap<usize,Arc<Scope>>>,
        parent: std::sync::Weak<Scope>,
    }
    impl Scope {
        pub(crate) fn group()->Arc<Self> {
            SCOPE.with(|parent| {
                let scope=Arc::new(Self{parent:Arc::downgrade(parent),..Self::default()});
                parent.groups.lock().unwrap().insert(Arc::as_ptr(&scope) as usize,scope.clone());
                scope
            })
        }
        pub(crate) async fn drain(&self, cancel_all:bool) {
            let mut children = std::mem::take(&mut *self.children.lock().unwrap());
            let mut cleanup=self.cleanup.lock().unwrap().clone();
            let mut groups=std::mem::take(&mut *self.groups.lock().unwrap()).into_values().collect::<Vec<_>>();
            while let Some(group)=groups.pop(){
                children.extend(std::mem::take(&mut *group.children.lock().unwrap()));
                cleanup.extend(group.cleanup.lock().unwrap().iter().cloned());
                groups.extend(std::mem::take(&mut *group.groups.lock().unwrap()).into_values());
            }
            let parent=self.parent.upgrade();
            let mut abandoned=Vec::new();
            for child in children.into_values() {
                if !cancel_all && child.handle_alive.load(Ordering::Acquire) && let Some(parent)=&parent {child.register(parent);}else{child.cancel();abandoned.push(child);}
            }
            for child in abandoned {child.wait().await;}
            cleanup.extend(self.cleanup.lock().unwrap().iter().cloned());
            for release in cleanup {release.wait().await;}
            self.cleanup.lock().unwrap().retain(|release|!release.finished.load(Ordering::Acquire));
            if let Some(parent)=self.parent.upgrade(){parent.groups.lock().unwrap().remove(&(self as *const Self as usize));}
        }
    }
    struct Cleanup {finished:std::sync::atomic::AtomicBool,done:Notify}
    impl Cleanup {async fn wait(&self){loop{let ready=self.done.notified();if self.finished.load(Ordering::Acquire){return;}ready.await;}}}
    struct CleanupDone(Arc<Cleanup>);
    impl Drop for CleanupDone {fn drop(&mut self){self.0.finished.store(true,Ordering::Release);self.0.done.notify_waiters();}}
    /// Keep cleanup registered even if an enclosing drain is itself cancelled.
    pub(crate) fn cleanup(future:impl Future<Output=()>+Send+'static){
        let Ok(runtime)=tokio::runtime::Handle::try_current()else{return;};
        let release=Arc::new(Cleanup{finished:std::sync::atomic::AtomicBool::new(false),done:Notify::new()});
        let _=SCOPE.try_with(|scope|{let mut pending=scope.cleanup.lock().unwrap();pending.retain(|release|!release.finished.load(Ordering::Acquire));pending.push(release.clone());});
        runtime.spawn(async move{let _done=CleanupDone(release);future.await;});
    }
    struct Control { abort: AbortHandle, shutdown: Mutex<Option<Box<dyn FnOnce()+Send>>>, state: AtomicU8, done: Notify, finished: std::sync::atomic::AtomicBool, handle_alive: std::sync::atomic::AtomicBool, owner:Mutex<std::sync::Weak<Scope>> }
    impl Control {
        fn register(self:&Arc<Self>,scope:&Arc<Scope>){
            let mut owner=self.owner.lock().unwrap();
            if let Some(previous)=owner.upgrade(){previous.children.lock().unwrap().remove(&(Arc::as_ptr(self) as usize));}
            *owner=Arc::downgrade(scope);
            if !self.finished.load(Ordering::Acquire){scope.children.lock().unwrap().insert(Arc::as_ptr(self) as usize,self.clone());}
        }
        fn complete(self:&Arc<Self>){
            self.finished.store(true,Ordering::Release);self.done.notify_waiters();
            if let Some(owner)=self.owner.lock().unwrap().upgrade(){owner.children.lock().unwrap().remove(&(Arc::as_ptr(self) as usize));}
        }
        fn cancel(&self) -> bool {
            if self.state.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_ok() { if let Some(shutdown)=self.shutdown.lock().unwrap().take(){shutdown();}else{self.abort.abort();} true } else { false }
        }
        async fn wait(&self) {
            loop { let ready=self.done.notified(); if self.finished.load(Ordering::Acquire) {return;} ready.await; }
        }
    }
    pub enum Failure<E> { Worker(E), Cancelled, Panic, Timeout, Range, Empty, Closed }
    impl<E> Failure<E>{pub fn map<U>(self,convert:impl FnOnce(E)->U)->Failure<U>{match self{Self::Worker(error)=>Failure::Worker(convert(error)),Self::Cancelled=>Failure::Cancelled,Self::Panic=>Failure::Panic,Self::Timeout=>Failure::Timeout,Self::Range=>Failure::Range,Self::Empty=>Failure::Empty,Self::Closed=>Failure::Closed}}}
    pub struct Job<T,E> { result: Option<oneshot::Receiver<Result<T,Failure<E>>>>, control: Arc<Control>, settled: bool }
    impl<T,E> Job<T,E> {
        pub fn cancel(&self)->bool {self.control.cancel()}
        pub async fn join(mut self)->Result<T,Failure<E>> {let result=self.result.take().unwrap().await.unwrap_or(Err(Failure::Panic));self.settled=true;result}
    }
    impl<T,E> Drop for Job<T,E> { fn drop(&mut self) {self.control.handle_alive.store(false,Ordering::Release);if !self.settled{self.control.cancel();let _=SCOPE.try_with(|scope|self.control.register(scope));}} }
    pub fn start<F,T,E>(future:F)->Job<T,E> where F:Future<Output=Result<T,Failure<E>>>+Send+'static,T:Send+'static,E:Send+'static {
        start_with_shutdown(future,None)
    }
    pub(crate) async fn in_scope<F:Future>(scope:Arc<Scope>,future:F)->F::Output {SCOPE.scope(scope,future).await}
    pub(crate) fn start_with_shutdown<F,T,E>(future:F,shutdown:Option<Box<dyn FnOnce()+Send>>)->Job<T,E> where F:Future<Output=Result<T,Failure<E>>>+Send+'static,T:Send+'static,E:Send+'static {
        let scope=Arc::new(Scope{parent:SCOPE.with(Arc::downgrade),..Scope::default()});
        let (gate,ready)=oneshot::channel::<Arc<Control>>();
        let child_scope=scope.clone();
        let worker=tokio::spawn(SCOPE.scope(child_scope,async move {
            let control=ready.await.expect("task registration");
            let result=future.await;
            if control.state.compare_exchange(0,2,Ordering::AcqRel,Ordering::Acquire).is_ok() {result} else {Err(Failure::Cancelled)}
        }));
        let control=Arc::new(Control{abort:worker.abort_handle(),shutdown:Mutex::new(shutdown),state:AtomicU8::new(0),done:Notify::new(),finished:std::sync::atomic::AtomicBool::new(false),handle_alive:std::sync::atomic::AtomicBool::new(true),owner:Mutex::new(std::sync::Weak::new())});
        SCOPE.with(|parent|control.register(parent));
        let (send,result)=oneshot::channel();let completion=control.clone();
        tokio::spawn(async move {
            let result=match worker.await {Ok(result)=>result,Err(error)=>Err(if error.is_cancelled(){Failure::Cancelled}else{Failure::Panic})};
            scope.drain(false).await;
            if let Err(result)=send.send(result){
                SCOPE.scope(scope.clone(),async move{drop(result)}).await;
                scope.drain(true).await;
            }
            completion.complete();
        });
        let _=gate.send(control.clone());
        Job{result:Some(result),control,settled:false}
    }
    pub fn block_on<F:Future>(future:F)->F::Output {
        let runtime=tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("create async runtime");
        runtime.block_on(async {
            let scope=Arc::new(Scope::default());
            let result=SCOPE.scope(scope.clone(),future).await;
            scope.drain(true).await;
            result
        })
    }
    pub async fn delay<E>(milliseconds:i64)->Result<(),Failure<E>> {
        if !(0..=31536000000).contains(&milliseconds){return Err(Failure::Range);}
        tokio::time::sleep(Duration::from_millis(milliseconds as u64)).await;Ok(())
    }
    pub async fn yield_now<E>()->Result<(),Failure<E>> {tokio::task::yield_now().await;Ok(())}
    pub async fn timeout<F,T,E>(future:F,milliseconds:i64)->Result<T,Failure<E>> where F:Future<Output=Result<T,Failure<E>>> {
        let scope=Scope::group();
        let result=SCOPE.scope(scope.clone(),async move {
            if !(0..=31536000000).contains(&milliseconds) {drop(future);Err(Failure::Range)} else {
                tokio::time::timeout(Duration::from_millis(milliseconds as u64),future).await.unwrap_or(Err(Failure::Timeout))
            }
        }).await;
        scope.drain(false).await;result
    }
    pub async fn race<F,T,E>(futures:Vec<F>)->Result<T,Failure<E>> where F:Future<Output=Result<T,Failure<E>>> {
        if futures.is_empty(){return Err(Failure::Empty);}
        let scope=Scope::group();
        let result=SCOPE.scope(scope.clone(),async move {
            let mut futures=futures.into_iter().map(Box::pin).collect::<Vec<_>>();
            std::future::poll_fn(|cx|{for future in &mut futures {if let Poll::Ready(value)=future.as_mut().poll(cx){return Poll::Ready(value);}}Poll::Pending}).await
        }).await;
        scope.drain(false).await;result
    }
    pub async fn all_settled<F,T,E>(futures:Vec<F>)->Vec<Result<T,Failure<E>>> where F:Future<Output=Result<T,Failure<E>>> {
        let scope=Scope::group();
        let result=SCOPE.scope(scope.clone(),async move {
            let mut futures=futures.into_iter().map(|future|Some(Box::pin(future))).collect::<Vec<_>>();
            let mut results=(0..futures.len()).map(|_|None).collect::<Vec<_>>();
            std::future::poll_fn(|cx|{
                let mut complete=true;
                for (future,result) in futures.iter_mut().zip(&mut results) {if let Some(value)=future {match value.as_mut().poll(cx){Poll::Ready(value)=>{*result=Some(value);*future=None;},Poll::Pending=>complete=false}}}
                if complete {Poll::Ready(results.iter_mut().map(|value|value.take().unwrap()).collect())}else{Poll::Pending}
            }).await
        }).await;
        scope.drain(false).await;result
    }
    pub struct Channel<T> {sender:mpsc::Sender<T>,receiver:tokio::sync::Mutex<mpsc::Receiver<T>>}
    pub struct Sender<T>(mpsc::Sender<T>);
    impl<T> Channel<T> {
        pub fn new(capacity:i64)->Result<Self,()> {if !(1..=1000000).contains(&capacity){return Err(());}let(sender,receiver)=mpsc::channel(capacity as usize);Ok(Self{sender,receiver:tokio::sync::Mutex::new(receiver)})}
        pub fn sender(&self)->Sender<T>{Sender(self.sender.clone())}
        pub async fn send<E>(&self,value:T)->Result<(),Failure<E>>{self.sender.send(value).await.map_err(|_|Failure::Closed)}
        pub async fn receive<E>(&self)->Result<T,Failure<E>>{self.receiver.lock().await.recv().await.ok_or(Failure::Closed)}
        pub fn close(&mut self){self.receiver.get_mut().close();}
    }
    impl<T> Sender<T>{pub async fn send<E>(&self,value:T)->Result<(),Failure<E>>{self.0.send(value).await.map_err(|_|Failure::Closed)}}
}
