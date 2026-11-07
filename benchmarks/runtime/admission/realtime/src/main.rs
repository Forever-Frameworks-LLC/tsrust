//! Direct backend admission only. No public compiler/runtime implementation.
use axum::{response::{IntoResponse,sse::{Event,KeepAlive,Sse}},body::Body};
use futures_util::stream;
use http_body_util::BodyExt;
use serde::Deserialize;
use std::{convert::Infallible,sync::{Arc,atomic::{AtomicUsize,Ordering}},time::Duration};
#[cfg(feature="websocket")]mod websocket;
#[derive(Deserialize)]struct Fixtures{sse:Vec<EventSpec>}
#[derive(Deserialize)]struct EventSpec{id:String,data:String,event:Option<String>,event_id:Option<String>,retry_ms:Option<i64>}
fn event(spec:&EventSpec)->Result<Event,&'static str>{
 if spec.event.as_ref().is_some_and(|v|v.contains(['\r','\n']))||spec.event_id.as_ref().is_some_and(|v|v.contains(['\r','\n','\0'])){return Err("field");}
 let mut event=Event::default().data(&spec.data);
 if let Some(value)=&spec.event{event=event.event(value);}
 if let Some(value)=&spec.event_id{event=event.id(value);}
 if let Some(value)=spec.retry_ms{if value<0{return Err("retry");}event=event.retry(Duration::from_millis(value as u64));}
 Ok(event)
}
struct Lifetime(Arc<AtomicUsize>);
impl Lifetime{fn new(active:Arc<AtomicUsize>)->Self{active.fetch_add(1,Ordering::SeqCst);Self(active)}}
impl Drop for Lifetime{fn drop(&mut self){self.0.fetch_sub(1,Ordering::SeqCst);}}
async fn bytes(body:Body)->Vec<u8>{body.collect().await.unwrap().to_bytes().to_vec()}
async fn sse_format(){
 let fixtures:Fixtures=serde_json::from_str(include_str!("../fixtures.json")).unwrap();
 for spec in fixtures.sse{let observed=match event(&spec){Ok(value)=>serde_json::json!({"id":spec.id,"bytes":bytes(Sse::new(stream::iter([Ok::<_,Infallible>(value)])).into_response().into_body()).await}),Err(code)=>serde_json::json!({"id":spec.id,"error":code})};println!("{observed}");}
}
async fn sse_lifetime(){
 // Capacity one is an explicit fixture, not a proposed ordinary facade default.
 let active=Arc::new(AtomicUsize::new(0));let(tx,rx)=tokio::sync::mpsc::channel(1);let(ready,blocked)=tokio::sync::oneshot::channel();let life=Lifetime::new(active.clone());
 let producer=tokio::spawn(async move{let _life=life;tx.send(Ok::<_,Infallible>(Event::default().data("first"))).await.unwrap();let second=tx.send(Ok::<_,Infallible>(Event::default().data("second")));tokio::pin!(second);assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(std::future::Future::poll(second.as_mut(),cx).is_pending())).await);ready.send(()).unwrap();assert!(second.await.is_err());});
 let source=stream::unfold(rx,|mut rx|async move{rx.recv().await.map(|value|(value,rx))});let body=Sse::new(source).into_response().into_body();blocked.await.unwrap();assert_eq!(active.load(Ordering::SeqCst),1);drop(body);producer.await.unwrap();assert_eq!(active.load(Ordering::SeqCst),0);
 println!("{}",serde_json::json!({"id":"sse-disconnect","active":active.load(Ordering::SeqCst),"producer_settled":true,"send_was_pending":true}));
 let(tx,rx)=tokio::sync::mpsc::channel::<Result<Event,Infallible>>(1);let source=stream::unfold(rx,|mut rx|async move{rx.recv().await.map(|value|(value,rx))});let mut body=Sse::new(source).into_response().into_body();let absent=tokio::time::timeout(Duration::from_millis(10),body.frame()).await.is_err();drop(body);drop(tx);assert!(absent);println!("{}",serde_json::json!({"id":"sse-no-keepalive","idle":true}));
 let(tx,rx)=tokio::sync::mpsc::channel::<Result<Event,Infallible>>(1);let source=stream::unfold(rx,|mut rx|async move{rx.recv().await.map(|value|(value,rx))});let mut body=Sse::new(source).keep_alive(KeepAlive::new().interval(Duration::from_millis(5)).text("alive")).into_response().into_body();let observed=tokio::time::timeout(Duration::from_secs(1),body.frame()).await.unwrap().unwrap().unwrap().into_data().unwrap();println!("{}",serde_json::json!({"id":"sse-explicit-keepalive","bytes":observed.to_vec()}));drop(body);drop(tx);
}
fn main(){let args:Vec<_>=std::env::args().collect();if args.get(1).map(String::as_str)==Some("identity"){print!("{}",include_str!(concat!(env!("OUT_DIR"),"/authored-identity.json")));return;}let runtime=tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap();runtime.block_on(async{match args.get(1).map(String::as_str){Some("sse-format")=>sse_format().await,Some("sse-lifetime")=>sse_lifetime().await,#[cfg(feature="websocket")]Some("ws-controls")=>websocket::controls().await,#[cfg(feature="websocket")]Some("serve")=>websocket::serve().await,#[cfg(feature="websocket-rustls")]Some("tls-client")=>websocket::tls_client(args.get(2).unwrap(),args.get(3).map(String::as_str)).await,_=>panic!("sse-format | sse-lifetime | ws-controls | serve (websocket feature)")}});}
