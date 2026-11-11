// Admission candidate, not a public Reader implementation. No borrowed caller
// storage crosses spawn_blocking. One reusable staging buffer belongs to the
// reader; cancellation retains staged bytes and deferred errors for the next read.
use std::{future::Future,io::{self,Read},pin::Pin,sync::{Arc,Mutex},task::{Context,Poll}};
struct State<R>{reader:Option<R>,buffer:Vec<u8>,pending:Option<tokio::task::JoinHandle<(R,Vec<u8>,io::Result<usize>)>>,start:usize,end:usize,error:Option<io::Error>}
pub struct OwnedRead<R>{state:Arc<Mutex<State<R>>>}
impl<R:Read+Send+'static> OwnedRead<R>{
 pub fn new(reader:R,capacity:usize)->Self{assert!(capacity>0);Self{state:Arc::new(Mutex::new(State{reader:Some(reader),buffer:vec![0;capacity],pending:None,start:0,end:0,error:None}))}}
 fn settle(state:&Arc<Mutex<State<R>>>,cx:&mut Context<'_>)->Poll<()> {
  let mut state=state.lock().unwrap();let Some(pending)=state.pending.as_mut()else{return Poll::Ready(());};
  let result=match Pin::new(pending).poll(cx){Poll::Pending=>return Poll::Pending,Poll::Ready(result)=>result};state.pending=None;
  match result{
   Ok((reader,buffer,result))=>{state.reader=Some(reader);state.buffer=buffer;state.start=0;match result{Ok(count)=>state.end=count,Err(error)=>{state.end=0;state.error=Some(error);}}},
   Err(error)=>state.error=Some(io::Error::other(format!("blocking read failed: {error}"))),
  }Poll::Ready(())
 }
 pub async fn read_into(&mut self,destination:&mut[u8])->io::Result<usize>{
  // Zero destination length does not probe EOF or consume staged bytes.
  if destination.is_empty(){return Ok(0);}
  {
   let mut state=self.state.lock().unwrap();
   if state.pending.is_none()&&state.start==state.end&&state.error.is_none(){
    let mut reader=state.reader.take().ok_or_else(||io::Error::other("reader unavailable after worker panic"))?;
    let mut buffer=std::mem::take(&mut state.buffer);let requested=destination.len().min(buffer.len());
    state.pending=Some(tokio::task::spawn_blocking(move||{
     let result=loop{match reader.read(&mut buffer[..requested]){Err(error)if error.kind()==io::ErrorKind::Interrupted=>continue,result=>break result}};
     (reader,buffer,result)
    }));
   }
  }
  let mut loan=PendingRead{state:self.state.clone(),armed:true};
  std::future::poll_fn(|cx|Self::settle(&self.state,cx)).await;loan.armed=false;
  let mut state=self.state.lock().unwrap();if let Some(error)=state.error.take(){return Err(error);}
  let count=destination.len().min(state.end-state.start);destination[..count].copy_from_slice(&state.buffer[state.start..state.start+count]);state.start+=count;Ok(count)
 }
 pub async fn read_exact_into(&mut self,destination:&mut[u8])->Result<usize,(io::ErrorKind,usize)>{
  let mut filled=0;while filled<destination.len(){match self.read_into(&mut destination[filled..]).await{
   Ok(0)=>return Err((io::ErrorKind::UnexpectedEof,filled)),Ok(count)=>filled+=count,Err(error)=>return Err((error.kind(),filled)),
  }}Ok(filled)
 }
}
struct PendingRead<R:Read+Send+'static>{state:Arc<Mutex<State<R>>>,armed:bool}
impl<R:Read+Send+'static> Drop for PendingRead<R>{fn drop(&mut self){if self.armed{
 let state=self.state.clone();crate::tsrust_native_futures::cleanup(async move{std::future::poll_fn(|cx|OwnedRead::<R>::settle(&state,cx)).await;});
}}}
