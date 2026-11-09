mod tsrust_native_net_transport {
 use crate::tsrust_native_net::{SocketAddress,Error,error,io_error};
 use tokio::io::AsyncWriteExt;
 pub struct Listener(tokio::net::TcpListener);
 pub struct Stream{pub(crate) inner:tokio::net::TcpStream}
 pub struct Udp(tokio::net::UdpSocket);
 pub struct Datagram{pub data:Vec<u8>,pub remote:SocketAddress}
 fn runtime(operation:&'static str)->Result<(),Error>{tokio::runtime::Handle::try_current().map(|_|()).map_err(|_|error("runtime",operation,"An active runtime is required"))}
 pub async fn listen(address:&SocketAddress)->Result<Listener,Error>{runtime("listenTcp")?;tokio::net::TcpListener::bind(*address).await.map(Listener).map_err(|value|io_error("listenTcp",value))}
 pub async fn connect(address:&SocketAddress)->Result<Stream,Error>{runtime("connectTcp")?;tokio::net::TcpStream::connect(*address).await.map(|inner|Stream{inner}).map_err(|value|io_error("connectTcp",value))}
 impl Listener{
  pub async fn accept(&self)->Result<(Stream,SocketAddress),Error>{self.0.accept().await.map(|(inner,address)|(Stream{inner},address)).map_err(|value|io_error("accept",value))}
  pub fn local_address(&self)->Result<SocketAddress,Error>{self.0.local_addr().map_err(|value|io_error("localAddress",value))}
  pub fn close(self){drop(self)}
 }
 impl Stream{
  pub fn local_address(&self)->Result<SocketAddress,Error>{self.inner.local_addr().map_err(|value|io_error("localAddress",value))}
  pub fn peer_address(&self)->Result<SocketAddress,Error>{self.inner.peer_addr().map_err(|value|io_error("peerAddress",value))}
  pub fn set_no_delay(&mut self,enabled:bool)->Result<(),Error>{self.inner.set_nodelay(enabled).map_err(|value|io_error("setNoDelay",value))}
  pub async fn shutdown_write(&mut self)->Result<(),Error>{self.inner.shutdown().await.map_err(|value|io_error("shutdownWrite",value))}
  pub fn split(self)->(crate::tsrust_native_io::Reader,crate::tsrust_native_io::Writer){let(reader,writer)=self.inner.into_split();(crate::tsrust_native_io::Reader::asynchronous(Box::new(reader)),crate::tsrust_native_io::Writer::asynchronous(Box::new(writer)))}
  pub fn close(self){drop(self)}
  pub async fn read_into(&mut self,bytes:&mut[u8])->Result<i64,crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::read_into(&mut self.inner,bytes).await}
  pub async fn read(&mut self,length:i64)->Result<Vec<u8>,crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::read(&mut self.inner,length).await}
  pub async fn read_exact(&mut self,bytes:&mut[u8])->Result<(),crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::read_exact(&mut self.inner,bytes).await}
  pub async fn read_all(&mut self,limit:Option<i64>)->Result<Vec<u8>,crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::read_all(&mut self.inner,limit).await}
  pub async fn write(&mut self,bytes:&[u8])->Result<i64,crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::write(&mut self.inner,bytes).await}
  pub async fn write_all(&mut self,bytes:&[u8])->Result<(),crate::tsrust_native_io::Error>{crate::tsrust_native_net_io::write_all(&mut self.inner,bytes).await}
  pub async fn flush(&mut self)->Result<(),crate::tsrust_native_io::Error>{self.inner.flush().await.map_err(|value|crate::tsrust_native_io::Error::io("flush",value))}
 }
 pub async fn bind_udp(address:&SocketAddress)->Result<Udp,Error>{runtime("bindUdp")?;tokio::net::UdpSocket::bind(*address).await.map(Udp).map_err(|value|io_error("bindUdp",value))}
 impl Udp{
  pub async fn send_to(&self,bytes:&[u8],address:&SocketAddress)->Result<i64,Error>{let count=self.0.send_to(bytes,*address).await.map_err(|value|io_error("sendTo",value))?;i64::try_from(count).map_err(|_|error("range","sendTo","Byte count exceeds int"))}
  pub async fn receive_from(&self,capacity:i64)->Result<Datagram,Error>{let capacity=usize::try_from(capacity).map_err(|_|error("range","receiveFrom","Capacity must be nonnegative and fit target"))?;let mut data=Vec::new();data.try_reserve_exact(capacity).map_err(|value|error("allocation","receiveFrom",value))?;data.resize(capacity,0);let(count,remote)=self.0.recv_from(&mut data).await.map_err(|value|io_error("receiveFrom",value))?;data.truncate(count);Ok(Datagram{data,remote})}
  pub fn local_address(&self)->Result<SocketAddress,Error>{self.0.local_addr().map_err(|value|io_error("localAddress",value))}
  pub fn close(self){drop(self)}
 }
}
