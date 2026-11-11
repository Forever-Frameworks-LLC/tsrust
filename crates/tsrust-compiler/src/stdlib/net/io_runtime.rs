// Full-stream operations preserve B05 byte/error/bound contracts without splitting.
// Copy's finite stack enums borrow existing endpoints; no trait object or future box.
mod tsrust_native_net_io {
 use crate::tsrust_native_io::{Error,Reader,Writer,BUFFER_SIZE,allocate,size};
 use crate::tsrust_native_io::{CopyRead,CopyWrite};
 use crate::tsrust_native_net_transport::Stream;
 use tokio::io::{AsyncReadExt,AsyncWriteExt};
 fn error(code:&'static str,operation:&'static str,message:&str)->Error{Error{code,operation,message:message.into(),os_code:None}}
 pub async fn read_into(stream:&mut tokio::net::TcpStream,bytes:&mut[u8])->Result<i64,Error>{if bytes.is_empty(){return Ok(0);}let count=loop{match stream.read(bytes).await{Err(value)if value.kind()==std::io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|value|Error::io("read",value))?}};i64::try_from(count).map_err(|_|error("range","readInto","Read length exceeds int."))}
 pub async fn read(stream:&mut tokio::net::TcpStream,length:i64)->Result<Vec<u8>,Error>{let mut bytes=allocate(size(length,"read",false)?,"read")?;let count=read_into(stream,&mut bytes).await?;bytes.truncate(count as usize);Ok(bytes)}
 pub async fn read_exact(stream:&mut tokio::net::TcpStream,bytes:&mut[u8])->Result<(),Error>{let mut filled=0;while filled<bytes.len(){let count=read_into(stream,&mut bytes[filled..]).await? as usize;if count==0{return Err(error("unexpected-eof","readExact","EOF before the destination was filled; completed prefix remains written."));}filled+=count;}Ok(())}
 pub async fn read_all(stream:&mut tokio::net::TcpStream,limit:Option<i64>)->Result<Vec<u8>,Error>{let limit=limit.map(|value|size(value,"readAll",false)).transpose()?;let mut output=Vec::new();let mut buffer=[0u8;BUFFER_SIZE];loop{let remaining=limit.map_or(buffer.len(),|limit|limit.saturating_sub(output.len()).saturating_add(1).min(buffer.len()));let count=read_into(stream,&mut buffer[..remaining]).await? as usize;if count==0{return Ok(output);}if limit.is_some_and(|limit|count>limit-output.len()){return Err(error("limit","readAll","Input exceeds the explicit byte limit; overrun detection consumes at most one extra byte."));}output.try_reserve(count).map_err(|_|error("allocation","readAll","Unable to grow output."))?;output.extend_from_slice(&buffer[..count]);}}
 pub async fn write(stream:&mut tokio::net::TcpStream,bytes:&[u8])->Result<i64,Error>{if bytes.is_empty(){return Ok(0);}let count=loop{match stream.write(bytes).await{Err(value)if value.kind()==std::io::ErrorKind::Interrupted=>continue,result=>break result.map_err(|value|Error::io("write",value))?}};i64::try_from(count).map_err(|_|error("range","write","Write length exceeds int."))}
 pub async fn write_all(stream:&mut tokio::net::TcpStream,bytes:&[u8])->Result<(),Error>{let mut written=0;while written<bytes.len(){let count=write(stream,&bytes[written..]).await? as usize;if count==0{return Err(error("write-zero","writeAll","Writer accepted zero bytes before completion."));}written+=count;}Ok(())}
// Endpoint implementations use only this family's own runtime types.
impl CopyRead for Stream{
 type Error=Error;
 fn copy_read<'a>(&'a mut self,bytes:&'a mut[u8])->impl std::future::Future<Output=Result<i64,Error>>+Send+'a{Stream::read_into(self,bytes)}
}
impl CopyWrite for Stream{
 type Error=Error;
 fn copy_write_all<'a>(&'a mut self,bytes:&'a[u8])->impl std::future::Future<Output=Result<(),Error>>+Send+'a{Stream::write_all(self,bytes)}
}
 pub enum CopyReader<'a>{Io(&'a mut Reader),Tcp(&'a mut Stream)}
 pub enum CopyWriter<'a>{Io(&'a mut Writer),Tcp(&'a mut Stream)}
 impl CopyReader<'_>{async fn read_into(&mut self,bytes:&mut[u8])->Result<i64,Error>{match self{Self::Io(reader)=>reader.read_into(bytes).await,Self::Tcp(reader)=>reader.read_into(bytes).await}}}
 impl CopyWriter<'_>{async fn write_all(&mut self,bytes:&[u8])->Result<(),Error>{match self{Self::Io(writer)=>writer.write_all(bytes).await,Self::Tcp(writer)=>writer.write_all(bytes).await}}}
impl CopyRead for CopyReader<'_>{
 type Error=Error;
 fn copy_read<'a>(&'a mut self,bytes:&'a mut[u8])->impl std::future::Future<Output=Result<i64,Error>>+Send+'a{CopyReader::read_into(self,bytes)}
}
impl CopyWrite for CopyWriter<'_>{
 type Error=Error;
 fn copy_write_all<'a>(&'a mut self,bytes:&'a[u8])->impl std::future::Future<Output=Result<(),Error>>+Send+'a{CopyWriter::write_all(self,bytes)}
}
 pub async fn copy(mut reader:CopyReader<'_>,mut writer:CopyWriter<'_>,buffer_size:i64,limit:Option<i64>)->Result<i64,Error>{crate::tsrust_native_io::copy_as(&mut reader,&mut writer,buffer_size,limit,std::convert::identity,std::convert::identity,std::convert::identity).await}
}
