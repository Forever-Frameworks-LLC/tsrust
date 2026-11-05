#[test]
fn exact_short_zero_eof_and_explicit_bounds_preserve_observable_progress(){
 crate::tsrust_native_futures::block_on(async{
  let mut reader=Reader::bytes(b"abc".to_vec());let mut empty=[];assert_eq!(reader.read_into(&mut empty).await.unwrap(),0);
  let mut destination=[9;5];assert_eq!(reader.read_exact(&mut destination).await.unwrap_err().code,"unexpected-eof");assert_eq!(&destination,b"abc\x09\x09");
  let mut reader=Reader::bytes(b"abc".to_vec());assert_eq!(reader.read_all(Some(2)).await.unwrap_err().code,"limit");assert!(reader.read(1).await.unwrap().is_empty());
  let mut reader=Reader::bytes(b"abc".to_vec());assert_eq!(reader.read(-1).await.unwrap_err().code,"range");assert_eq!(reader.read_all(None).await.unwrap(),b"abc");
 });
}
#[test]
fn lines_distinguish_empty_final_and_eof_and_validate_raw_byte_limit(){
 crate::tsrust_native_futures::block_on(async{
  let mut lines=LineReader::new(Reader::bytes("é\r\n\nlast".as_bytes().to_vec()),None).unwrap();
  for expected in [Some("é"),Some(""),Some("last"),None]{assert_eq!(lines.read_line().await.unwrap().as_deref(),expected);}
  let mut lines=LineReader::new(Reader::bytes(b"ab\n".to_vec()),Some(2)).unwrap();assert_eq!(lines.read_line().await.unwrap_err().code,"limit");assert_eq!(lines.read_line().await.unwrap(),None);
  assert!(matches!(LineReader::new(Reader::bytes(vec![]),Some(-1)),Err(error)if error.code=="range"));
 });
}
struct WakeSignal(tokio::sync::Notify);
impl std::task::Wake for WakeSignal{fn wake(self:Arc<Self>){self.0.notify_one();}fn wake_by_ref(self:&Arc<Self>){self.0.notify_one();}}
struct Release(Option<std::sync::mpsc::Sender<()>>);
impl Drop for Release{fn drop(&mut self){if let Some(release)=self.0.take(){let _=release.send(());}}}
struct Gated{started:Option<tokio::sync::oneshot::Sender<()>>,release:std::sync::mpsc::Receiver<()>,bytes:std::io::Cursor<Vec<u8>>,close:Option<(tokio::sync::oneshot::Sender<()>,std::sync::mpsc::Receiver<()>)>,closed:Arc<std::sync::atomic::AtomicUsize>,fail:bool}
impl Drop for Gated{fn drop(&mut self){if let Some((started,release))=self.close.take(){let _=started.send(());release.recv_timeout(std::time::Duration::from_secs(5)).expect("close release deadline");}self.closed.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
impl Read for Gated{fn read(&mut self,destination:&mut[u8])->std::io::Result<usize>{if let Some(started)=self.started.take(){let _=started.send(());self.release.recv_timeout(std::time::Duration::from_secs(5)).expect("read release deadline");}if self.fail{return Err(io::ErrorKind::BrokenPipe.into());}Read::read(&mut self.bytes,destination)}}
#[test]
fn cancelled_blocking_read_waits_actual_completion_and_retains_unwritten_bytes(){
 crate::tsrust_native_futures::block_on(async{
  let(started,observed)=tokio::sync::oneshot::channel();let(release,wait)=std::sync::mpsc::channel();
  let mut reader=Reader{inner:ReadBackend::Blocking(Blocking::new(Backend::Controlled(Box::new(Gated{started:Some(started),release:wait,bytes:std::io::Cursor::new(b"abc".to_vec()),close:None,closed:Arc::new(std::sync::atomic::AtomicUsize::new(0)),fail:false}))))};
  let mut release=Release(Some(release));
  let mut destination=[9;3];let mut read=Box::pin(reader.read_into(&mut destination));
  std::future::poll_fn(|cx|{assert!(read.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;
  tokio::time::timeout(std::time::Duration::from_secs(5),observed).await.expect("read start deadline").unwrap();drop(read);assert_eq!(destination,[9;3]);
  let state=match &reader.inner{ReadBackend::Blocking(reader)=>reader.state.clone(),_=>unreachable!()};
  let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);let cleanup_waker=loop{if let Some(waker)=state.lock().unwrap().cleanup_waker.clone(){break waker;}assert!(std::time::Instant::now()<deadline,"cleanup poll deadline");tokio::task::yield_now().await;};
  let signal=Arc::new(WakeSignal(tokio::sync::Notify::new()));let waker=std::task::Waker::from(signal.clone());let mut context=Context::from_waker(&waker);let mut resumed=Box::pin(reader.read_into(&mut destination));assert!(resumed.as_mut().poll(&mut context).is_pending());
  // Force cleanup to be the JoinHandle's latest poller. Completing it must also wake the operation.
  assert!(Blocking::settle(&state,&mut Context::from_waker(&cleanup_waker),true).is_pending());
  let _=release.0.take().unwrap().send(());tokio::time::timeout(std::time::Duration::from_secs(5),signal.0.notified()).await.expect("operation did not receive its completion wake");
  assert!(matches!(resumed.as_mut().poll(&mut context),Poll::Ready(Ok(3))));drop(resumed);assert_eq!(&destination,b"abc");reader.close().await.unwrap();
 });
}

struct ShortInput{bytes:std::io::Cursor<Vec<u8>>,interrupted:bool}
impl AsyncRead for ShortInput{fn poll_read(mut self:Pin<&mut Self>,_:&mut Context<'_>,destination:&mut tokio::io::ReadBuf<'_>)->Poll<io::Result<()>>{if destination.remaining()==0{return Poll::Ready(Ok(()));}if !self.interrupted{self.interrupted=true;return Poll::Ready(Err(io::ErrorKind::Interrupted.into()));}let mut byte=[0];let count=Read::read(&mut self.bytes,&mut byte)?;if count!=0&&destination.remaining()!=0{destination.put_slice(&byte[..count]);}Poll::Ready(Ok(()))}}
struct ShortOutput{bytes:Arc<Mutex<Vec<u8>>>,zero:bool}
impl AsyncWrite for ShortOutput{
 fn poll_write(self:Pin<&mut Self>,_:&mut Context<'_>,bytes:&[u8])->Poll<io::Result<usize>>{let count=if self.zero{0}else{bytes.len().min(2)};self.bytes.lock().unwrap().extend_from_slice(&bytes[..count]);Poll::Ready(Ok(count))}
 fn poll_flush(self:Pin<&mut Self>,_:&mut Context<'_>)->Poll<io::Result<()>>{Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))}
 fn poll_shutdown(self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<io::Result<()>>{self.poll_flush(cx)}
}
#[test]
fn short_interrupted_utf8_copy_and_flush_failures_remain_explicit(){
 crate::tsrust_native_futures::block_on(async{
  let input=ShortInput{bytes:std::io::Cursor::new("é\r\nlast".as_bytes().to_vec()),interrupted:false};let mut lines=LineReader::new(Reader::asynchronous(Box::new(input)),None).unwrap();assert_eq!(lines.read_line().await.unwrap().as_deref(),Some("é"));assert_eq!(lines.read_line().await.unwrap().as_deref(),Some("last"));assert!(lines.read_line().await.unwrap().is_none());
  let mut malformed=LineReader::new(Reader::bytes(vec![255,b'\n']),None).unwrap();assert_eq!(malformed.read_line().await.unwrap_err().code,"encoding");
  let bytes=Arc::new(Mutex::new(Vec::new()));let mut output=Writer::asynchronous(Box::new(ShortOutput{bytes:bytes.clone(),zero:false}));let mut input=Reader::bytes(b"abcdef".to_vec());assert_eq!(copy(&mut input,&mut output,3,Some(4)).await.unwrap_err().code,"limit");assert_eq!(&*bytes.lock().unwrap(),b"abcd");assert_eq!(input.read_all(None).await.unwrap(),b"f");assert_eq!(output.flush().await.unwrap_err().code,"broken-pipe");assert_eq!(output.close().await.unwrap_err().code,"broken-pipe");
  let mut output=Writer::asynchronous(Box::new(ShortOutput{bytes:Arc::new(Mutex::new(Vec::new())),zero:true}));assert_eq!(output.write_all(b"x").await.unwrap_err().code,"write-zero");
 });
}
#[test]
fn direct_reader_polling_observes_eof_after_pending_and_ready_reads(){
 crate::tsrust_native_futures::block_on(async{let mut reader=Reader::bytes(b"abc".to_vec());let mut destination=Vec::new();AsyncReadExt::read_to_end(&mut reader,&mut destination).await.unwrap();assert_eq!(destination,b"abc");});
}

#[test]
fn owning_read_drop_drains_both_pending_read_and_actual_close(){
 crate::tsrust_native_futures::block_on(async{
  let(started,observed)=tokio::sync::oneshot::channel();let(release_read,wait_read)=std::sync::mpsc::channel();let(close_started,saw_close)=tokio::sync::oneshot::channel();let(release_close,wait_close)=std::sync::mpsc::channel();let closed=Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let reader=Reader{inner:ReadBackend::Blocking(Blocking::new(Backend::Controlled(Box::new(Gated{started:Some(started),release:wait_read,bytes:std::io::Cursor::new(b"abc".to_vec()),close:Some((close_started,wait_close)),closed:closed.clone(),fail:false}))))};
  let mut read_release=Release(Some(release_read));let mut close_release=Release(Some(release_close));let scope=crate::tsrust_native_futures::Scope::group();
  crate::tsrust_native_futures::in_scope(scope.clone(),async move{let mut operation=Box::pin(async move{let mut reader=reader;let mut byte=[0];AsyncReadExt::read(&mut reader,&mut byte).await});std::future::poll_fn(|cx|{assert!(operation.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;tokio::time::timeout(std::time::Duration::from_secs(5),observed).await.expect("owning read start deadline").unwrap();drop(operation);}).await;
  let mut drain=Box::pin(scope.drain(true));std::future::poll_fn(|cx|{assert!(drain.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;let _=read_release.0.take().unwrap().send(());
  tokio::time::timeout(std::time::Duration::from_secs(5),saw_close).await.expect("close start deadline").unwrap();std::future::poll_fn(|cx|{assert!(drain.as_mut().poll(cx).is_pending(),"drain returned before close completed");Poll::Ready(())}).await;assert_eq!(closed.load(std::sync::atomic::Ordering::SeqCst),0);
  let _=close_release.0.take().unwrap().send(());tokio::time::timeout(std::time::Duration::from_secs(5),drain).await.expect("actual close settlement deadline");assert_eq!(closed.load(std::sync::atomic::Ordering::SeqCst),1);
 });
}

#[test]
fn explicit_close_releases_owner_before_reporting_a_deferred_read_failure(){
 crate::tsrust_native_futures::block_on(async{
  let(started,observed)=tokio::sync::oneshot::channel();let(release_read,wait_read)=std::sync::mpsc::channel();let(close_started,saw_close)=tokio::sync::oneshot::channel();let(release_close,wait_close)=std::sync::mpsc::channel();let closed=Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let mut reader=Reader{inner:ReadBackend::Blocking(Blocking::new(Backend::Controlled(Box::new(Gated{started:Some(started),release:wait_read,bytes:std::io::Cursor::new(vec![]),close:Some((close_started,wait_close)),closed:closed.clone(),fail:true}))))};let mut read_release=Release(Some(release_read));let mut close_release=Release(Some(release_close));
  let state=match &reader.inner{ReadBackend::Blocking(reader)=>reader.state.clone(),_=>unreachable!()};let mut bytes=[0];let mut read=Box::pin(reader.read_into(&mut bytes));std::future::poll_fn(|cx|{assert!(read.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;tokio::time::timeout(std::time::Duration::from_secs(5),observed).await.expect("failing read start deadline").unwrap();drop(read);let _=read_release.0.take().unwrap().send(());
  let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);loop{if state.lock().unwrap().failure.is_some(){break;}assert!(std::time::Instant::now()<deadline,"deferred failure deadline");tokio::task::yield_now().await;}
  let mut close=Box::pin(reader.close());std::future::poll_fn(|cx|{assert!(close.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;tokio::time::timeout(std::time::Duration::from_secs(5),saw_close).await.expect("explicit close start deadline").unwrap();std::future::poll_fn(|cx|{assert!(close.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;assert_eq!(closed.load(std::sync::atomic::Ordering::SeqCst),0);
  let _=close_release.0.take().unwrap().send(());let failure=tokio::time::timeout(std::time::Duration::from_secs(5),close).await.expect("explicit close settlement deadline").unwrap_err();assert_eq!(failure.code,"broken-pipe");assert_eq!(closed.load(std::sync::atomic::Ordering::SeqCst),1);
 });
}
