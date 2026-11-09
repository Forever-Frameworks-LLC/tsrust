use crate::{Mode,native_bindings::{NativeBinding,NativeBackend,NativeParameter,NativeType},native_signatures::{Signature,NativeTypeId}};
const IP:Signature=Signature::Named(NativeTypeId::IpAddress);
const ADDRESS:Signature=Signature::Named(NativeTypeId::SocketAddress);
const ERROR:Signature=Signature::Named(NativeTypeId::NetError);
const FAMILY:Signature=Signature::Union(&[Signature::StringLiteral("any"),Signature::StringLiteral("ipv4"),Signature::StringLiteral("ipv6")]);
const fn parameter(name:&'static str,signature:&'static Signature,access:Mode,optional:bool)->NativeParameter{NativeParameter{name,ty:NativeType::Structured(signature),access,optional,rest:false}}
const fn binding(name:&'static str,parameters:&'static[NativeParameter],result:&'static Signature)->NativeBinding{NativeBinding{module:"std:net",name,parameters,result:NativeType::Structured(result),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"net_intrinsic"},features:&[]}}
pub(crate) const PARSE_IP:NativeBinding=binding("parseIp",&[parameter("text",&Signature::String,Mode::Read,false)],&Signature::Result(&IP,&ERROR));
pub(crate) const FORMAT_IP:NativeBinding=binding("formatIp",&[parameter("address",&IP,Mode::Read,false)],&Signature::String);
pub(crate) const PARSE_ADDRESS:NativeBinding=binding("parseAddress",&[parameter("text",&Signature::String,Mode::Read,false)],&Signature::Result(&ADDRESS,&ERROR));
pub(crate) const FORMAT_ADDRESS:NativeBinding=binding("formatAddress",&[parameter("address",&ADDRESS,Mode::Read,false)],&Signature::String);
pub(crate) const IS_IPV4:NativeBinding=binding("isIPv4",&[parameter("address",&IP,Mode::Read,false)],&Signature::Boolean);
pub(crate) const IS_IPV6:NativeBinding=binding("isIPv6",&[parameter("address",&IP,Mode::Read,false)],&Signature::Boolean);
pub(crate) const LOOKUP:NativeBinding=NativeBinding{module:"std:dns",name:"lookup",parameters:&[parameter("host",&Signature::String,Mode::Read,false),parameter("family",&FAMILY,Mode::Read,true)],result:NativeType::Structured(&Signature::Result(&Signature::Array(&IP),&ERROR)),features:&["async"],..binding("lookup",&[],&Signature::Void)};
pub(crate) const LOOKUP_POLL:NativeBinding=NativeBinding{name:"__lookup",..LOOKUP};
pub(crate) const OPERATIONS:&[NativeBinding]=&[PARSE_IP,FORMAT_IP,PARSE_ADDRESS,FORMAT_ADDRESS,IS_IPV4,IS_IPV6,LOOKUP,LOOKUP_POLL];

const LISTENER:Signature=Signature::Named(NativeTypeId::TcpListener);
const STREAM:Signature=Signature::Named(NativeTypeId::TcpStream);
const UDP:Signature=Signature::Named(NativeTypeId::UdpSocket);
const fn transport(name:&'static str,parameters:&'static[NativeParameter],result:&'static Signature)->NativeBinding{NativeBinding{features:&["io","async"],..binding(name,parameters,result)}}
pub(crate) const LISTEN:NativeBinding=transport("listenTcp",&[parameter("address",&ADDRESS,Mode::Read,false)],&Signature::Result(&LISTENER,&ERROR));
pub(crate) const LISTEN_POLL:NativeBinding=NativeBinding{name:"__listenTcp",..LISTEN};
pub(crate) const CONNECT:NativeBinding=transport("connectTcp",&[parameter("address",&ADDRESS,Mode::Read,false)],&Signature::Result(&STREAM,&ERROR));
pub(crate) const CONNECT_POLL:NativeBinding=NativeBinding{name:"__connectTcp",..CONNECT};
pub(crate) const BIND_UDP:NativeBinding=transport("bindUdp",&[parameter("address",&ADDRESS,Mode::Read,false)],&Signature::Result(&UDP,&ERROR));
pub(crate) const BIND_UDP_POLL:NativeBinding=NativeBinding{name:"__bindUdp",..BIND_UDP};
pub(crate) const ACCEPT:NativeBinding=transport("__accept",&[parameter("listener",&LISTENER,Mode::Read,false)],&Signature::Result(&Signature::Tuple(&[STREAM,ADDRESS]),&ERROR));
pub(crate) const LISTENER_ADDRESS:NativeBinding=transport("__listenerAddress",&[parameter("listener",&LISTENER,Mode::Read,false)],&Signature::Result(&ADDRESS,&ERROR));
pub(crate) const LISTENER_CLOSE:NativeBinding=transport("__listenerClose",&[parameter("listener",&LISTENER,Mode::Consume,false)],&Signature::Void);
pub(crate) const STREAM_LOCAL:NativeBinding=transport("__streamLocal",&[parameter("stream",&STREAM,Mode::Read,false)],&Signature::Result(&ADDRESS,&ERROR));
pub(crate) const STREAM_PEER:NativeBinding=transport("__streamPeer",&[parameter("stream",&STREAM,Mode::Read,false)],&Signature::Result(&ADDRESS,&ERROR));
pub(crate) const NO_DELAY:NativeBinding=transport("__setNoDelay",&[parameter("stream",&STREAM,Mode::Mutate,false),parameter("enabled",&Signature::Boolean,Mode::Copy,false)],&Signature::Result(&Signature::Void,&ERROR));
pub(crate) const SHUTDOWN:NativeBinding=transport("__shutdownWrite",&[parameter("stream",&STREAM,Mode::Mutate,false)],&Signature::Result(&Signature::Void,&ERROR));
pub(crate) const SPLIT:NativeBinding=transport("__split",&[parameter("stream",&STREAM,Mode::Consume,false)],&Signature::Tuple(&[Signature::Named(NativeTypeId::IoReader),Signature::Named(NativeTypeId::IoWriter)]));
pub(crate) const STREAM_CLOSE:NativeBinding=transport("__streamClose",&[parameter("stream",&STREAM,Mode::Consume,false)],&Signature::Void);
pub(crate) const SEND_TO:NativeBinding=transport("__sendTo",&[parameter("socket",&UDP,Mode::Read,false),parameter("bytes",&Signature::Named(NativeTypeId::Bytes),Mode::Read,false),parameter("address",&ADDRESS,Mode::Read,false)],&Signature::Result(&Signature::Int,&ERROR));
pub(crate) const RECEIVE_FROM:NativeBinding=transport("__receiveFrom",&[parameter("socket",&UDP,Mode::Read,false),parameter("capacity",&Signature::Int,Mode::Copy,false)],&Signature::Result(&Signature::Named(NativeTypeId::Datagram),&ERROR));
pub(crate) const UDP_ADDRESS:NativeBinding=transport("__udpAddress",&[parameter("socket",&UDP,Mode::Read,false)],&Signature::Result(&ADDRESS,&ERROR));
pub(crate) const UDP_CLOSE:NativeBinding=transport("__udpClose",&[parameter("socket",&UDP,Mode::Consume,false)],&Signature::Void);
pub(crate) const TRANSPORT:&[NativeBinding]=&[LISTEN,LISTEN_POLL,CONNECT,CONNECT_POLL,BIND_UDP,BIND_UDP_POLL,ACCEPT,LISTENER_ADDRESS,LISTENER_CLOSE,STREAM_LOCAL,STREAM_PEER,NO_DELAY,SHUTDOWN,SPLIT,STREAM_CLOSE,SEND_TO,RECEIVE_FROM,UDP_ADDRESS,UDP_CLOSE];
