//! Thin transport lifecycle and B05 full-stream composition witnesses.
#[path="../src/stdlib/fs/runtime/resource.rs"]pub(crate) mod resource;
#[path="../src/stdlib/fs/runtime/fs.rs"]pub(crate) mod fs;
#[path="../src/stdlib/fs/runtime/buffered_fs.rs"]pub(crate) mod buffered_fs;
mod tsrust_native_files{pub(crate)use crate::{fs,buffered_fs};}
include!("../src/stdlib/futures/runtime.rs");
include!("../src/stdlib/io/runtime.rs");
include!("../src/stdlib/net/runtime.rs");
include!("../src/stdlib/net/transport_runtime.rs");
include!("../src/stdlib/net/io_runtime.rs");
use tsrust_native_net_transport::{Stream,Listener,Udp};
#[test]
#[ignore="Requires an explicitly selected host supporting 60000-byte UDP datagrams; not a throughput measurement."]
fn udp_large_datagram_delivers_all_60000_bytes(){
 tsrust_native_futures::block_on(async{
  tokio::time::timeout(std::time::Duration::from_secs(10),async{
   let sender=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();
   let receiver=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();
   let data=(0..60_000usize).map(|index|((index*31+(index>>8))%256)as u8).collect::<Vec<_>>();
   assert_eq!(sender.send_to(&data,&receiver.local_address().unwrap()).await.expect("host does not support the required 60000-byte datagram; this is not a passing capability result"),60_000);
   let received=receiver.receive_from(60_000).await.unwrap();
   assert_eq!(received.data,data);assert_eq!(received.remote,sender.local_address().unwrap());
   receiver.close();sender.close();
  }).await.expect("60000-byte UDP delivery timed out");
 });
}
async fn pair()->(Stream,Stream){let listener=tsrust_native_net_transport::listen(&"127.0.0.1:0".parse().unwrap()).await.unwrap();let address=listener.local_address().unwrap();let accepted=tokio::spawn(async move{let(stream,_)=listener.accept().await.unwrap();listener.close();stream});let client=tsrust_native_net_transport::connect(&address).await.unwrap();(client,accepted.await.unwrap())}
#[test]
fn native_network_owners_and_borrowed_operations_are_send_sync(){fn send_sync<T:Send+Sync>(){}send_sync::<Listener>();send_sync::<Stream>();send_sync::<Udp>();fn send<T:Send>(_:T){}tsrust_native_futures::block_on(async{let(mut client,mut peer)=pair().await;send(client.read_all(None));send(peer.shutdown_write());let socket=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();send(socket.receive_from(1));socket.close();client.close();peer.close();});}
#[test]
fn full_stream_read_write_half_close_and_bounded_progress(){tsrust_native_futures::block_on(async{let(mut client,mut peer)=pair().await;assert!(!client.inner.nodelay().unwrap());client.set_no_delay(true).unwrap();assert!(client.inner.nodelay().unwrap());client.set_no_delay(false).unwrap();assert!(!client.inner.nodelay().unwrap());assert_eq!(client.peer_address().unwrap(),peer.local_address().unwrap());let worker=tokio::spawn(async move{let bytes=peer.read_all(None).await.unwrap();assert_eq!(bytes,[0,255,1,2,3]);peer.write_all(b"reply after EOF").await.unwrap();peer.shutdown_write().await.unwrap();peer.close();});client.write_all(&[0,255,1,2,3]).await.unwrap();client.flush().await.unwrap();client.shutdown_write().await.unwrap();assert_eq!(client.read_all(None).await.unwrap(),b"reply after EOF");client.close();worker.await.unwrap();let(mut client,mut peer)=pair().await;peer.write_all(b"abcd").await.unwrap();peer.shutdown_write().await.unwrap();assert_eq!(client.read_all(Some(2)).await.unwrap_err().code,"limit");assert_eq!(client.read_all(None).await.unwrap(),b"d");client.close();peer.close();});}
#[test]
fn owned_halves_drop_eof_and_retained_reader_survives_writer(){tsrust_native_futures::block_on(async{let(client,mut peer)=pair().await;let(mut reader,writer)=client.split();drop(writer);assert_eq!(peer.read(1).await.unwrap(),b"");peer.write_all(b"after close").await.unwrap();peer.shutdown_write().await.unwrap();assert_eq!(reader.read_all(None).await.unwrap(),b"after close");reader.close().await.unwrap();peer.close();});}
#[test]
fn copy_composes_all_finite_tcp_io_endpoint_shapes(){tsrust_native_futures::block_on(async{
 let(mut client,mut peer)=pair().await;let mut input=tsrust_native_io::Reader::bytes(b"memory to full".to_vec());assert_eq!(tsrust_native_net_io::copy(tsrust_native_net_io::CopyReader::Io(&mut input),tsrust_native_net_io::CopyWriter::Tcp(&mut client),3,None).await.unwrap(),14);client.shutdown_write().await.unwrap();assert_eq!(peer.read_all(None).await.unwrap(),b"memory to full");input.close().await.unwrap();client.close();peer.close();
 let(mut input,mut source)=pair().await;let(mut destination,mut output)=pair().await;source.write_all(b"full to full").await.unwrap();source.shutdown_write().await.unwrap();assert_eq!(tsrust_native_net_io::copy(tsrust_native_net_io::CopyReader::Tcp(&mut input),tsrust_native_net_io::CopyWriter::Tcp(&mut destination),3,None).await.unwrap(),12);destination.shutdown_write().await.unwrap();assert_eq!(output.read_all(None).await.unwrap(),b"full to full");input.close();source.close();destination.close();output.close();
 let(mut input,mut source)=pair().await;let(destination,mut output)=pair().await;let(read_half,mut writer)=destination.split();read_half.close().await.unwrap();source.write_all(b"full to half").await.unwrap();source.shutdown_write().await.unwrap();assert_eq!(tsrust_native_net_io::copy(tsrust_native_net_io::CopyReader::Tcp(&mut input),tsrust_native_net_io::CopyWriter::Io(&mut writer),3,None).await.unwrap(),12);writer.close().await.unwrap();assert_eq!(output.read_all(None).await.unwrap(),b"full to half");input.close();source.close();output.close();});}
