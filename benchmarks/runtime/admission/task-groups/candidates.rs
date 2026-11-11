// Included inside the unchanged runtime module by the admission driver only.
// F is the emitter's existing Pin<Box<dyn Future>> and therefore Unpin even
// when the owned or borrowed future behind that pointer is not Unpin.
pub(crate) async fn all_once<F,T,E>(futures:Vec<F>)->Result<Vec<T>,Failure<E>> where F:Future<Output=Result<T,Failure<E>>>+Unpin {
    let scope=Scope::group();
    let result=SCOPE.scope(scope.clone(),async move {
        let mut futures=futures.into_iter().map(Some).collect::<Vec<_>>();
        let mut results=(0..futures.len()).map(|_|None).collect::<Vec<_>>();
        std::future::poll_fn(|cx|{
            let mut complete=true;
            for (future,result) in futures.iter_mut().zip(&mut results) {if let Some(value)=future {match std::pin::Pin::new(value).poll(cx){
                Poll::Ready(Ok(value))=>{*result=Some(value);*future=None;},
                Poll::Ready(Err(error))=>return Poll::Ready(Err(error)),
                Poll::Pending=>complete=false,
            }}}
            if complete {Poll::Ready(Ok(results.iter_mut().map(|value|value.take().unwrap()).collect()))}else{Poll::Pending}
        }).await
    }).await;
    scope.drain(false).await;result
}

// Four is the exact measured source literal width, not a new API limit.
pub(crate) async fn all_four<A,B,C,D,T,E>(a:A,b:B,c:C,d:D)->Result<Vec<T>,Failure<E>>
where A:Future<Output=Result<T,Failure<E>>>,B:Future<Output=Result<T,Failure<E>>>,C:Future<Output=Result<T,Failure<E>>>,D:Future<Output=Result<T,Failure<E>>> {
    let scope=Scope::group();
    let result=SCOPE.scope(scope.clone(),async move {
        let mut a=std::pin::pin!(Some(a));let mut b=std::pin::pin!(Some(b));
        let mut c=std::pin::pin!(Some(c));let mut d=std::pin::pin!(Some(d));
        let mut results=[None,None,None,None];
        std::future::poll_fn(|cx|{
            let mut complete=true;
            macro_rules! poll_slot {($slot:ident,$index:expr)=>{
                if let Some(value)=$slot.as_mut().as_pin_mut(){match value.poll(cx){
                    Poll::Ready(Ok(value))=>{results[$index]=Some(value);$slot.as_mut().set(None);},
                    Poll::Ready(Err(error))=>return Poll::Ready(Err(error)),
                    Poll::Pending=>complete=false,
                }}
            }}
            poll_slot!(a,0);poll_slot!(b,1);poll_slot!(c,2);poll_slot!(d,3);
            if complete {Poll::Ready(Ok(results.iter_mut().map(|value|value.take().unwrap()).collect()))}else{Poll::Pending}
        }).await
    }).await;
    scope.drain(false).await;result
}
