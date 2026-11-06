mod tsrust_native_io {
 use std::{future::Future,io::{self,Read,Write},pin::Pin,sync::{Arc,Mutex},task::{Context,Poll}};
 use tokio::io::{AsyncRead,AsyncReadExt,AsyncWrite,AsyncWriteExt};
 // Tokio's copy working-buffer default, not a total stream or allocation bound.
 pub const BUFFER_SIZE:usize=8192;
 #[derive(Debug)]pub struct Error{pub code:&'static str,pub operation:&'static str,pub message:std::string::String,pub os_code:Option<i64>}
 impl std::fmt::Display for Error{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.message)}}
 impl std::error::Error for Error{}
 fn error(code:&'static str,operation:&'static str,message:&str)->Error{Error{code,operation,message:message.into(),os_code:None}}
 impl Error{
  pub fn io(operation:&'static str,failure:io::Error)->Self{
   let os_code=failure.raw_os_error().map(i64::from);let code=match failure.kind(){io::ErrorKind::UnexpectedEof=>"unexpected-eof",io::ErrorKind::WriteZero=>"write-zero",io::ErrorKind::Interrupted=>"interrupted",io::ErrorKind::PermissionDenied=>"permission-denied",io::ErrorKind::NotFound=>"not-found",io::ErrorKind::InvalidInput=>"invalid-input",io::ErrorKind::BrokenPipe=>"broken-pipe",_=>"io"};
   Self{code,operation,message:failure.to_string(),os_code}
  }
  fn fs(failure:crate::tsrust_native_files::fs::FsFailure)->Self{Self{code:failure.code,operation:failure.operation,message:failure.message,os_code:failure.os_code}}
 }
 fn size(value:i64,operation:&'static str,positive:bool)->Result<usize,Error>{usize::try_from(value).ok().filter(|size|!positive||*size>0).ok_or_else(||error("range",operation,"Byte counts must fit the target and be nonnegative; working buffers must be positive."))}
 fn allocate(length:usize,operation:&'static str)->Result<Vec<u8>,Error>{let mut output=Vec::new();output.try_reserve_exact(length).map_err(|_|error("allocation",operation,"Unable to allocate byte storage."))?;output.resize(length,0);Ok(output)}
 enum Backend{File(crate::tsrust_native_files::fs::FileHandle),Reader(crate::tsrust_native_files::buffered_fs::BufferedReader),Writer(crate::tsrust_native_files::buffered_fs::BufferedWriter),Stdout(io::Stdout),Stderr(io::Stderr),#[cfg(test)] Controlled(Box<dyn Read+Send>)}
 impl Backend{
  fn read(&mut self,bytes:&mut[u8])->Result<usize,Error>{loop{let result=match self{Self::File(file)=>Read::read(file,bytes),Self::Reader(reader)=>Read::read(reader,bytes),#[cfg(test)] Self::Controlled(reader)=>reader.read(bytes),_=>return Err(error("direction","read","This resource is not readable."))};match result{Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>return result.map_err(|failure|Error::io("read",failure))}}}
  fn write(&mut self,bytes:&[u8])->Result<usize,Error>{loop{let result=match self{Self::File(file)=>Write::write(file,bytes),Self::Writer(writer)=>return crate::tsrust_native_files::buffered_fs::buffered_write(writer,bytes).map(|count|count as usize).map_err(Error::fs),Self::Stdout(output)=>output.write(bytes),Self::Stderr(output)=>output.write(bytes),_=>return Err(error("direction","write","This resource is not writable."))};match result{Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>return result.map_err(|failure|Error::io("write",failure))}}}
  fn flush(&mut self)->Result<(),Error>{match self{Self::File(file)=>file.flush().map_err(|failure|Error::io("flush",failure)),Self::Writer(writer)=>crate::tsrust_native_files::buffered_fs::flush(writer).map_err(Error::fs),Self::Stdout(output)=>output.flush().map_err(|failure|Error::io("flush",failure)),Self::Stderr(output)=>output.flush().map_err(|failure|Error::io("flush",failure)),_=>Ok(())}}
  fn close(self)->Result<(),Error>{match self{Self::File(file)=>file.close().map_err(Error::fs),Self::Reader(reader)=>reader.close().map_err(Error::fs),Self::Writer(writer)=>writer.close().map_err(Error::fs),mut output=>output.flush()}}
 }
 #[derive(Clone,Copy)]enum Operation{Read(usize),Write(usize),Flush,Close}
 struct Completed{backend:Option<Backend>,buffer:Vec<u8>,operation:Operation,result:Result<usize,Error>}
 struct State{backend:Option<Backend>,buffer:Vec<u8>,start:usize,end:usize,pending:Option<tokio::task::JoinHandle<Completed>>,failure:Option<Error>,count:usize,owner_alive:bool,cleanup_registered:bool,operation_waker:Option<std::task::Waker>,cleanup_waker:Option<std::task::Waker>}
 impl Drop for State{fn drop(&mut self){if let Some(backend)=self.backend.take(){if tokio::runtime::Handle::try_current().is_ok(){crate::tsrust_native_futures::cleanup(async move{let _=tokio::task::spawn_blocking(move||backend.close()).await;});}else{let _=backend.close();}}}}
 struct Blocking{state:Arc<Mutex<State>>}
 impl Blocking{
  fn new(backend:Backend)->Self{Self{state:Arc::new(Mutex::new(State{backend:Some(backend),buffer:vec![0;BUFFER_SIZE],start:0,end:0,pending:None,failure:None,count:0,owner_alive:true,cleanup_registered:false,operation_waker:None,cleanup_waker:None}))}}
  fn settle(state:&Arc<Mutex<State>>,cx:&mut Context<'_>,cleanup:bool)->Poll<()>{
   let mut state=state.lock().unwrap();if state.pending.is_none(){return Poll::Ready(());}
   let waiter=if cleanup{&mut state.cleanup_waker}else{&mut state.operation_waker};if waiter.as_ref().is_none_or(|waiter|!waiter.will_wake(cx.waker())){*waiter=Some(cx.waker().clone());}
   let pending=state.pending.as_mut().unwrap();let completed=match Pin::new(pending).poll(cx){Poll::Pending=>return Poll::Pending,Poll::Ready(completed)=>completed};state.pending=None;
   match completed{Ok(completed)=>{state.backend=completed.backend;state.buffer=completed.buffer;state.count=0;match completed.result{Ok(count)=>{state.count=count;if matches!(completed.operation,Operation::Read(_)){state.start=0;state.end=count;}},Err(failure)=>state.failure=Some(failure)}},Err(failure)=>state.failure=Some(error("worker","io",&failure.to_string()))};let waiters=[state.operation_waker.take(),state.cleanup_waker.take()];drop(state);for waiter in waiters.into_iter().flatten(){waiter.wake();}Poll::Ready(())
  }
  fn cleanup(state:&Arc<Mutex<State>>){
   let register={let mut state=state.lock().unwrap();if state.cleanup_registered||state.pending.is_none()&&state.backend.is_none(){false}else{state.cleanup_registered=true;true}};
   if register{crate::tsrust_native_futures::cleanup(Self::settle_owned(state.clone()));}
  }
  async fn settle_owned(state:Arc<Mutex<State>>){
   loop{
    std::future::poll_fn(|cx|Self::settle(&state,cx,true)).await;
    // The unique Blocking owner marks its drop explicitly; cleanup Arcs are not owners.
    // A resumed operation can start before this cleanup wakes. Retain registration until that operation also settles.
    let backend={let mut state=state.lock().unwrap();if state.pending.is_some(){continue;}state.cleanup_registered=false;if !state.owner_alive{state.backend.take()}else{None}};
    if let Some(backend)=backend{let _=tokio::task::spawn_blocking(move||backend.close()).await;}return;
   }
  }
  async fn operation(&mut self,operation:Operation,input:&[u8])->Result<usize,Error>{
   let mut deferred=None;let mut loan=Pending{state:self.state.clone(),armed:true};std::future::poll_fn(|cx|Self::settle(&self.state,cx,false)).await;
   {let mut state=self.state.lock().unwrap();if let Some(failure)=state.failure.take(){if matches!(operation,Operation::Close){deferred=Some(failure);}else{loan.armed=false;return Err(failure);}}if matches!(operation,Operation::Read(_))&&state.start<state.end{loan.armed=false;return Ok(state.end-state.start);}
    let mut backend=match state.backend.take(){Some(backend)=>backend,None=>{loan.armed=false;return Err(deferred.unwrap_or_else(||error("closed","io","The resource has been closed.")));}};let mut buffer=std::mem::take(&mut state.buffer);
    let operation=match operation{Operation::Read(length)=>Operation::Read(length.min(buffer.len())),Operation::Write(length)=>{let length=length.min(buffer.len());buffer[..length].copy_from_slice(&input[..length]);Operation::Write(length)},other=>other};
    state.pending=Some(tokio::task::spawn_blocking(move||{
     if matches!(operation,Operation::Close){return Completed{backend:None,buffer,operation,result:backend.close().map(|_|0)};}
     let result=match operation{Operation::Read(length)=>backend.read(&mut buffer[..length]),Operation::Write(length)=>backend.write(&buffer[..length]),Operation::Flush=>backend.flush().map(|_|0),Operation::Close=>unreachable!()};Completed{backend:Some(backend),buffer,operation,result}
    }));
   }
   std::future::poll_fn(|cx|Self::settle(&self.state,cx,false)).await;loan.armed=false;let mut state=self.state.lock().unwrap();if let Some(failure)=state.failure.take(){Err(failure)}else if let Some(failure)=deferred{Err(failure)}else{Ok(state.count)}
  }
  fn poll_read_into(&mut self,cx:&mut Context<'_>,destination:&mut[u8])->Poll<Result<usize,Error>>{
   if destination.is_empty(){return Poll::Ready(Ok(0));}
   loop{
    let was_pending=self.state.lock().unwrap().pending.is_some();
    if Self::settle(&self.state,cx,false).is_pending(){return Poll::Pending;}
    let mut state=self.state.lock().unwrap();
    if let Some(failure)=state.failure.take(){return Poll::Ready(Err(failure));}
    if was_pending&&state.count==0&&state.start==state.end{return Poll::Ready(Ok(0));}
    if state.start<state.end{let count=destination.len().min(state.end-state.start);destination[..count].copy_from_slice(&state.buffer[state.start..state.start+count]);state.start+=count;return Poll::Ready(Ok(count));}
    let Some(mut backend)=state.backend.take()else{return Poll::Ready(Err(error("closed","read","The resource has been closed.")));};
    let mut buffer=std::mem::take(&mut state.buffer);let length=destination.len().min(buffer.len());
    state.pending=Some(tokio::task::spawn_blocking(move||{let result=backend.read(&mut buffer[..length]);Completed{backend:Some(backend),buffer,operation:Operation::Read(length),result}}));drop(state);
    if Self::settle(&self.state,cx,false).is_pending(){return Poll::Pending;}
    let state=self.state.lock().unwrap();if state.failure.is_none()&&state.end==0{return Poll::Ready(Ok(0));}
   }
  }
  async fn read_into(&mut self,destination:&mut[u8])->Result<usize,Error>{if destination.is_empty(){return Ok(0);}self.operation(Operation::Read(destination.len()),&[]).await?;let mut state=self.state.lock().unwrap();let count=destination.len().min(state.end-state.start);destination[..count].copy_from_slice(&state.buffer[state.start..state.start+count]);state.start+=count;Ok(count)}
 }
 impl Drop for Blocking{fn drop(&mut self){let needs_cleanup={let mut state=self.state.lock().unwrap();state.owner_alive=false;state.pending.is_some()||state.backend.is_some()};if needs_cleanup&&tokio::runtime::Handle::try_current().is_ok(){Blocking::cleanup(&self.state);}}}

 struct Pending{state:Arc<Mutex<State>>,armed:bool}
 impl Drop for Pending{fn drop(&mut self){if self.armed{Blocking::cleanup(&self.state);}}}
 pub struct Reader{inner:ReadBackend}
 enum ReadBackend{Blocking(Blocking),Memory(io::Cursor<Vec<u8>>),Async(Box<dyn AsyncRead+Send+Unpin>)}
 impl Reader{
  pub fn file(file:crate::tsrust_native_files::fs::FileHandle)->Self{Self{inner:ReadBackend::Blocking(Blocking::new(Backend::File(file)))}}
  pub fn buffered(reader:crate::tsrust_native_files::buffered_fs::BufferedReader)->Self{Self{inner:ReadBackend::Blocking(Blocking::new(Backend::Reader(reader)))}}
  pub fn bytes(bytes:Vec<u8>)->Self{Self{inner:ReadBackend::Memory(io::Cursor::new(bytes))}}
  pub fn asynchronous(reader:Box<dyn AsyncRead+Send+Unpin>)->Self{Self{inner:ReadBackend::Async(reader)}}
  pub async fn read_into(&mut self,destination:&mut[u8])->Result<i64,Error>{if destination.is_empty(){return Ok(0);}let count=match &mut self.inner{ReadBackend::Blocking(reader)=>reader.read_into(destination).await?,ReadBackend::Memory(reader)=>Read::read(reader,destination).map_err(|failure|Error::io("read",failure))?,ReadBackend::Async(reader)=>loop{match reader.read(destination).await{Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|failure|Error::io("read",failure))?}}};i64::try_from(count).map_err(|_|error("range","readInto","Read length exceeds int."))}
  pub async fn read(&mut self,length:i64)->Result<Vec<u8>,Error>{let mut output=allocate(size(length,"read",false)?,"read")?;let count=self.read_into(&mut output).await?;output.truncate(count as usize);Ok(output)}
  pub async fn read_exact(&mut self,destination:&mut[u8])->Result<(),Error>{let mut filled=0;while filled<destination.len(){let count=self.read_into(&mut destination[filled..]).await? as usize;if count==0{return Err(error("unexpected-eof","readExact","EOF before the destination was filled; completed prefix remains written."));}filled+=count;}Ok(())}
  pub async fn read_all(&mut self,limit:Option<i64>)->Result<Vec<u8>,Error>{let limit=limit.map(|limit|size(limit,"readAll",false)).transpose()?;let mut output=Vec::new();let mut buffer=[0u8;BUFFER_SIZE];loop{let remaining=limit.map_or(buffer.len(),|limit|limit.saturating_sub(output.len()).saturating_add(1).min(buffer.len()));let count=self.read_into(&mut buffer[..remaining]).await? as usize;if count==0{return Ok(output);}if limit.is_some_and(|limit|count>limit-output.len()){return Err(error("limit","readAll","Input exceeds the explicit byte limit; overrun detection consumes at most one extra byte."));}output.try_reserve(count).map_err(|_|error("allocation","readAll","Unable to grow output."))?;output.extend_from_slice(&buffer[..count]);}}
  pub async fn close(mut self)->Result<(),Error>{match &mut self.inner{ReadBackend::Blocking(reader)=>reader.operation(Operation::Close,&[]).await.map(|_|()),_=>Ok(())}}
 }
 impl AsyncRead for Reader{
  fn poll_read(mut self:Pin<&mut Self>,cx:&mut Context<'_>,destination:&mut tokio::io::ReadBuf<'_>)->Poll<io::Result<()>>{
   if destination.remaining()==0{return Poll::Ready(Ok(()));}
   match &mut self.inner{
    ReadBackend::Blocking(reader)=>match reader.poll_read_into(cx,destination.initialize_unfilled()){
     Poll::Pending=>Poll::Pending,Poll::Ready(Err(error))=>Poll::Ready(Err(io::Error::other(error))),Poll::Ready(Ok(count))=>{destination.advance(count);Poll::Ready(Ok(()))}},
    ReadBackend::Memory(reader)=>{match Read::read(reader,destination.initialize_unfilled()){Ok(count)=>{destination.advance(count);Poll::Ready(Ok(()))},Err(error)=>Poll::Ready(Err(error))}},
    ReadBackend::Async(reader)=>Pin::new(reader).poll_read(cx,destination),
   }
  }
 }
 pub struct Writer{inner:WriteBackend}
 enum WriteBackend{Blocking(Blocking),Async(Box<dyn AsyncWrite+Send+Unpin>)}
 impl Writer{
  pub fn file(file:crate::tsrust_native_files::fs::FileHandle)->Self{Self{inner:WriteBackend::Blocking(Blocking::new(Backend::File(file)))}}
  pub fn buffered(writer:crate::tsrust_native_files::buffered_fs::BufferedWriter)->Self{Self{inner:WriteBackend::Blocking(Blocking::new(Backend::Writer(writer)))}}
  pub fn stdout()->Self{Self{inner:WriteBackend::Blocking(Blocking::new(Backend::Stdout(io::stdout())))}}
  pub fn stderr()->Self{Self{inner:WriteBackend::Blocking(Blocking::new(Backend::Stderr(io::stderr())))}}
  pub fn asynchronous(writer:Box<dyn AsyncWrite+Send+Unpin>)->Self{Self{inner:WriteBackend::Async(writer)}}
  pub async fn write(&mut self,bytes:&[u8])->Result<i64,Error>{if bytes.is_empty(){return Ok(0);}let count=match &mut self.inner{WriteBackend::Blocking(writer)=>writer.operation(Operation::Write(bytes.len()),bytes).await?,WriteBackend::Async(writer)=>loop{match writer.write(bytes).await{Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|failure|Error::io("write",failure))?}}};i64::try_from(count).map_err(|_|error("range","write","Write length exceeds int."))}
  pub async fn write_all(&mut self,bytes:&[u8])->Result<(),Error>{let mut written=0;while written<bytes.len(){let count=self.write(&bytes[written..]).await? as usize;if count==0{return Err(error("write-zero","writeAll","Writer accepted zero bytes before completion."));}written+=count;}Ok(())}
  pub async fn flush(&mut self)->Result<(),Error>{match &mut self.inner{WriteBackend::Blocking(writer)=>writer.operation(Operation::Flush,&[]).await.map(|_|()),WriteBackend::Async(writer)=>writer.flush().await.map_err(|failure|Error::io("flush",failure))}}
  pub async fn close(mut self)->Result<(),Error>{match &mut self.inner{WriteBackend::Blocking(writer)=>writer.operation(Operation::Close,&[]).await.map(|_|()),WriteBackend::Async(writer)=>writer.shutdown().await.map_err(|failure|Error::io("close",failure))}}
 }
 pub async fn copy(reader:&mut Reader,writer:&mut Writer,buffer_size:i64,limit:Option<i64>)->Result<i64,Error>{let mut buffer=allocate(size(buffer_size,"copy",true)?,"copy")?;let limit=limit.map(|limit|size(limit,"copy",false)).transpose()?;let mut total=0usize;loop{let remaining=limit.map_or(buffer.len(),|limit|limit.saturating_sub(total).saturating_add(1).min(buffer.len()));let count=reader.read_into(&mut buffer[..remaining]).await? as usize;if count==0{return i64::try_from(total).map_err(|_|error("range","copy","Copied length exceeds int."));}if let Some(limit)=limit && count>limit-total{writer.write_all(&buffer[..limit-total]).await?;return Err(error("limit","copy","Input exceeds the explicit byte limit; the allowed prefix was written and overrun detection consumed one extra byte."));}writer.write_all(&buffer[..count]).await?;total=total.checked_add(count).ok_or_else(||error("range","copy","Copied length exceeds target range."))?;}}
 pub struct LineReader{reader:Reader,chunk:Vec<u8>,start:usize,end:usize,line:Vec<u8>,limit:Option<usize>}
 impl LineReader{
  pub fn new(reader:Reader,limit:Option<i64>)->Result<Self,Error>{Ok(Self{reader,chunk:allocate(BUFFER_SIZE,"lines")?,start:0,end:0,line:Vec::new(),limit:limit.map(|limit|size(limit,"lines",false)).transpose()?})}
  pub async fn read_line(&mut self)->Result<Option<std::string::String>,Error>{loop{
   if self.start==self.end{self.start=0;self.end=self.reader.read_into(&mut self.chunk).await? as usize;if self.end==0{if self.line.is_empty(){return Ok(None);}return self.finish().map(Some);}}
   let remaining=&self.chunk[self.start..self.end];
   let newline=remaining.iter().position(|value|*value==b'\n');
   let stop=self.start+newline.map_or(remaining.len(),|index|index+1);
   while self.start<stop{
    let permitted=self.limit.map_or(stop-self.start,|limit|limit.saturating_sub(self.line.len()).min(stop-self.start));
    if permitted==0{self.start+=1;self.line.clear();return Err(error("limit","readLine","Line exceeds the explicit raw-byte limit, including terminators; consumed prefix is not rolled back."));}
    // Grow at the same full-capacity boundary as the bytewise implementation.
    // A failed reservation consumes its detection byte and retains the prior prefix.
    if self.line.len()==self.line.capacity(){if self.line.try_reserve(1).is_err(){self.start+=1;return Err(error("allocation","readLine","Unable to grow line buffer."));}}
    let count=permitted.min(self.line.capacity()-self.line.len());
    self.line.extend_from_slice(&self.chunk[self.start..self.start+count]);self.start+=count;
   }
   if newline.is_some(){return self.finish().map(Some);}
  }}
  fn finish(&mut self)->Result<std::string::String,Error>{let mut line=std::mem::take(&mut self.line);if line.last()==Some(&b'\n'){line.pop();if line.last()==Some(&b'\r'){line.pop();}}std::string::String::from_utf8(line).map_err(|_|error("encoding","readLine","Line is not valid UTF-8."))}
  pub async fn close(self)->Result<(),Error>{self.reader.close().await}
 }
 pub struct Stdin{inner:io::Stdin}
 impl Stdin{
  pub fn new()->Self{Self{inner:io::stdin()}}
  pub fn read(&mut self,length:i64)->Result<Vec<u8>,Error>{let mut output=allocate(size(length,"readStdin",false)?,"readStdin")?;let count=loop{match self.inner.read(&mut output){Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|failure|Error::io("readStdin",failure))?}};output.truncate(count);Ok(output)}
  pub fn read_line(&mut self,limit:Option<i64>)->Result<Option<std::string::String>,Error>{let limit=limit.map(|limit|size(limit,"readLineStdin",false)).transpose()?;let mut output=Vec::new();let mut input=self.inner.lock();loop{let mut byte=[0];let count=loop{match input.read(&mut byte){Err(failure)if failure.kind()==io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|failure|Error::io("readLineStdin",failure))?}};if count==0{if output.is_empty(){return Ok(None);}break;}if limit.is_some_and(|limit|output.len()>=limit){return Err(error("limit","readLineStdin","Line exceeds explicit raw-byte limit, including terminators."));}output.try_reserve(1).map_err(|_|error("allocation","readLineStdin","Unable to grow line buffer."))?;output.push(byte[0]);if byte[0]==b'\n'{break;}}
   if output.last()==Some(&b'\n'){output.pop();if output.last()==Some(&b'\r'){output.pop();}}std::string::String::from_utf8(output).map(Some).map_err(|_|error("encoding","readLineStdin","Line is not valid UTF-8."))
  }
 }
 #[cfg(test)]mod tests{use super::*;include!("runtime_tests.rs");}

}
