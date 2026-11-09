// Included inside tsrust_native_http. Axum owns all event formatting.
#[derive(Default)]
pub struct SseOptions {pub keep_alive_ms:Option<i64>,pub keep_alive_text:Option<String>}
fn sse_options(options:SseOptions)->Result<Option<axum::response::sse::KeepAlive>,Error>{
    if options.keep_alive_ms.is_none()&&options.keep_alive_text.is_none(){return Ok(None);}
    let mut keep_alive=axum::response::sse::KeepAlive::new();
    if let Some(value)=options.keep_alive_ms{
        let value=u64::try_from(value).map_err(|_|error("sse","keepAliveMs must be nonnegative"))?;
        let duration=Duration::from_millis(value);
        if tokio::time::Instant::now().checked_add(duration).is_none(){return Err(error("sse","keepAliveMs exceeds the backend timer range"));}
        keep_alive=keep_alive.interval(duration);
    }
    if let Some(text)=options.keep_alive_text{
        if text.contains(['\r','\n']){return Err(error("sse","keepAliveText must not contain CR or LF"));}
        keep_alive=keep_alive.text(text);
    }
    Ok(Some(keep_alive))
}

pub fn sse_event(data:&str,event:Option<&str>,id:Option<&str>,retry_ms:Option<i64>)->Result<axum::response::sse::Event,Error>{
    if event.as_ref().is_some_and(|value|value.contains(['\r','\n'])){return Err(error("sse","event must not contain CR or LF"));}
    if id.as_ref().is_some_and(|value|value.contains(['\r','\n','\0'])){return Err(error("sse","id must not contain CR, LF or NUL"));}
    let retry=retry_ms.map(|value|u64::try_from(value).map(Duration::from_millis).map_err(|_|error("sse","retryMs must be nonnegative"))).transpose()?;
    let mut value=axum::response::sse::Event::default().data(data);
    if let Some(event)=event{value=value.event(event);}
    if let Some(id)=id{value=value.id(id);}
    if let Some(retry)=retry{value=value.retry(retry);}
    Ok(value)
}
struct SseEvents<T,F,E:Send+'static>{receiver:tokio::sync::mpsc::Receiver<T>,event:F,producer:Option<tasks::Job<(),E>>,failure:Option<Error>,finished:bool}
impl<T:Send+'static,F:Fn(T)->Result<axum::response::sse::Event,Error>+Send+Unpin+'static,E:Send+'static> futures_core::Stream for SseEvents<T,F,E>{
    type Item=Result<axum::response::sse::Event,Error>;
    fn poll_next(mut self:Pin<&mut Self>,cx:&mut std::task::Context<'_>)->Poll<Option<Self::Item>>{
        if self.finished{return Poll::Ready(None);}
        if let Some(job)=&mut self.producer{
            if let Poll::Ready(result)=job.poll_join(cx){
                // The typed cause remains owned until actual settlement. The
                // HTTP transport boundary neither serializes it nor exposes E.
                if result.is_err(){self.failure=Some(error("producer","SSE producer failed after response creation"));}
                self.producer.take();self.receiver.close();
            }
        }
        match self.receiver.poll_recv(cx){
            Poll::Pending=>Poll::Pending,
            Poll::Ready(Some(value))=>{
                let value=(self.event)(value);
                if value.is_err(){self.finished=true;self.receiver.close();if let Some(producer)=&self.producer{producer.cancel();}}
                Poll::Ready(Some(value))
            },
            Poll::Ready(None) if self.producer.is_some()=>Poll::Pending,
            Poll::Ready(None)=>{self.finished=true;Poll::Ready(self.failure.take().map(Err))}
        }
    }
}
impl<T,F,E:Send+'static> Drop for SseEvents<T,F,E>{
    fn drop(&mut self){
        self.receiver.close();
        if let Some(producer)=self.producer.take(){
            producer.cancel();
            tasks::cleanup(async move{drop(producer.join().await);});
        }
    }
}
impl Response{
    pub fn sse<T:Send+'static,F:Fn(T)->Result<axum::response::sse::Event,Error>+Send+Unpin+'static>(channel:tasks::Channel<T>,options:SseOptions,event:F)->Result<Self,Error>{
        Self::sse_inner(channel,options,event,None::<tasks::Job<(),()>>)
    }
    pub fn sse_with_producer<T:Send+'static,F:Fn(T)->Result<axum::response::sse::Event,Error>+Send+Unpin+'static,E:Send+'static>(channel:tasks::Channel<T>,producer:tasks::Job<(),E>,options:SseOptions,event:F)->Result<Self,Error>{
        Self::sse_inner(channel,options,event,Some(producer))
    }
    fn sse_inner<T:Send+'static,F:Fn(T)->Result<axum::response::sse::Event,Error>+Send+Unpin+'static,E:Send+'static>(channel:tasks::Channel<T>,options:SseOptions,event:F,producer:Option<tasks::Job<(),E>>)->Result<Self,Error>{
        use axum::response::IntoResponse;
        let events=SseEvents{receiver:channel.into_receiver(),event,producer,failure:None,finished:false};
        let keep_alive=sse_options(options)?;
        let response=axum::response::sse::Sse::new(events);
        let response=if let Some(keep_alive)=keep_alive{response.keep_alive(keep_alive).into_response()}else{response.into_response()};
        Ok(Self{inner:response.map(ExclusiveBody::new),limit:None})
    }
}
