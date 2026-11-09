// HTTP integration remains separate so file-only I/O does not require HTTP dependencies.
mod tsrust_native_http_io {
 use std::{pin::Pin,task::{Context,Poll}};
 use tokio::io::{AsyncRead,ReadBuf};
 use axum::body::{Body,Bytes,HttpBody};
 use super::{tsrust_native_http,tsrust_native_io};
 pub struct FramedReader {body:tsrust_native_http::Stream,frame:Bytes,offset:usize}
 impl AsyncRead for FramedReader {
  fn poll_read(mut self:Pin<&mut Self>,cx:&mut Context<'_>,destination:&mut ReadBuf<'_>)->Poll<std::io::Result<()>>{
   if destination.remaining()==0{return Poll::Ready(Ok(()));}
   loop{
    if self.offset<self.frame.len(){let count=destination.remaining().min(self.frame.len()-self.offset);destination.put_slice(&self.frame[self.offset..self.offset+count]);self.offset+=count;return Poll::Ready(Ok(()));}
    match Pin::new(&mut self.body).poll_frame(cx){
     Poll::Pending=>return Poll::Pending,Poll::Ready(None)=>return Poll::Ready(Ok(())),
     Poll::Ready(Some(Err(error)))=>return Poll::Ready(Err(std::io::Error::other(error))),
     Poll::Ready(Some(Ok(frame)))=>{if let Ok(frame)=frame.into_data(){self.frame=frame;self.offset=0;}}
    }
   }
  }
 }
 pub fn reader(body:tsrust_native_http::Stream)->tsrust_native_io::Reader{tsrust_native_io::Reader::asynchronous(Box::new(FramedReader{body,frame:Bytes::new(),offset:0}))}
 struct ReaderBody{reader:Option<tsrust_native_io::Reader>,chunk:Vec<u8>}
 impl HttpBody for ReaderBody{
  type Data=Bytes;type Error=std::io::Error;
  fn poll_frame(self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<Option<Result<http_body::Frame<Bytes>,Self::Error>>>{
   let this=self.get_mut();let Some(reader)=&mut this.reader else{return Poll::Ready(None);};
   if this.chunk.is_empty(){this.chunk.resize(tsrust_native_io::BUFFER_SIZE,0);}
   let mut destination=ReadBuf::new(&mut this.chunk);
   match Pin::new(reader).poll_read(cx,&mut destination){
    Poll::Pending=>Poll::Pending,Poll::Ready(Err(error))=>{this.reader=None;Poll::Ready(Some(Err(error)))},
    Poll::Ready(Ok(()))=>{let count=destination.filled().len();if count==0{this.reader=None;return Poll::Ready(None);}this.chunk.truncate(count);Poll::Ready(Some(Ok(http_body::Frame::data(std::mem::take(&mut this.chunk).into()))))}
   }
  }
 }
 pub fn body(reader:tsrust_native_io::Reader)->tsrust_native_http::Stream{tsrust_native_http::Stream::unbounded(Body::new(ReaderBody{reader:Some(reader),chunk:Vec::new()}))}
}
