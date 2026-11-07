//! The admitted bridge contracts exercise the actual public streaming runtime.
mod support;
#[path="../src/stdlib/fs/runtime/resource.rs"]mod resource;
#[path="../src/stdlib/fs/runtime/fs.rs"]pub mod fs;
#[path="../src/stdlib/fs/runtime/buffered_fs.rs"]pub mod buffered_fs;
mod tsrust_native_files{pub use crate::{fs,buffered_fs};}

#[cfg(feature="logging")] include!("../src/stdlib/logging/runtime.rs");
include!("../src/stdlib/futures/runtime.rs");
include!("../src/stdlib/io/runtime.rs");
mod tsrust_native_threads{include!("../src/stdlib/threads/runtime_core.rs");include!("../src/stdlib/threads/runtime_async.rs");}
include!("../src/stdlib/compression/runtime.rs");
include!("../src/stdlib/compression/runtime_streams.rs");
use tsrust_native_compression_streams as bridge;
mod contracts{
// Focused source-only bridge witnesses. Execute only with an exclusive lease.
use crate::{bridge::{self,Failure,TransformReader,TransformWriter,Format,Direction},tsrust_native_io as io_backend,tsrust_native_threads as pool,tsrust_native_futures as tasks};
use std::{sync::{Arc,Mutex,atomic::{AtomicBool,AtomicUsize,Ordering}},future::Future,pin::Pin,task::{Context,Poll,Waker},io};
use tokio::io::{AsyncRead,AsyncWrite,ReadBuf};
mod baseline{
 include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../benchmarks/runtime/admission/compression/reference.rs"));
 pub(super) fn encoded(format:&str,input:&[u8])->Vec<u8>{compress(Format::parse(format),input,-1,64,Strategy::Writer).unwrap()}
 pub(super) fn decoded(format:&str,input:&[u8])->Vec<u8>{decompress(Format::parse(format),input,1,None).unwrap()}
 pub(super) fn source(size:usize)->Vec<u8>{input(size,true,7)}
 pub(super) fn metadata(input:&[u8])->Vec<u8>{let mut encoder=flate2::GzBuilder::new().filename("owned-name.bin").comment("split metadata").mtime(0).operating_system(255).write(Vec::new(),flate2::Compression::default());encoder.write_all(input).unwrap();encoder.finish().unwrap()}
}
fn need<T>(result:Result<T,Failure>)->T{match result{Ok(value)=>value,Err(Failure::Codec(error))=>panic!("codec: {error}"),Err(Failure::Io(error))=>panic!("I/O: {error}"),Err(Failure::Pool(_))=>panic!("pool"),Err(Failure::Closed)=>panic!("closed")}}
struct IoControl{blocked:AtomicBool,calls:AtomicUsize,drops:AtomicUsize,waker:Mutex<Option<Waker>>,bytes:Mutex<Vec<u8>>,fail_write:AtomicBool,fail_flush:AtomicBool,fail_close:AtomicBool}
impl IoControl{fn new()->Arc<Self>{Arc::new(Self{blocked:AtomicBool::new(false),calls:AtomicUsize::new(0),drops:AtomicUsize::new(0),waker:Mutex::new(None),bytes:Mutex::new(Vec::new()),fail_write:AtomicBool::new(false),fail_flush:AtomicBool::new(false),fail_close:AtomicBool::new(false)})}fn release(&self){self.blocked.store(false,Ordering::SeqCst);if let Some(waker)=self.waker.lock().unwrap().take(){waker.wake();}}fn waiting(&self,cx:&Context<'_>)->bool{if self.blocked.load(Ordering::SeqCst){*self.waker.lock().unwrap()=Some(cx.waker().clone());true}else{false}}}
struct ShortRead{bytes:Vec<u8>,position:usize,width:usize,interrupt:bool,control:Arc<IoControl>}
impl Drop for ShortRead{fn drop(&mut self){self.control.drops.fetch_add(1,Ordering::SeqCst);}}
impl AsyncRead for ShortRead{fn poll_read(mut self:Pin<&mut Self>,cx:&mut Context<'_>,out:&mut ReadBuf<'_>)->Poll<io::Result<()>>{if self.control.waiting(cx){return Poll::Pending;}self.control.calls.fetch_add(1,Ordering::SeqCst);if self.interrupt{self.interrupt=false;return Poll::Ready(Err(io::ErrorKind::Interrupted.into()));}let count=out.remaining().min(self.width).min(self.bytes.len()-self.position);out.put_slice(&self.bytes[self.position..self.position+count]);self.position+=count;Poll::Ready(Ok(()))}}
struct ShortWrite{width:usize,interrupt:bool,control:Arc<IoControl>}
impl Drop for ShortWrite{fn drop(&mut self){self.control.drops.fetch_add(1,Ordering::SeqCst);}}
impl AsyncWrite for ShortWrite{
 fn poll_write(mut self:Pin<&mut Self>,cx:&mut Context<'_>,bytes:&[u8])->Poll<io::Result<usize>>{if self.control.waiting(cx){return Poll::Pending;}self.control.calls.fetch_add(1,Ordering::SeqCst);if self.control.fail_write.load(Ordering::SeqCst){return Poll::Ready(Err(io::Error::from_raw_os_error(32)));}if self.interrupt{self.interrupt=false;return Poll::Ready(Err(io::ErrorKind::Interrupted.into()));}let count=bytes.len().min(self.width);self.control.bytes.lock().unwrap().extend_from_slice(&bytes[..count]);Poll::Ready(Ok(count))}
 fn poll_flush(self:Pin<&mut Self>,_:&mut Context<'_>)->Poll<io::Result<()>>{Poll::Ready(if self.control.fail_flush.load(Ordering::SeqCst){Err(io::Error::from_raw_os_error(32))}else{Ok(())})}
 fn poll_shutdown(self:Pin<&mut Self>,_:&mut Context<'_>)->Poll<io::Result<()>>{Poll::Ready(if self.control.fail_close.load(Ordering::SeqCst){Err(io::Error::from_raw_os_error(32))}else{Ok(())})}
}
fn source(bytes:Vec<u8>,width:usize)->(io_backend::Reader,Arc<IoControl>){let control=IoControl::new();(io_backend::Reader::asynchronous(Box::new(ShortRead{bytes,position:0,width,interrupt:true,control:control.clone()})),control)}
fn sink()->(io_backend::Writer,Arc<IoControl>){let control=IoControl::new();(io_backend::Writer::asynchronous(Box::new(ShortWrite{width:3,interrupt:true,control:control.clone()})),control)}
async fn read_all(reader:&mut TransformReader,width:usize)->Result<Vec<u8>,Failure>{let mut output=Vec::new();let mut chunk=vec![0;width];loop{let count=reader.read_into(&mut chunk).await? as usize;if count==0{return Ok(output);}output.extend_from_slice(&chunk[..count]);}}
async fn write_all(writer:&mut TransformWriter,bytes:&[u8],width:usize)->Result<(),Failure>{let mut offset=0;while offset<bytes.len(){let end=(offset+width).min(bytes.len());let count=writer.write(&bytes[offset..end]).await? as usize;assert!(count>0);offset+=count;}Ok(())}
async fn pending<F:Future>(future:Pin<&mut F>){let mut future=future;std::future::poll_fn(|cx|{assert!(future.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;}
struct Release(Option<std::sync::mpsc::Sender<()>>);impl Release{fn send(&mut self){if let Some(send)=self.0.take(){let _=send.send(());}}}impl Drop for Release{fn drop(&mut self){self.send();}}
async fn formats(){
 let pool=pool::create_pool(1,1).unwrap();
 for(format,name)in[(Format::Gzip,"gzip"),(Format::Zlib,"zlib"),(Format::Raw,"raw")]{for size in[0,1,64,8193]{for width in[1,7,8192]{
  let original=baseline::source(size);let encoded=baseline::encoded(name,&original);
  for(direction,input,expected)in[(Direction::Encode,original.clone(),encoded.clone()),(Direction::Decode,encoded.clone(),original.clone())]{
   let(source,control)=self::source(input.clone(),width);let mut reader=TransformReader::new(source,pool.submission(),format,direction,None,None).unwrap();let output=need(read_all(&mut reader,5).await);if matches!(direction,Direction::Encode){assert_eq!(baseline::decoded(name,&output),original);}else{assert_eq!(output,expected);}need(reader.finish().await).close().await.unwrap();assert_eq!(control.drops.load(Ordering::SeqCst),1);
   let(sink,control)=self::sink();let mut writer=TransformWriter::new(sink,pool.submission(),format,direction,None,None).unwrap();need(write_all(&mut writer,&input,width).await);need(writer.close().await);let output=control.bytes.lock().unwrap();if matches!(direction,Direction::Encode){assert_eq!(baseline::decoded(name,&output),original);}else{assert_eq!(*output,expected);}assert_eq!(control.drops.load(Ordering::SeqCst),1);assert_eq!(pool.stats().outstanding,0);
  }
 }}}
 let original=baseline::source(8193);let first=baseline::metadata(&original);let second=baseline::encoded("gzip",b"second");let members=[first,second].concat();let(source,_)=self::source(members,1);let mut reader=TransformReader::new(source,pool.submission(),Format::Gzip,Direction::Decode,None,None).unwrap();let output=need(read_all(&mut reader,1).await);let mut expected=original;expected.extend_from_slice(b"second");assert_eq!(output,expected);need(reader.close().await);
 println!("{{\"group\":\"formats\",\"readerWriter\":true,\"shortInterruptedIo\":true,\"gzipMetadataMembers\":true,\"poolOne\":true}}");
}
async fn errors(){
 let pool=pool::create_pool(1,1).unwrap();
 for(format,name)in[(Format::Gzip,"gzip"),(Format::Zlib,"zlib"),(Format::Raw,"raw")]{
  let original=baseline::source(64);let encoded=baseline::encoded(name,&original);
  for limit in[0,63,64]{let(source,_)=self::source(encoded.clone(),1);let mut reader=TransformReader::new(source,pool.submission(),format,Direction::Decode,None,Some(limit)).unwrap();let output=read_all(&mut reader,7).await;if limit==64{assert_eq!(need(output),original);need(reader.close().await);}else{assert!(matches!(output,Err(Failure::Codec(_))));}}
  for limit in[0,63,64]{let(sink,control)=self::sink();let mut writer=TransformWriter::new(sink,pool.submission(),format,Direction::Decode,None,Some(limit)).unwrap();let result=match write_all(&mut writer,&encoded,1).await{Ok(())=>writer.close().await,Err(error)=>Err(error)};if limit==64{need(result);assert_eq!(*control.bytes.lock().unwrap(),original);}else{assert!(matches!(result,Err(Failure::Codec(_))));assert!(control.bytes.lock().unwrap().len()<=limit as usize);}}
  for broken in[encoded[..encoded.len()-1].to_vec(),[encoded.clone(),b"tail".to_vec()].concat()]{
   let(source,_)=self::source(broken.clone(),1);let reader=TransformReader::new(source,pool.submission(),format,Direction::Decode,None,None).unwrap();assert!(matches!(reader.finish().await,Err(Failure::Codec(_))));
   let(sink,_)=self::sink();let mut writer=TransformWriter::new(sink,pool.submission(),format,Direction::Decode,None,None).unwrap();match write_all(&mut writer,&broken,1).await{Ok(())=>assert!(matches!(writer.finish().await,Err(Failure::Codec(_)))),Err(error)=>assert!(matches!(error,Failure::Codec(_)))}
  }
 }
 let original=baseline::encoded("gzip",b"checksum");let mut corrupt=original.clone();let crc=corrupt.len()-8;corrupt[crc]^=1;let(source,_)=self::source(corrupt,1);let reader=TransformReader::new(source,pool.submission(),Format::Gzip,Direction::Decode,None,None).unwrap();assert!(matches!(reader.finish().await,Err(Failure::Codec(_))));
 for failure in["write","flush","close"]{let(sink,control)=self::sink();match failure{"write"=>control.fail_write.store(true,Ordering::SeqCst),"flush"=>control.fail_flush.store(true,Ordering::SeqCst),_=>control.fail_close.store(true,Ordering::SeqCst)}let mut writer=TransformWriter::new(sink,pool.submission(),Format::Gzip,Direction::Encode,None,None).unwrap();let result=match writer.write(b"I/O failure").await{Ok(_)=>writer.close().await,Err(error)=>Err(error)};assert!(matches!(result,Err(Failure::Io(error))if error.os_code==Some(32)));}
 println!("{{\"group\":\"errors\",\"incrementalPolicy\":true,\"truncationTrailingChecksum\":true,\"finishFailures\":true,\"typedIo\":true}}");
}
async fn backpressure(){
 let pool=pool::create_pool(1,1).unwrap();let original=baseline::source(32768);let(source,source_control)=self::source(baseline::encoded("gzip",&original),8192);let(sink,sink_control)=self::sink();sink_control.blocked.store(true,Ordering::SeqCst);
 let mut reader=TransformReader::new(source,pool.submission(),Format::Gzip,Direction::Decode,None,None).unwrap();let mut writer=TransformWriter::new(sink,pool.submission(),Format::Gzip,Direction::Encode,None,None).unwrap();
 let copy=async{let mut chunk=[0;8192];loop{let count=reader.read_into(&mut chunk).await? as usize;if count==0{break;}write_all(&mut writer,&chunk[..count],8192).await?;}Ok::<_,Failure>(())};let mut copy=Box::pin(copy);
 while sink_control.waker.lock().unwrap().is_none(){std::future::poll_fn(|cx|{match copy.as_mut().poll(cx){Poll::Ready(_)=>panic!("sink was expected to block"),Poll::Pending=>Poll::Ready(())}}).await;tokio::task::yield_now().await;}
 assert_eq!(pool.stats().running,0);assert_eq!(pool.stats().outstanding,0);let reads=source_control.calls.load(Ordering::SeqCst);for _ in 0..4{pending(copy.as_mut()).await;}assert_eq!(source_control.calls.load(Ordering::SeqCst),reads);sink_control.release();need(copy.as_mut().await);drop(copy);need(reader.close().await);need(writer.close().await);assert_eq!(baseline::decoded("gzip",&sink_control.bytes.lock().unwrap()),original);
 let(source,control)=self::source(b"input".to_vec(),1);control.blocked.store(true,Ordering::SeqCst);let mut reader=TransformReader::new(source,pool.submission(),Format::Zlib,Direction::Encode,None,None).unwrap();let mut destination=[0;7];let read=reader.read_into(&mut destination);let mut read=Box::pin(read);while control.waker.lock().unwrap().is_none(){pending(read.as_mut()).await;tokio::task::yield_now().await;}assert_eq!(pool.stats().outstanding,0);control.release();need(read.as_mut().await);drop(read);drop(reader);
 println!("{{\"group\":\"backpressure\",\"noCpuWorkerOnIo\":true,\"noUpstreamReadAhead\":true,\"poolOneComposition\":true}}");
}
async fn cancellation(){
 let scope=Arc::new(tasks::Scope::default());tasks::in_scope(scope.clone(),async{
  let pool=pool::create_pool(1,1).unwrap();let original=baseline::source(8193);let(source,control)=self::source(original.clone(),8192);let mut reader=TransformReader::new(source,pool.submission(),Format::Zlib,Direction::Encode,None,None).unwrap();
  let mut zero=[];assert_eq!(need(reader.read_into(&mut zero).await),0);assert_eq!(pool.stats().outstanding,0);assert_eq!(control.calls.load(Ordering::SeqCst),0);
  need(bridge::initialize_reader(&mut reader).await);
  let drops=Arc::new(AtomicUsize::new(0));let(started,observed)=tokio::sync::oneshot::channel();let(send,receive)=std::sync::mpsc::channel();let mut release=Release(Some(send));bridge::gate_reader(&mut reader,bridge::Gate{started,release:receive},drops.clone());
  {let mut destination=[41;7];let mut read=Box::pin(reader.read_into(&mut destination));pending(read.as_mut()).await;observed.await.unwrap();drop(read);assert_eq!(destination,[41;7]);}
  assert_eq!(pool.stats().running,1);let mut drain=Box::pin(scope.drain(false));pending(drain.as_mut()).await;assert_eq!(drops.load(Ordering::SeqCst),0);release.send();drain.await;assert_eq!(pool.stats().outstanding,0);assert!(matches!(reader.read_into(&mut[0;7]).await,Err(Failure::Pool(pool::AsyncTaskError::Completion(pool::JoinError::Cancelled)))));
  let encoded=need(read_all(&mut reader,7).await);assert_eq!(baseline::decoded("zlib",&encoded),original);need(reader.close().await);assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(control.drops.load(Ordering::SeqCst),1);
  let(source,control)=self::source(b"lazy".to_vec(),1);let mut reader=TransformReader::new(source,pool.submission(),Format::Zlib,Direction::Encode,None,None).unwrap();let mut destination=[0;7];drop(reader.read_into(&mut destination));assert_eq!(pool.stats().outstanding,0);assert_eq!(control.calls.load(Ordering::SeqCst),0);drop(reader);assert_eq!(control.drops.load(Ordering::SeqCst),1);
 }).await;
 println!("{{\"group\":\"cancellation\",\"actualCpuSettlement\":true,\"restoredCodecState\":true,\"noDestinationAfterDrop\":true,\"lazyNoAdmission\":true}}");
}

async fn queued(){
 let scope=Arc::new(tasks::Scope::default());tasks::in_scope(scope.clone(),async{
  let pool=pool::create_pool(1,2).unwrap();let original=baseline::source(64);let(source,control)=self::source(original.clone(),1);let mut reader=TransformReader::new(source,pool.submission(),Format::Zlib,Direction::Encode,None,None).unwrap();need(bridge::initialize_reader(&mut reader).await);let(started,observed)=tokio::sync::oneshot::channel();let(send,receive)=std::sync::mpsc::channel();let mut release=Release(Some(send));
  let mut blocker=Box::pin(pool::task_async(pool.submission(),move||{let _=started.send(());receive.recv_timeout(std::time::Duration::from_secs(5)).unwrap();Ok::<_,()>(())}));pending(blocker.as_mut()).await;observed.await.unwrap();
  let(did_start,mut saw_start)=tokio::sync::oneshot::channel();let(send_gate,gate_receive)=std::sync::mpsc::channel();let mut release_gate=Release(Some(send_gate));let drops=Arc::new(AtomicUsize::new(0));bridge::gate_reader(&mut reader,bridge::Gate{started:did_start,release:gate_receive},drops.clone());
  let mut destination=[41;7];let mut read=Box::pin(reader.read_into(&mut destination));pending(read.as_mut()).await;assert_eq!(pool.stats().outstanding,2);drop(read);assert_eq!(destination,[41;7]);assert!(matches!(saw_start.try_recv(),Err(tokio::sync::oneshot::error::TryRecvError::Empty)));release.send();assert!(blocker.await.is_ok());scope.drain(false).await;assert_eq!(pool.stats().outstanding,0);assert!(control.calls.load(Ordering::SeqCst)>0);assert!(matches!(saw_start.try_recv(),Err(tokio::sync::oneshot::error::TryRecvError::Empty)));
  release_gate.send();assert!(matches!(reader.read_into(&mut[0;7]).await,Err(Failure::Pool(pool::AsyncTaskError::Completion(pool::JoinError::Cancelled)))));let encoded=need(read_all(&mut reader,7).await);assert_eq!(baseline::decoded("zlib",&encoded),original);assert!(saw_start.try_recv().is_ok());need(reader.close().await);assert_eq!(drops.load(Ordering::SeqCst),1);
 }).await;
 println!("{{\"group\":\"queued\",\"cancelBeforeStart\":true,\"retainedQueuedInput\":true,\"restoredInputs\":true,\"capacityReleased\":true}}");
}
async fn files(directory:&std::path::Path){let pool=pool::create_pool(1,1).unwrap();let original=baseline::source(32768);let input=directory.join("input.bin");let output=directory.join("output.bin");std::fs::write(&input,&original).unwrap();let source=io_backend::Reader::file(crate::tsrust_native_files::fs::open(input.to_str().unwrap()).unwrap());let sink=io_backend::Writer::file(crate::tsrust_native_files::fs::create(output.to_str().unwrap()).unwrap());let mut reader=TransformReader::new(source,pool.submission(),Format::Gzip,Direction::Encode,None,None).unwrap();let mut writer=TransformWriter::new(sink,pool.submission(),Format::Gzip,Direction::Decode,None,None).unwrap();let mut chunk=[0;8192];loop{let count=need(reader.read_into(&mut chunk).await)as usize;if count==0{break;}need(write_all(&mut writer,&chunk[..count],8192).await);}need(reader.close().await);need(writer.close().await);assert_eq!(std::fs::read(output).unwrap(),original);assert_eq!(pool.stats().outstanding,0);println!("{{\"group\":\"files\",\"actualB05FilePipeline\":true,\"poolOne\":true,\"explicitClose\":true}}");}
// Independent incremental native decoder: a Sync prefix must contain the entire
// first payload, while still reporting a live (not StreamEnd) single stream.
fn prefix(format:Format,bytes:&[u8],expected:&[u8]){
 let mut codec=match format{Format::Gzip=>flate2::Decompress::new_gzip(15),Format::Zlib=>flate2::Decompress::new(true),Format::Raw=>flate2::Decompress::new(false)};let mut output=vec![0;expected.len()+64];let status=codec.decompress(bytes,&mut output,flate2::FlushDecompress::None).unwrap();assert_ne!(status,flate2::Status::StreamEnd);assert_eq!(codec.total_in()as usize,bytes.len());output.truncate(codec.total_out()as usize);assert_eq!(output,expected);
}
async fn flush(){
 let pool=pool::create_pool(1,1).unwrap();let first=baseline::source(8193);let second=b"after sync flush";
 for(format,name)in[(Format::Gzip,"gzip"),(Format::Zlib,"zlib"),(Format::Raw,"raw")]{
  let(sink,control)=self::sink();let mut writer=TransformWriter::new(sink,pool.submission(),format,Direction::Encode,None,None).unwrap();need(write_all(&mut writer,&first,7).await);need(writer.flush().await);let before=control.bytes.lock().unwrap().clone();prefix(format,&before,&first);assert_eq!(pool.stats().outstanding,0);
  need(writer.flush().await);prefix(format,&control.bytes.lock().unwrap(),&first);need(write_all(&mut writer,second,1).await);need(writer.flush().await);let expected=[first.as_slice(),second].concat();prefix(format,&control.bytes.lock().unwrap(),&expected);need(writer.close().await);let completed=control.bytes.lock().unwrap().clone();assert_eq!(baseline::decoded(name,&completed),expected);
  let mut native=match format{Format::Gzip=>flate2::Decompress::new_gzip(15),Format::Zlib=>flate2::Decompress::new(true),Format::Raw=>flate2::Decompress::new(false)};let mut decoded=vec![0;expected.len()+64];assert_eq!(native.decompress(&completed,&mut decoded,flate2::FlushDecompress::None).unwrap(),flate2::Status::StreamEnd);assert_eq!(native.total_in()as usize,completed.len());assert_eq!(native.total_out()as usize,expected.len());assert_eq!(&decoded[..expected.len()],expected);
  if matches!(format,Format::Gzip){let os=if cfg!(windows){10}else if cfg!(target_os="macos"){19}else{3};assert_eq!(&completed[..10],&[31,139,8,0,0,0,0,0,0,os]);let(sink,control2)=self::sink();let mut repeat=TransformWriter::new(sink,pool.submission(),format,Direction::Encode,None,None).unwrap();need(write_all(&mut repeat,&first,7).await);need(repeat.flush().await);need(repeat.flush().await);need(write_all(&mut repeat,second,1).await);need(repeat.flush().await);need(repeat.close().await);assert_eq!(*control2.bytes.lock().unwrap(),completed);}
 }
 println!("{{\"group\":\"flush\",\"syncReadableBeforeFinish\":true,\"continueSameMember\":true,\"nativeDeterministicHeader\":true,\"allFormats\":true}}");
}
async fn disposal(){
 for format in[Format::Gzip,Format::Zlib,Format::Raw]{for initialized in[false,true]{
  let pool=pool::create_pool(1,1).unwrap();let(sink,control)=self::sink();let mut writer=TransformWriter::new(sink,pool.submission(),format,Direction::Encode,None,None).unwrap();let drops=Arc::new(AtomicUsize::new(0));bridge::watch_writer(&mut writer,drops.clone());if initialized{need(write_all(&mut writer,b"unfinished payload",1).await);need(writer.flush().await);prefix(format,&control.bytes.lock().unwrap(),b"unfinished payload");}let before=control.bytes.lock().unwrap().clone();let calls=control.calls.load(Ordering::SeqCst);assert_eq!(pool.stats().outstanding,0);pool.shutdown().unwrap();drop(writer);assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(Arc::strong_count(&drops),1);assert_eq!(*control.bytes.lock().unwrap(),before);assert_eq!(control.calls.load(Ordering::SeqCst),calls);assert_eq!(control.drops.load(Ordering::SeqCst),1);assert_eq!(pool.stats().outstanding,0);
  let pool=pool::create_pool(1,1).unwrap();let(source,control)=self::source(b"not read".to_vec(),1);let mut reader=TransformReader::new(source,pool.submission(),format,Direction::Encode,None,None).unwrap();let drops=Arc::new(AtomicUsize::new(0));bridge::watch_reader(&mut reader,drops.clone());if initialized{need(bridge::initialize_reader(&mut reader).await);}assert_eq!(control.calls.load(Ordering::SeqCst),0);assert_eq!(pool.stats().outstanding,0);pool.shutdown().unwrap();drop(reader);assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(Arc::strong_count(&drops),1);assert_eq!(control.drops.load(Ordering::SeqCst),1);assert_eq!(pool.stats().outstanding,0);
 }}
 println!("{{\"group\":\"disposal\",\"idleNoAdmission\":true,\"closedPoolDrop\":true,\"initializedStateDropOnce\":true,\"noImplicitFinish\":true}}");
}
pub async fn run(directory:&std::path::Path,selected:&[String]){for group in selected{match group.as_str(){"formats"=>formats().await,"errors"=>errors().await,"backpressure"=>backpressure().await,"cancellation"=>cancellation().await,"queued"=>queued().await,"files"=>files(directory).await,"flush"=>flush().await,"disposal"=>disposal().await,_=>panic!("unknown bridge group")}}}

}
#[test]fn public_compression_streams_formats(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["formats".into()]));}
#[test]fn public_compression_streams_errors(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["errors".into()]));}
#[test]fn public_compression_streams_backpressure(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["backpressure".into()]));}
#[test]fn public_compression_streams_cancellation(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["cancellation".into()]));}
#[test]fn public_compression_streams_queued(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["queued".into()]));}
#[test]fn public_compression_streams_files(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["files".into()]));}
#[test]fn public_compression_streams_flush(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["flush".into()]));}
#[test]fn public_compression_streams_disposal(){let directory=support::TempDirectory::new();tsrust_native_futures::block_on(contracts::run(&directory.0,&["disposal".into()]));}
