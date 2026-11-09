// Direct Rust admission only. This is not generated-facade acceptance.
include!(env!("TSRUST_IO_FUTURES_SOURCE"));
mod owned_read;mod frame_cursor;
use owned_read::OwnedRead;
use std::{io::{self,Read},sync::{Arc,Mutex,Condvar,atomic::{AtomicUsize,Ordering}},time::Duration};
struct ShortRead{source:io::Cursor<Vec<u8>>,interrupt:bool,calls:Arc<AtomicUsize>}
impl Read for ShortRead{fn read(&mut self,out:&mut[u8])->io::Result<usize>{self.calls.fetch_add(1,Ordering::SeqCst);if self.interrupt{self.interrupt=false;return Err(io::ErrorKind::Interrupted.into());}let length=out.len().min(2);self.source.read(&mut out[..length])}}
struct FailRead{first:bool}
impl Read for FailRead{fn read(&mut self,out:&mut[u8])->io::Result<usize>{if self.first{self.first=false;out[..2].copy_from_slice(&[14,15]);Ok(2)}else{Err(io::ErrorKind::BrokenPipe.into())}}}
#[derive(Default)]struct Gate{open:Mutex<bool>,changed:Condvar}
impl Gate{fn release(&self){*self.open.lock().unwrap()=true;self.changed.notify_all();}fn wait(&self)->io::Result<()>{let open=self.open.lock().unwrap();let(open,result)=self.changed.wait_timeout_while(open,Duration::from_secs(5),|open|!*open).unwrap();if !*open&&result.timed_out(){Err(io::ErrorKind::TimedOut.into())}else{Ok(())}}}
struct ReleaseOnDrop(Arc<Gate>);impl Drop for ReleaseOnDrop{fn drop(&mut self){self.0.release();}}
struct GatedRead{source:io::Cursor<Vec<u8>>,gate:Arc<Gate>,started:Option<tokio::sync::oneshot::Sender<()>>,finished:Arc<AtomicUsize>}
impl Read for GatedRead{fn read(&mut self,out:&mut[u8])->io::Result<usize>{if let Some(started)=self.started.take(){let _=started.send(());}self.gate.wait()?;let result=self.source.read(out);self.finished.fetch_add(1,Ordering::SeqCst);result}}
async fn deadline_contract(){
 let gate=Arc::new(Gate::default());let finished=Arc::new(AtomicUsize::new(0));let(started,ready)=tokio::sync::oneshot::channel();
 let mut reader=OwnedRead::new(GatedRead{source:io::Cursor::new(vec![12,13]),gate:gate.clone(),started:Some(started),finished:finished.clone()},8);
 let mut destination=[80u8;4];let release=ReleaseOnDrop(gate.clone());
 {
  let operation=async{reader.read_into(&mut destination[1..3]).await.map_err(tsrust_native_futures::Failure::Worker)};
  let mut timeout=std::pin::pin!(tsrust_native_futures::timeout(operation,0));
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(timeout.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
  tokio::time::timeout(Duration::from_secs(5),ready).await.unwrap().unwrap();tokio::time::sleep(Duration::from_millis(1)).await;
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(timeout.as_mut(),cx).is_pending(),"deadline released blocking work early");std::task::Poll::Ready(())}).await;
  assert_eq!(finished.load(Ordering::SeqCst),0);gate.release();assert!(matches!(tokio::time::timeout(Duration::from_secs(5),timeout.as_mut()).await.unwrap(),Err(tsrust_native_futures::Failure::Timeout)));
 }
 drop(release);assert_eq!(destination,[80;4]);assert_eq!(reader.read_into(&mut destination[1..3]).await.unwrap(),2);assert_eq!(destination,[80,12,13,80]);assert_eq!(finished.load(Ordering::SeqCst),1);
}
async fn contracts()->String{
 deadline_contract().await;
 let calls=Arc::new(AtomicUsize::new(0));let mut reader=OwnedRead::new(ShortRead{source:io::Cursor::new(vec![1,2,3]),interrupt:true,calls:calls.clone()},8);
 let mut output=[99u8;5];let pointer=output.as_ptr();let zero=reader.read_into(&mut[]).await.unwrap();assert_eq!(calls.load(Ordering::SeqCst),0);
 let exact=reader.read_exact_into(&mut output).await.unwrap_err();assert_eq!(exact,(io::ErrorKind::UnexpectedEof,3));assert_eq!(output,[1,2,3,99,99]);assert_eq!(pointer,output.as_ptr());
 let mut failing=OwnedRead::new(FailRead{first:true},8);let mut partial_error=[99u8;4];let failed=failing.read_exact_into(&mut partial_error).await.unwrap_err();assert_eq!(failed,(io::ErrorKind::BrokenPipe,2));assert_eq!(partial_error,[14,15,99,99]);
 let gate=Arc::new(Gate::default());let finished=Arc::new(AtomicUsize::new(0));let(started,ready)=tokio::sync::oneshot::channel();
 let mut gated=OwnedRead::new(GatedRead{source:io::Cursor::new(vec![4,5,6,7]),gate:gate.clone(),started:Some(started),finished:finished.clone()},8);
 let mut destination=[90u8;6];let identity=destination.as_ptr();let scope=tsrust_native_futures::Scope::group();
 tsrust_native_futures::in_scope(scope.clone(),async{
  let mut future=std::pin::pin!(gated.read_into(&mut destination[1..5]));
  std::future::poll_fn(|cx|{assert!(std::future::Future::poll(future.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
  tokio::time::timeout(Duration::from_secs(5),ready).await.unwrap().unwrap();
 }).await;
 let release=ReleaseOnDrop(gate.clone());let mut drain=std::pin::pin!(scope.drain(true));
 std::future::poll_fn(|cx|{assert!(std::future::Future::poll(drain.as_mut(),cx).is_pending());std::task::Poll::Ready(())}).await;
 assert_eq!(destination,[90;6]);assert_eq!(finished.load(Ordering::SeqCst),0);gate.release();tokio::time::timeout(Duration::from_secs(5),drain.as_mut()).await.unwrap();drop(release);
 assert_eq!(finished.load(Ordering::SeqCst),1);let recovered=gated.read_into(&mut destination[1..5]).await.unwrap();assert_eq!(recovered,4);assert_eq!(destination,[90,4,5,6,7,90]);assert_eq!(identity,destination.as_ptr());assert_eq!(finished.load(Ordering::SeqCst),1,"cancelled data must be staged, not reread");
 let frame=bytes::Bytes::from_static(&[8,9,10,11]);let frame_pointer=frame.as_ptr();let mut frames=frame_cursor::FrameCursor::new([bytes::Bytes::new(),frame]);let mut frame_output=[0u8;2];assert_eq!(frames.read_into(&mut frame_output),2);assert_eq!(frame_output,[8,9]);assert_eq!(frames.remaining_pointer(),frame_pointer.wrapping_add(2));assert_eq!(frames.read_into(&mut frame_output),2);assert_eq!(frame_output,[10,11]);assert_eq!(frames.read_into(&mut frame_output),0);
 format!("{{\"zero\":{zero},\"partial\":{:?},\"filled\":{},\"cancelledDestination\":{:?},\"recovered\":{recovered},\"blockingCompletions\":{},\"frameTail\":{:?},\"borrowedIdentity\":true,\"drainWaited\":true,\"deadlineWaited\":true,\"partialError\":{partial_error:?},\"errorFilled\":2}}",output,exact.1,destination,finished.load(Ordering::SeqCst),frame_output)
}
#[cfg(not(profile_probe))]fn main(){println!("{}",tsrust_native_futures::block_on(contracts()));}
