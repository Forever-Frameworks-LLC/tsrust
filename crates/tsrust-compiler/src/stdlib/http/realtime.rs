//! Realtime HTTP operations share native signatures and ordinary owner checking.
use super::*;
const EVENTS:Signature=Signature::Applied(NativeTypeId::AsyncChannel,&[Signature::Named(NativeTypeId::HttpSseEvent)]);
const EVENT_INPUT:NativeParameter=NativeParameter{name:"events",ty:NativeType::Structured(&EVENTS),access:Mode::Consume,optional:false,rest:false};
const SSE_OPTIONS:NativeParameter=NativeParameter{name:"options",ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpSseOptions)),access:Mode::Read,optional:true,rest:false};
pub(crate) const SSE:NativeBinding=NativeBinding{module:"std:http",name:"__sse",parameters:&[
 EVENT_INPUT,SSE_OPTIONS,
],result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::HttpResponse),&Signature::Named(NativeTypeId::HttpError))),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"http_intrinsic"},features:&["http"]};
pub(crate) const PUBLIC_SSE:NativeBinding=NativeBinding{name:"sse",..SSE};
pub(crate) const SSE_PRODUCER:NativeBinding=NativeBinding{name:"__sseWithProducer",parameters:&[
 EVENT_INPUT,NativeParameter{name:"producer",ty:NativeType::ObservedValue,access:Mode::Consume,optional:false,rest:false},SSE_OPTIONS,
],..SSE};
pub(crate) const PUBLIC_SSE_PRODUCER:NativeBinding=NativeBinding{name:"sseWithProducer",..SSE_PRODUCER};

pub(crate) const MESSAGE:Signature=Signature::Union(&[Signature::Named(NativeTypeId::HttpWebSocketText),Signature::Named(NativeTypeId::HttpWebSocketBinary),Signature::Named(NativeTypeId::HttpWebSocketClose)]);
pub(crate) const CONNECT:NativeBinding=NativeBinding{module:"std:http",name:"__connectWebSocket",parameters:&[
 NativeParameter{name:"url",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},NativeParameter{name:"options",ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketClientOptions)),access:Mode::Read,optional:true,rest:false},
],result:NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::HttpWebSocket),&Signature::Named(NativeTypeId::HttpError))),features:&["http-websocket"],..SSE};
pub(crate) const PUBLIC_CONNECT:NativeBinding=NativeBinding{name:"connectWebSocket",..CONNECT};
const SOCKET_INPUT:NativeParameter=NativeParameter{name:"socket",ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocket)),access:Mode::Mutate,optional:false,rest:false};
const OWNED_SOCKET_INPUT:NativeParameter=NativeParameter{access:Mode::Consume,..SOCKET_INPUT};
const MESSAGE_INPUT:NativeParameter=NativeParameter{name:"message",ty:NativeType::Structured(&MESSAGE),access:Mode::Consume,optional:false,rest:false};
const CLOSE_FRAME:NativeParameter=NativeParameter{name:"frame",ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketClose)),access:Mode::Consume,optional:true,rest:false};
pub(crate) const RECEIVE:NativeBinding=NativeBinding{name:"__webSocketReceive",parameters:&[SOCKET_INPUT],result:NativeType::Structured(&Signature::Result(&Signature::Optional(&MESSAGE),&Signature::Named(NativeTypeId::HttpError))),..CONNECT};
pub(crate) const READ_RECEIVE:NativeBinding=NativeBinding{name:"__webSocketReadReceive",parameters:&[NativeParameter{ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketRead)),..SOCKET_INPUT}],..RECEIVE};
pub(crate) const SEND:NativeBinding=NativeBinding{name:"__webSocketSend",parameters:&[SOCKET_INPUT,MESSAGE_INPUT],result:NativeType::Structured(&Signature::Result(&Signature::Void,&Signature::Named(NativeTypeId::HttpError))),..CONNECT};
pub(crate) const WRITE_SEND:NativeBinding=NativeBinding{name:"__webSocketWriteSend",parameters:&[NativeParameter{ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketWrite)),..SOCKET_INPUT},MESSAGE_INPUT],..SEND};
pub(crate) const SPLIT:NativeBinding=NativeBinding{name:"__webSocketSplit",parameters:&[OWNED_SOCKET_INPUT],result:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketHalves)),..CONNECT};
pub(crate) const CLOSE:NativeBinding=NativeBinding{name:"__webSocketClose",parameters:&[OWNED_SOCKET_INPUT,CLOSE_FRAME],result:SEND.result,..CONNECT};
pub(crate) const WRITE_CLOSE:NativeBinding=NativeBinding{name:"__webSocketWriteClose",parameters:&[NativeParameter{ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketWrite)),..OWNED_SOCKET_INPUT},CLOSE_FRAME],..CLOSE};
pub(crate) const ROUTE:NativeBinding=NativeBinding{name:"__routerWebSocket",parameters:&[
 NativeParameter{name:"app",ty:NativeType::ObservedValue,access:Mode::Mutate,optional:false,rest:false},NativeParameter{name:"path",ty:NativeType::String,access:Mode::Read,optional:false,rest:false},NativeParameter{name:"handler",ty:NativeType::ObservedValue,access:Mode::Consume,optional:false,rest:false},NativeParameter{name:"options",ty:NativeType::Structured(&Signature::Named(NativeTypeId::HttpWebSocketOptions)),access:Mode::Read,optional:true,rest:false},
],result:SEND.result,..CONNECT};

pub(crate) fn asynchronous(name:&str)->bool{matches!(name,"__connectWebSocket"|"__webSocketReceive"|"__webSocketReadReceive"|"__webSocketSend"|"__webSocketWriteSend"|"__webSocketClose"|"__webSocketWriteClose")}

// Prepare this finite family only after selecting a socket type or operation.
pub(crate) const OPERATIONS:&[NativeBinding]=&[CONNECT,RECEIVE,READ_RECEIVE,SEND,WRITE_SEND,SPLIT,CLOSE,WRITE_CLOSE,ROUTE];
pub(crate) fn socket_type(id:NativeTypeId)->bool{matches!(id,NativeTypeId::HttpWebSocket|NativeTypeId::HttpWebSocketRead|NativeTypeId::HttpWebSocketWrite|NativeTypeId::HttpWebSocketHalves|NativeTypeId::HttpWebSocketOptions|NativeTypeId::HttpWebSocketClientOptions|NativeTypeId::HttpWebSocketHeader|NativeTypeId::HttpWebSocketText|NativeTypeId::HttpWebSocketBinary|NativeTypeId::HttpWebSocketClose)}
