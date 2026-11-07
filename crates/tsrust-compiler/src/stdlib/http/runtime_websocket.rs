#[cfg(feature="http-websocket")]
mod websocket {
    use super::*;
    use futures_util::{SinkExt,StreamExt,stream::{SplitSink,SplitStream}};
    use tokio_tungstenite::{WebSocketStream,MaybeTlsStream,tungstenite::{self,Message as ClientMessage,protocol::{WebSocketConfig,frame::coding::CloseCode}}};
    type ClientSocket=WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
    type ServerSocket=axum::extract::ws::WebSocket;
    #[derive(Default,Clone)]
    pub struct Options{
        pub read_buffer_size:Option<i64>,pub write_buffer_size:Option<i64>,pub max_write_buffer_size:Option<Option<i64>>,
        pub max_message_size:Option<Option<i64>>,pub max_frame_size:Option<Option<i64>>,pub accept_unmasked_frames:Option<bool>,
    }
    impl Options{
        fn config(&self)->Result<WebSocketConfig,Error>{
            let mut config=WebSocketConfig::default();
            if let Some(value)=self.read_buffer_size{config.read_buffer_size=size(value)?;}
            if let Some(value)=self.write_buffer_size{config.write_buffer_size=size(value)?;}
            if let Some(value)=self.max_write_buffer_size{config.max_write_buffer_size=value.map(size).transpose()?.unwrap_or(usize::MAX);}
            if let Some(value)=self.max_message_size{config.max_message_size=value.map(size).transpose()?;}
            if let Some(value)=self.max_frame_size{config.max_frame_size=value.map(size).transpose()?;}
            if let Some(value)=self.accept_unmasked_frames{config.accept_unmasked_frames=value;}
            if config.max_write_buffer_size<=config.write_buffer_size{return Err(error("options","maxWriteBufferSize must exceed writeBufferSize"));}
            Ok(config)
        }
    }
    fn size(value:i64)->Result<usize,Error>{usize::try_from(value).map_err(|_|error("options","WebSocket sizes must be nonnegative and target-representable"))}
    pub enum Message{Text(String),Binary(Vec<u8>),Close{code:Option<i64>,reason:Option<String>}}
    pub enum Socket{Server(ServerSocket),Client(ClientSocket)}
    pub enum ReadHalf{Server(SplitStream<ServerSocket>),Client(SplitStream<ClientSocket>)}
    pub enum WriteHalf{Server(SplitSink<ServerSocket,axum::extract::ws::Message>),Client(SplitSink<ClientSocket,ClientMessage>)}
    pub struct Halves{pub read:ReadHalf,pub write:WriteHalf}
    fn backend_error(error:tungstenite::Error)->Error{
        let code=match error{tungstenite::Error::Protocol(_)=>"protocol",tungstenite::Error::Utf8(_)=>"utf8",tungstenite::Error::Capacity(_)=>"capacity",tungstenite::Error::WriteBufferFull(_)=>"write-buffer-full",tungstenite::Error::ConnectionClosed|tungstenite::Error::AlreadyClosed=>"closed",tungstenite::Error::Tls(_)=>"tls",tungstenite::Error::Url(_)=>"url",tungstenite::Error::Http(_)|tungstenite::Error::HttpFormat(_)=>"handshake",tungstenite::Error::Io(_)=>"io",_=>"websocket"};
        // Transport failures must not disclose request credentials or headers.
        super::error(code,"WebSocket transport failed")
    }
    fn server_error(error:axum::Error)->Error{match error.into_inner().downcast::<tungstenite::Error>(){Ok(error)=>backend_error(*error),Err(_)=>super::error("websocket","WebSocket transport failed")}}
    fn close_frame(code:Option<i64>,reason:Option<String>)->Result<Option<tungstenite::protocol::CloseFrame>,Error>{
        if code.is_none()&&reason.is_none(){return Ok(None);}
        let code=u16::try_from(code.unwrap_or(1000)).map_err(|_|error("options","Close code must be an allowed16-bit backend close code"))?;
        if !CloseCode::from(code).is_allowed(){return Err(error("options","Close code is reserved or invalid"));}
        let reason=reason.unwrap_or_default();
        // The maintained backend's control-frame payload includes the two-byte code.
        if reason.len()>123{return Err(error("options","Close reason exceeds the backend123-byte control-frame range"));}
        Ok(Some(tungstenite::protocol::CloseFrame{code:code.into(),reason:reason.into()}))
    }
    fn client_message(message:Message)->Result<ClientMessage,Error>{Ok(match message{
        Message::Text(text)=>ClientMessage::Text(text.into()),Message::Binary(bytes)=>ClientMessage::Binary(bytes.into()),
        Message::Close{code,reason}=>ClientMessage::Close(close_frame(code,reason)?),
    })}
    fn server_message(message:Message)->Result<axum::extract::ws::Message,Error>{Ok(match message{
        Message::Text(text)=>axum::extract::ws::Message::Text(text.into()),Message::Binary(bytes)=>axum::extract::ws::Message::Binary(bytes.into()),
        Message::Close{code,reason}=>axum::extract::ws::Message::Close(close_frame(code,reason)?.map(|frame|axum::extract::ws::CloseFrame{code:frame.code.into(),reason:axum::extract::ws::Utf8Bytes::try_from(axum::body::Bytes::from(frame.reason)).expect("backend-validated UTF8")})),
    })}
    fn received_client(message:ClientMessage)->Result<Option<Message>,Error>{Ok(match message{
        ClientMessage::Text(text)=>{let bytes:axum::body::Bytes=text.into();Some(Message::Text(String::from_utf8(Vec::from(bytes)).expect("backend-validated UTF8")))},
        ClientMessage::Binary(bytes)=>Some(Message::Binary(Vec::from(bytes))),
        ClientMessage::Close(frame)=>Some(Message::Close{code:frame.as_ref().map(|frame|i64::from(u16::from(frame.code))),reason:frame.map(|frame|String::from_utf8(Vec::from(axum::body::Bytes::from(frame.reason))).expect("backend-validated UTF8"))}),
        ClientMessage::Ping(_)|ClientMessage::Pong(_)=>None,
        ClientMessage::Frame(_)=>return Err(error("protocol","Backend exposed an unexpected raw frame")),
    })}
    fn received_server(message:axum::extract::ws::Message)->Option<Message>{match message{
        axum::extract::ws::Message::Text(text)=>{let bytes:axum::body::Bytes=text.into();Some(Message::Text(String::from_utf8(Vec::from(bytes)).expect("backend-validated UTF8")))},
        axum::extract::ws::Message::Binary(bytes)=>Some(Message::Binary(Vec::from(bytes))),
        axum::extract::ws::Message::Close(frame)=>Some(Message::Close{code:frame.as_ref().map(|frame|i64::from(frame.code)),reason:frame.map(|frame|String::from_utf8(Vec::from(axum::body::Bytes::from(frame.reason))).expect("backend-validated UTF8"))}),
        axum::extract::ws::Message::Ping(_)|axum::extract::ws::Message::Pong(_)=>None,
    }}
    impl Socket{
        pub fn split(self)->Halves{match self{Self::Server(socket)=>{let(write,read)=socket.split();Halves{read:ReadHalf::Server(read),write:WriteHalf::Server(write)}},Self::Client(socket)=>{let(write,read)=socket.split();Halves{read:ReadHalf::Client(read),write:WriteHalf::Client(write)}}}}
        pub async fn receive(&mut self)->Result<Option<Message>,Error>{loop{match self{
            Self::Server(socket)=>match socket.next().await{Some(Ok(message))=>if let Some(message)=received_server(message){return Ok(Some(message));},Some(Err(error))=>return Err(server_error(error)),None=>return Ok(None)},
            Self::Client(socket)=>match socket.next().await{Some(Ok(message))=>if let Some(message)=received_client(message)?{return Ok(Some(message));},Some(Err(error))=>return Err(backend_error(error)),None=>return Ok(None)},
        }}}
        pub async fn send(&mut self,message:Message)->Result<(),Error>{match self{Self::Server(socket)=>socket.send(server_message(message)?).await.map_err(server_error),Self::Client(socket)=>socket.send(client_message(message)?).await.map_err(backend_error)}}
        pub async fn close(mut self,code:Option<i64>,reason:Option<String>)->Result<(),Error>{
            if code.is_some()||reason.is_some(){self.send(Message::Close{code,reason}).await?;}
            match &mut self{Self::Server(socket)=>SinkExt::close(socket).await.map_err(server_error)?,Self::Client(socket)=>SinkExt::close(socket).await.map_err(backend_error)?}
            while let Some(message)=self.receive().await?{if matches!(message,Message::Close{..}){break;}}Ok(())
        }
    }
    impl ReadHalf{
        pub async fn receive(&mut self)->Result<Option<Message>,Error>{loop{match self{
            Self::Server(socket)=>match socket.next().await{Some(Ok(message))=>if let Some(message)=received_server(message){return Ok(Some(message));},Some(Err(error))=>return Err(server_error(error)),None=>return Ok(None)},
            Self::Client(socket)=>match socket.next().await{Some(Ok(message))=>if let Some(message)=received_client(message)?{return Ok(Some(message));},Some(Err(error))=>return Err(backend_error(error)),None=>return Ok(None)},
        }}}
    }
    impl WriteHalf{
        pub async fn send(&mut self,message:Message)->Result<(),Error>{match self{Self::Server(socket)=>socket.send(server_message(message)?).await.map_err(server_error),Self::Client(socket)=>socket.send(client_message(message)?).await.map_err(backend_error)}}
        pub async fn close(mut self,code:Option<i64>,reason:Option<String>)->Result<(),Error>{
            if code.is_some()||reason.is_some(){self.send(Message::Close{code,reason}).await?;}
            match &mut self{Self::Server(socket)=>SinkExt::close(socket).await.map_err(server_error),Self::Client(socket)=>SinkExt::close(socket).await.map_err(backend_error)}
        }
    }
    pub async fn connect(url:&str,ca:Option<&str>,headers:impl IntoIterator<Item=(String,String)>,options:Options)->Result<Socket,Error>{
        use tungstenite::client::IntoClientRequest;
        let config=options.config()?;
        let parsed=reqwest::Url::parse(url).map_err(|_|error("url","Use an absolute ws or wss URL"))?;
        if !matches!(parsed.scheme(),"ws"|"wss"){return Err(error("url","Use an absolute ws or wss URL"));}
        let mut request=url.into_client_request().map_err(backend_error)?;
        for(name,value)in headers{
            let name=HeaderName::from_bytes(name.as_bytes()).map_err(|_|error("options","Invalid WebSocket request header name"))?;
            let value=HeaderValue::from_str(&value).map_err(|_|error("options","Invalid WebSocket request header value"))?;
            request.headers_mut().append(name,value);
        }
        let mut roots=if ca.is_some(){rustls::RootCertStore::empty()}else{rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned())};
        if let Some(ca)=ca{for certificate in certificates(ca)?{roots.add(certificate).map_err(|_|error("tls","Invalid CA certificate"))?;}}
        let config_tls=rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider())).with_safe_default_protocol_versions().map_err(|_|error("tls","Invalid TLS protocol configuration"))?.with_root_certificates(roots).with_no_client_auth();
        let connector=tokio_tungstenite::Connector::Rustls(Arc::new(config_tls));
        tokio_tungstenite::connect_async_tls_with_config(request,Some(config),false,Some(connector)).await.map(|(socket,_)|Socket::Client(socket)).map_err(backend_error)
    }
    impl<S:Send+Sync+'static> Router<S>{
        pub fn websocket<F,Fut>(&mut self,path:&str,options:Options,handler:F)->Result<(),Error>where F:Fn(Socket)->Fut+Send+Sync+'static,Fut:Future<Output=()>+Send+'static{
            use axum::{extract::FromRequestParts,response::IntoResponse};
            let config=options.config()?;let handler=Arc::new(handler);
            self.route("GET",path,move|context|{
                let handler=handler.clone();Box::pin(async move{
                    let request=std::mem::replace(&mut context.field0.inner,http::Request::new(ExclusiveBody::new(Body::empty())));
                    let(mut parts,body)=request.into_parts();
                    let upgrade=axum::extract::WebSocketUpgrade::from_request_parts(&mut parts,&()).await;
                    context.field0.inner=http::Request::from_parts(parts,body);
                    let upgrade=match upgrade{Ok(upgrade)=>upgrade,Err(rejection)=>return Some(Response{inner:rejection.into_response().map(ExclusiveBody::new),limit:None})};
                    let upgrade=upgrade.read_buffer_size(config.read_buffer_size).write_buffer_size(config.write_buffer_size).max_write_buffer_size(config.max_write_buffer_size).accept_unmasked_frames(config.accept_unmasked_frames);
                    // Axum's setters accept concrete bounds only. Its default is the
                    // same selected WebSocketConfig; an explicitly unlimited bound uses usize::MAX.
                    let upgrade=upgrade.max_message_size(config.max_message_size.unwrap_or(usize::MAX)).max_frame_size(config.max_frame_size.unwrap_or(usize::MAX));
                    let(sender,receiver)=tokio::sync::oneshot::channel();
                    // Reserve the scoped owner before handing the response to Hyper.
                    // Failed/cancelled upgrades close the oneshot without user work.
                    let job=tasks::start(async move{if let Ok(socket)=receiver.await{handler(Socket::Server(socket)).await;}Ok::<_,tasks::Failure<()>>(())});
                    // Keep the handle alive through response delivery so request
                    // completion reparents it; joining here would block HTTP101.
                    let response=upgrade.on_upgrade(move|socket|async move{let _=sender.send(socket);drop(job.join().await);}).into_response();
                    Some(Response{inner:response.map(ExclusiveBody::new),limit:None})
                })
            })
        }
    }
}
#[cfg(feature="http-websocket")]
pub use websocket::{Socket as WebSocket,ReadHalf as WebSocketRead,WriteHalf as WebSocketWrite,Halves as WebSocketHalves,Message as WebSocketMessage,Options as WebSocketOptions,connect as connect_websocket};
