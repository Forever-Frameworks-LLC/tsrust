mod tsrust_native_logging {
 // Local tracing dispatch and synchronous sinks. No global subscriber or worker.
 use std::{future::Future,pin::Pin,io::{self,Write},sync::{Arc,Mutex}};
 use tracing::{Dispatch,Level,Span,Instrument,instrument::WithSubscriber};
 use tracing_subscriber::{layer::SubscriberExt,Layer,filter::filter_fn,fmt::MakeWriter};
 #[derive(Debug)]pub struct Error{pub code:&'static str,pub message:String}
 impl std::fmt::Display for Error{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.message)}}
 impl std::error::Error for Error{}
 fn error(code:&'static str,message:impl Into<String>)->Error{Error{code,message:message.into()}}
 fn io_error(_:io::Error)->Error{error("io","Logging sink operation failed")}
 enum Destination{Stdout(io::Stdout),Stderr(io::Stderr),File(std::fs::File)}
 impl Write for Destination{
  fn write(&mut self,bytes:&[u8])->io::Result<usize>{match self{Self::Stdout(value)=>value.write(bytes),Self::Stderr(value)=>value.write(bytes),Self::File(value)=>value.write(bytes)}}
  fn flush(&mut self)->io::Result<()>{match self{Self::Stdout(value)=>value.flush(),Self::Stderr(value)=>value.flush(),Self::File(value)=>value.flush()}}
 }
 #[derive(Clone)]struct Sink(Arc<Mutex<Destination>>);
 struct Writer(Sink);
 impl Write for Writer{
  fn write(&mut self,bytes:&[u8])->io::Result<usize>{self.0.0.lock().map_err(|_|io::Error::other("Logging sink lock failed"))?.write(bytes)}
  fn flush(&mut self)->io::Result<()>{self.0.0.lock().map_err(|_|io::Error::other("Logging sink lock failed"))?.flush()}
 }
 impl<'a>MakeWriter<'a> for Sink{type Writer=Writer;fn make_writer(&'a self)->Writer{Writer(self.clone())}}
 // WithDispatch sets the default only while polling. Its inner Instrumented
 // enters the span during drop, so retain that composition under the same
 // dispatch until its future has actually been dropped in place.
 pin_project_lite::pin_project! {
  struct ContextFuture<F> {
   #[pin]
   inner:Option<F>,
   dispatch:Dispatch,
  }
  impl<F> PinnedDrop for ContextFuture<F> {
   fn drop(this:Pin<&mut Self>){
    let mut this=this.project();
    tracing::dispatcher::with_default(this.dispatch,||this.inner.set(None));
   }
  }
 }
 impl<F:Future> Future for ContextFuture<F>{
  type Output=F::Output;
  fn poll(self:std::pin::Pin<&mut Self>,cx:&mut std::task::Context<'_>)->std::task::Poll<Self::Output>{
   self.project().inner.as_pin_mut().expect("context future remains owned until drop").poll(cx)
  }
 }
 type Fields=dyn Fn(&mut dyn FnMut())+Send+Sync+'static;
 pub struct Logger{dispatch:Dispatch,sink:Sink,fields:Option<Arc<Fields>>}
 impl Logger{
  pub fn create(destination:&str,path:Option<&str>,level:Option<&str>)->Result<Self,Error>{
   let maximum=match level.unwrap_or("info"){"trace"=>Some(Level::TRACE),"debug"=>Some(Level::DEBUG),"info"=>Some(Level::INFO),"warn"=>Some(Level::WARN),"error"=>Some(Level::ERROR),"off"=>None,_=>return Err(error("options","Unknown logging level"))};
   let destination=match(destination,path){("stdout",None)=>Destination::Stdout(io::stdout()),("stderr",None)=>Destination::Stderr(io::stderr()),("file",Some(path))=>Destination::File(std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(io_error)?),_=>return Err(error("options","Choose stdout/stderr without path or file with a path"))};
   let sink=Sink(Arc::new(Mutex::new(destination)));
   // Preserve selected formatter defaults: timestamps, target, current span and span list.
   // Context spans remain registered independently of the event threshold.
   let layer=tracing_subscriber::fmt::layer().json().with_writer(sink.clone()).with_filter(filter_fn(move|metadata|metadata.is_span()||maximum.is_some_and(|maximum|*metadata.level()<=maximum)));
   Ok(Self{dispatch:Dispatch::new(tracing_subscriber::registry().with(layer)),sink,fields:None})
  }
  pub fn share(&self)->Self{Self{dispatch:self.dispatch.clone(),sink:self.sink.clone(),fields:self.fields.clone()}}
  pub fn enabled(&self,level:&str)->bool{tracing::dispatcher::with_default(&self.dispatch,||match level{"debug"=>tracing::enabled!(Level::DEBUG),"info"=>tracing::enabled!(Level::INFO),"warn"=>tracing::enabled!(Level::WARN),"error"=>tracing::enabled!(Level::ERROR),_=>false})}
  // Compiler-generated closures own a checked concrete scalar record, using static tracing fields.
  // Each child adds one Arc. Payloads move once and are never cloned per event.
  pub fn child<F>(&self,enter:F)->Self where F:Fn(&mut dyn FnMut())+Send+Sync+'static{
   let parent=self.fields.clone();let fields:Arc<Fields>=Arc::new(move|emit:&mut dyn FnMut()|{if let Some(parent)=&parent{parent(&mut||enter(emit));}else{enter(emit);}});
   Self{dispatch:self.dispatch.clone(),sink:self.sink.clone(),fields:Some(fields)}
  }
  pub fn emit(&self,emit:impl FnOnce()){
   tracing::dispatcher::with_default(&self.dispatch,||{let mut emit=Some(emit);let mut once=||emit.take().expect("checked logging event runs once")();if let Some(fields)=&self.fields{fields(&mut once);}else{once();}})
  }
  pub fn flush(&self)->Result<(),Error>{self.sink.0.lock().map_err(|_|error("io","Logging sink lock failed"))?.flush().map_err(io_error)}
  // A handle close flushes and releases that owner. Shared children retain their sink until final drop.
  pub fn close(self)->Result<(),Error>{self.flush()}
  pub fn request<F:Future>(&self,request:u64,future:F)->impl Future<Output=F::Output>+use<F>{
   let span=tracing::dispatcher::with_default(&self.dispatch,||tracing::info_span!("request",request_id=request));ContextFuture{inner:Some(future.instrument(span).with_subscriber(self.dispatch.clone())),dispatch:self.dispatch.clone()}
  }
 }
 // Capture both dispatch and current ancestry at Task ownership boundaries. Instrumented owns
 // its future and enters only while polling/dropping, including cancellation destructors.
 pub fn capture<F:Future>(future:F)->impl Future<Output=F::Output>{let dispatch=tracing::dispatcher::get_default(Clone::clone);ContextFuture{inner:Some(future.instrument(Span::current()).with_subscriber(dispatch.clone())),dispatch}}
}
