mod tsrust_native_compression_streams {
// Owned bounded CPU steps; B05 I/O is awaited only after the pool step settles.
use std::{convert::Infallible,io::{self},sync::{Arc,Mutex}};
use crate::{tsrust_native_io as io_backend,tsrust_native_threads as pool,tsrust_native_compression as backend};
use io_backend::{CopyRead,CopyWrite};
const WORKING:usize=io_backend::BUFFER_SIZE; // Existing B05 working buffer, never a total expansion cap.
pub enum Failure{Codec(backend::Error),Io(io_backend::Error),Pool(pool::AsyncTaskError<Infallible>),Closed}
pub use backend::{Format,Direction};
// Native low-level state drops without implicitly finishing. Gzip framing and
// checksums are entirely owned by flate2's admitted zlib-rs backend.
enum Codec{New{format:Format,direction:Direction,level:flate2::Compression},Encode(flate2::Compress),Decode(flate2::Decompress)}
#[derive(Clone,Copy,PartialEq,Eq)]enum Progress{Advance,Input,Output,Finished}
struct Work{codec:Codec,format:Format,input:Vec<u8>,input_start:usize,input_len:usize,eof:bool,output:Vec<u8>,start:usize,end:usize,decoded:usize,limit:Option<usize>,progress:Progress,failure:Option<backend::Error>,poisoned:bool,member_ended:bool,flush_requested:bool,#[cfg(test)]gate:Option<Gate>,#[cfg(test)]drops:Option<Arc<std::sync::atomic::AtomicUsize>>}
impl Work{
 fn new(format:Format,direction:Direction,level:Option<i64>,limit:Option<i64>)->Result<Self,backend::Error>{
  let level=backend::level(level,"compression")?;let limit=backend::limit(limit,"compression")?;
  Ok(Self{codec:Codec::New{format,direction,level},format,input:vec![0;WORKING],input_start:0,input_len:0,eof:false,output:vec![0;WORKING],start:0,end:0,decoded:0,limit,progress:Progress::Advance,failure:None,poisoned:false,member_ended:false,flush_requested:false,#[cfg(test)]gate:None,#[cfg(test)]drops:None})
 }
 fn step(&mut self)->Result<(),backend::Error>{
  #[cfg(test)]if let Some(gate)=self.gate.take(){let _=gate.started.send(());gate.release.recv_timeout(std::time::Duration::from_secs(5)).expect("synthetic CPU release");}
  assert_eq!(self.start,self.end);self.start=0;self.end=0;
  if let Codec::New{format,direction,level}=&self.codec{self.codec=match direction{Direction::Encode=>Codec::Encode(backend::compressor(*format,*level)),Direction::Decode=>Codec::Decode(backend::decompressor(*format))};}
  let decoding=matches!(&self.codec,Codec::Decode(_));let empty=self.input_start==self.input_len;
  if self.member_ended{
   self.flush_requested=false;
   if empty{self.progress=if self.eof{Progress::Finished}else{Progress::Input};return Ok(());}
   if decoding&&matches!(self.format,Format::Gzip){self.codec=Codec::Decode(backend::decompressor(self.format));self.member_ended=false;}else{return Err(backend::error("trailing-data","decompress","Trailing compressed data"));}
  }
  if empty&&!self.eof&&!self.flush_requested{self.progress=Progress::Input;return Ok(());}
  self.output.resize(WORKING,0);let capacity=if decoding{self.limit.map_or(WORKING,|limit|limit.saturating_sub(self.decoded).max(1).min(WORKING))}else{WORKING};
  let input=&self.input[self.input_start..self.input_len];
  let(status,used,made)=match &mut self.codec{
   Codec::Encode(codec)=>{let before=(codec.total_in(),codec.total_out());let flush=if self.eof{flate2::FlushCompress::Finish}else if self.flush_requested{flate2::FlushCompress::Sync}else{flate2::FlushCompress::None};let status=codec.compress(input,&mut self.output[..capacity],flush).map_err(|error|backend::error("codec","compression",error))?;(status,(codec.total_in()-before.0)as usize,(codec.total_out()-before.1)as usize)},
   Codec::Decode(codec)=>{let before=(codec.total_in(),codec.total_out());let status=codec.decompress(input,&mut self.output[..capacity],if self.flush_requested{flate2::FlushDecompress::Sync}else{flate2::FlushDecompress::None}).map_err(|error|backend::error("codec","compression",error))?;(status,(codec.total_in()-before.0)as usize,(codec.total_out()-before.1)as usize)},Codec::New{..}=>unreachable!()
  };
  self.input_start+=used;assert!(self.input_start<=self.input_len);assert!(made<=capacity);
  if decoding{if let Some(limit)=self.limit{if made>limit.saturating_sub(self.decoded){return Err(backend::error("output-limit","decompress","Explicit decompressed-byte limit exceeded"));}self.decoded+=made;}}
  if status==flate2::Status::StreamEnd{self.member_ended=true;}
  // Native Sync is complete when all input is consumed and output space remains.
  // Keep requesting Sync across full output chunks, without forcing stream EOF.
  if self.flush_requested&&self.input_start==self.input_len&&made<capacity{self.flush_requested=false;}
  self.end=made;
  if made>0{self.progress=Progress::Output;return Ok(());}
  if self.member_ended{self.progress=Progress::Advance;return Ok(());}
  if used==0&&self.eof{return Err(backend::error("truncated","decompress","Incomplete compressed stream"));}
  if used==0&&self.input_start<self.input_len{return Err(backend::error("codec","compression","Codec made no progress"));}
  self.progress=if self.input_start==self.input_len&&!self.eof&&!self.flush_requested{Progress::Input}else{Progress::Advance};Ok(())
 }
}
struct SlotState{work:Option<Work>,pending:bool,pool_failure:Option<pool::AsyncTaskError<Infallible>>,alive:bool}
struct Slot{state:Mutex<SlotState>,ready:tokio::sync::Notify}
// Core cancellation drops successful T after a started job is cancelled. This
// owner restores its work on that path as well as queued callback disposal.
struct Recovery{work:Option<Work>,slot:Arc<Slot>}
impl Drop for Recovery{fn drop(&mut self){if let Some(work)=self.work.take(){let mut state=self.slot.state.lock().unwrap();if state.alive{assert!(state.work.is_none());state.work=Some(work);}}}}
struct Engine{slot:Arc<Slot>,capability:pool::PoolSubmission}
impl Engine{
 fn new(capability:pool::PoolSubmission,work:Work)->Self{Self{capability,slot:Arc::new(Slot{state:Mutex::new(SlotState{work:Some(work),pending:false,pool_failure:None,alive:true}),ready:tokio::sync::Notify::new()})}}
 async fn settled(&self)->Result<(),Failure>{loop{let mut notified=std::pin::pin!(self.slot.ready.notified());notified.as_mut().enable();{let mut state=self.slot.state.lock().unwrap();if !state.pending{if let Some(error)=state.pool_failure.take(){return Err(Failure::Pool(error));}if state.work.is_none(){return Err(Failure::Closed);}return Ok(());}}notified.await;}}
 async fn step(&self)->Result<(),Failure>{
  self.settled().await?;let work={let mut state=self.slot.state.lock().unwrap();state.pending=true;state.work.take().ok_or(Failure::Closed)?};let recovery=Recovery{work:Some(work),slot:self.slot.clone()};let slot=self.slot.clone();
  pool::task_async_publish(pool::repeat_submission(&self.capability),move||{let mut recovery=recovery;let work=recovery.work.as_mut().unwrap();if let Err(error)=work.step(){work.failure=Some(error);work.poisoned=true;}Ok::<_,Infallible>(recovery)},move|result|{let mut state=slot.state.lock().unwrap();match result{Ok(mut recovery)=>{let work=recovery.work.take();if state.alive{assert!(state.work.is_none());state.work=work;}},Err(error)=>{if matches!(&error,pool::AsyncTaskError::Completion(pool::JoinError::Panic)){if let Some(work)=state.work.as_mut(){work.poisoned=true;}}state.pool_failure=Some(error)}}state.pending=false;drop(state);slot.ready.notify_waiters();}).await;
  self.settled().await
 }
 fn status(&self)->Result<Progress,Failure>{let mut state=self.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;if let Some(error)=work.failure.take(){return Err(Failure::Codec(error));}if work.poisoned{return Err(Failure::Closed);}Ok(work.progress)}
}
impl Drop for Engine{fn drop(&mut self){let mut state=self.slot.state.lock().unwrap();state.alive=false;state.work.take();}}
pub struct TransformReader{source:io_backend::Reader,engine:Engine,input:Vec<u8>}
impl TransformReader{
 pub fn new(source:io_backend::Reader,capability:pool::PoolSubmission,format:Format,direction:Direction,level:Option<i64>,limit:Option<i64>)->Result<Self,backend::Error>{Ok(Self{source,engine:Engine::new(capability,Work::new(format,direction,level,limit)?),input:vec![0;WORKING]})}
 pub async fn read_into(&mut self,destination:&mut[u8])->Result<i64,Failure>{
  if destination.is_empty(){return Ok(0);}loop{self.engine.settled().await?;
   {let mut state=self.engine.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;if let Some(error)=work.failure.take(){return Err(Failure::Codec(error));}if work.poisoned{return Err(Failure::Closed);}if work.start<work.end{let count=destination.len().min(work.end-work.start);destination[..count].copy_from_slice(&work.output[work.start..work.start+count]);work.start+=count;return Ok(count as i64);}if work.progress==Progress::Finished{return Ok(0);}}
   if self.engine.status()?==Progress::Input{let count=self.source.read_into(&mut self.input).await.map_err(Failure::Io)? as usize;let mut state=self.engine.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;std::mem::swap(&mut work.input,&mut self.input);work.input_start=0;work.input_len=count;work.eof=count==0;work.progress=Progress::Advance;}
   self.engine.step().await?;
  }
 }
 pub async fn finish(mut self)->Result<io_backend::Reader,Failure>{let mut discard=[0;WORKING];while self.read_into(&mut discard).await?>0{}Ok(self.source)}
 pub async fn close(self)->Result<(),Failure>{self.finish().await?.close().await.map_err(Failure::Io)}
}
struct OutputLoan{slot:Arc<Slot>,bytes:Option<Vec<u8>>,position:usize,end:usize,armed:bool}
impl Drop for OutputLoan{fn drop(&mut self){let mut state=self.slot.state.lock().unwrap();if let Some(work)=state.work.as_mut(){work.output=self.bytes.take().unwrap();work.start=self.position;work.end=self.end;work.poisoned|=self.armed;}}}
pub struct TransformWriter{sink:io_backend::Writer,engine:Engine}
impl TransformWriter{
 pub fn new(sink:io_backend::Writer,capability:pool::PoolSubmission,format:Format,direction:Direction,level:Option<i64>,limit:Option<i64>)->Result<Self,backend::Error>{Ok(Self{sink,engine:Engine::new(capability,Work::new(format,direction,level,limit)?)})}
 async fn drain(&mut self)->Result<(),Failure>{
  self.engine.settled().await?;let output={let mut state=self.engine.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;if let Some(error)=work.failure.take(){return Err(Failure::Codec(error));}if work.poisoned{return Err(Failure::Closed);}if work.start==work.end{return Ok(());}OutputLoan{slot:self.engine.slot.clone(),bytes:Some(std::mem::take(&mut work.output)),position:work.start,end:work.end,armed:true}};
  let mut output=output;
  // One bounded output owner survives short writes. A cancelled sink operation
  // may have unreported B05 write effects, so poison rather than replay bytes.
  while output.position<output.end{
   let count=self.sink.write(&output.bytes.as_ref().unwrap()[output.position..output.end]).await.map_err(Failure::Io)? as usize;
   if count==0{return Err(Failure::Io(io_backend::Error::io("write",io::ErrorKind::WriteZero.into())));}output.position+=count;
  }
  output.armed=false;drop(output);Ok(())
 }

 async fn ready_for_input(&mut self)->Result<(),Failure>{loop{self.engine.settled().await?;self.drain().await?;match self.engine.status()?{Progress::Input=>return Ok(()),Progress::Finished=>return Err(Failure::Closed),_=>self.engine.step().await?}}}
 pub async fn write(&mut self,input:&[u8])->Result<i64,Failure>{
  if input.is_empty(){return Ok(0);}self.ready_for_input().await?;let count=input.len().min(WORKING);
  {let mut state=self.engine.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;work.input[..count].copy_from_slice(&input[..count]);work.input_start=0;work.input_len=count;work.eof=false;work.progress=Progress::Advance;}
  self.ready_for_input().await?;Ok(count as i64)
 }

 // Flush requests native Sync, drains bounded output, then flushes B05.
 // It does not signal EOF or manufacture a completed trailer.
 pub async fn flush(&mut self)->Result<(),Failure>{
  self.ready_for_input().await?;
  {let mut state=self.engine.slot.state.lock().unwrap();let work=state.work.as_mut().ok_or(Failure::Closed)?;work.flush_requested=true;}
  loop{let requested={let state=self.engine.slot.state.lock().unwrap();state.work.as_ref().ok_or(Failure::Closed)?.flush_requested};if !requested{break;}self.engine.step().await?;self.drain().await?;}
  self.sink.flush().await.map_err(Failure::Io)
 }
 pub async fn finish(mut self)->Result<io_backend::Writer,Failure>{
  self.engine.settled().await?;self.drain().await?;{let mut state=self.engine.slot.state.lock().unwrap();state.work.as_mut().ok_or(Failure::Closed)?.eof=true;}
  loop{self.engine.step().await?;self.drain().await?;if self.engine.status()?==Progress::Finished{break;}}
  self.sink.flush().await.map_err(Failure::Io)?;Ok(self.sink)
 }
 pub async fn close(self)->Result<(),Failure>{self.finish().await?.close().await.map_err(Failure::Io)}
}


fn io_failure(code:&'static str,operation:&'static str,message:&str)->Failure{Failure::Io(io_backend::Error{code,operation,message:message.into(),os_code:None})}
fn size(value:i64,operation:&'static str,positive:bool)->Result<usize,Failure>{usize::try_from(value).ok().filter(|value|!positive||*value>0).ok_or_else(||io_failure("range",operation,"Byte count must fit the target and be nonnegative; buffer size must be positive"))}
fn storage(length:usize,operation:&'static str)->Result<Vec<u8>,Failure>{let mut bytes=Vec::new();bytes.try_reserve_exact(length).map_err(|_|io_failure("allocation",operation,"Unable to reserve byte storage"))?;bytes.resize(length,0);Ok(bytes)}
impl TransformReader{
 pub async fn read(&mut self,maximum:i64)->Result<Vec<u8>,Failure>{let mut output=storage(size(maximum,"read",false)?,"read")?;let count=self.read_into(&mut output).await? as usize;output.truncate(count);Ok(output)}
 pub async fn read_exact(&mut self,destination:&mut[u8])->Result<(),Failure>{let mut offset=0;while offset<destination.len(){let count=self.read_into(&mut destination[offset..]).await? as usize;if count==0{return Err(Failure::Io(io_backend::Error::io("readExact",io::ErrorKind::UnexpectedEof.into())));}offset+=count;}Ok(())}
 pub async fn read_all(&mut self,maximum:Option<i64>)->Result<Vec<u8>,Failure>{let maximum=maximum.map(|value|size(value,"readAll",false)).transpose()?;let mut output=Vec::new();let mut bytes=[0;WORKING];loop{let capacity=maximum.map_or(WORKING,|bound|bound.saturating_sub(output.len()).saturating_add(1).min(WORKING));let count=self.read_into(&mut bytes[..capacity]).await? as usize;if count==0{return Ok(output);}if maximum.is_some_and(|bound|count>bound.saturating_sub(output.len())){return Err(io_failure("limit","readAll","Input exceeds the explicit byte limit; the overrun detection byte was consumed"));}output.try_reserve(count).map_err(|_|io_failure("allocation","readAll","Unable to grow byte storage"))?;output.extend_from_slice(&bytes[..count]);}}
}
impl TransformWriter{pub async fn write_all(&mut self,input:&[u8])->Result<(),Failure>{let mut offset=0;while offset<input.len(){let count=self.write(&input[offset..]).await? as usize;if count==0{return Err(Failure::Io(io_backend::Error::io("writeAll",io::ErrorKind::WriteZero.into())));}offset+=count;}Ok(())}}
// The finite B05/compression endpoint family keeps every transport/codec/pool error.
// There is no trait object, per-chunk Future box or worker waiting on async I/O.
// Endpoint implementations use only this family's own runtime types.
impl CopyRead for TransformReader{
 type Error=Failure;
 fn copy_read<'a>(&'a mut self,bytes:&'a mut[u8])->impl std::future::Future<Output=Result<i64,Failure>>+Send+'a{TransformReader::read_into(self,bytes)}
}
impl CopyWrite for TransformWriter{
 type Error=Failure;
 fn copy_write_all<'a>(&'a mut self,bytes:&'a[u8])->impl std::future::Future<Output=Result<(),Failure>>+Send+'a{TransformWriter::write_all(self,bytes)}
}
pub enum ReaderRef<'a>{Io(&'a mut io_backend::Reader),Compression(&'a mut TransformReader)}
pub enum WriterRef<'a>{Io(&'a mut io_backend::Writer),Compression(&'a mut TransformWriter)}
impl ReaderRef<'_>{async fn read_into(&mut self,bytes:&mut[u8])->Result<i64,Failure>{match self{Self::Io(reader)=>reader.read_into(bytes).await.map_err(Failure::Io),Self::Compression(reader)=>reader.read_into(bytes).await}}}
impl WriterRef<'_>{async fn write_all(&mut self,bytes:&[u8])->Result<(),Failure>{match self{Self::Io(writer)=>writer.write_all(bytes).await.map_err(Failure::Io),Self::Compression(writer)=>writer.write_all(bytes).await}}}
impl CopyRead for ReaderRef<'_>{
 type Error=Failure;
 fn copy_read<'a>(&'a mut self,bytes:&'a mut[u8])->impl std::future::Future<Output=Result<i64,Failure>>+Send+'a{ReaderRef::read_into(self,bytes)}
}
impl CopyWrite for WriterRef<'_>{
 type Error=Failure;
 fn copy_write_all<'a>(&'a mut self,bytes:&'a[u8])->impl std::future::Future<Output=Result<(),Failure>>+Send+'a{WriterRef::write_all(self,bytes)}
}
pub async fn copy(mut reader:ReaderRef<'_>,mut writer:WriterRef<'_>,buffer_size:i64,maximum:Option<i64>)->Result<i64,Failure>{io_backend::copy_as(&mut reader,&mut writer,buffer_size,maximum,std::convert::identity,std::convert::identity,Failure::Io).await}

#[cfg(test)]pub(crate) struct Gate{pub started:tokio::sync::oneshot::Sender<()>,pub release:std::sync::mpsc::Receiver<()>}
#[cfg(test)]impl Drop for Work{fn drop(&mut self){if let Some(drops)=&self.drops{drops.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}}
#[cfg(test)]pub(crate) fn gate_reader(reader:&mut TransformReader,gate:Gate,drops:Arc<std::sync::atomic::AtomicUsize>){let mut state=reader.engine.slot.state.lock().unwrap();let work=state.work.as_mut().unwrap();work.gate=Some(gate);work.drops=Some(drops);}
#[cfg(test)]pub(crate) fn reader_workspace(reader:&TransformReader)->(usize,usize){let state=reader.engine.slot.state.lock().unwrap();let work=state.work.as_ref().unwrap();(work.input.capacity(),work.output.capacity())}

#[cfg(test)]pub(crate) async fn initialize_reader(reader:&mut TransformReader)->Result<(),Failure>{reader.engine.step().await}

#[cfg(test)]pub(crate) fn watch_writer(writer:&mut TransformWriter,drops:Arc<std::sync::atomic::AtomicUsize>){let mut state=writer.engine.slot.state.lock().unwrap();state.work.as_mut().unwrap().drops=Some(drops);}
#[cfg(test)]pub(crate) fn watch_reader(reader:&mut TransformReader,drops:Arc<std::sync::atomic::AtomicUsize>){let mut state=reader.engine.slot.state.lock().unwrap();state.work.as_mut().unwrap().drops=Some(drops);}

}