#[test]
fn borrowed_accept_and_receive_cancellation_retain_socket_owners(){tsrust_native_futures::block_on(async{let listener=tsrust_native_net_transport::listen(&"127.0.0.1:0".parse().unwrap()).await.unwrap();assert!(tokio::time::timeout(std::time::Duration::from_millis(5),listener.accept()).await.is_err());let address=listener.local_address().unwrap();let connected=tokio::spawn(async move{tsrust_native_net_transport::connect(&address).await.unwrap()});let(peer,_)=listener.accept().await.unwrap();let client=connected.await.unwrap();peer.close();client.close();listener.close();let receiver=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();assert!(tokio::time::timeout(std::time::Duration::from_millis(5),receiver.receive_from(10)).await.is_err());let sender=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();sender.send_to(b"kept",&receiver.local_address().unwrap()).await.unwrap();assert_eq!(receiver.receive_from(10).await.unwrap().data,b"kept");receiver.close();sender.close();});}
#[test]
fn udp_empty_boundaries_zero_capacity_and_explicit_negative_range(){tsrust_native_futures::block_on(async{let sender=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();let receiver=tsrust_native_net_transport::bind_udp(&"127.0.0.1:0".parse().unwrap()).await.unwrap();assert_eq!(receiver.receive_from(-1).await.err().unwrap().code,"range");let address=receiver.local_address().unwrap();sender.send_to(&[],&address).await.unwrap();let message=receiver.receive_from(0).await.unwrap();assert!(message.data.is_empty());assert_eq!(message.remote,sender.local_address().unwrap());sender.send_to(&[0,255,7],&address).await.unwrap();assert_eq!(receiver.receive_from(3).await.unwrap().data,[0,255,7]);sender.send_to(&[1,2,3],&address).await.unwrap();match receiver.receive_from(1).await{Ok(message)=>assert_eq!(message.data,[1]),Err(error)=>assert!(error.os_code.is_some())}sender.send_to(&[9],&address).await.unwrap();assert_eq!(receiver.receive_from(1).await.unwrap().data,[9]);receiver.close();sender.close();});}
