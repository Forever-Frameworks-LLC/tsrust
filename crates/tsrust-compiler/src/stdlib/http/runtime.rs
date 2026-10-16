mod tsrust_native_http {
    use std::{collections::{BTreeMap,BTreeSet},future::{Future},pin::Pin,sync::Arc,task::Poll,time::Duration};
    use axum::{body::Body,http::{self,HeaderName,HeaderValue,Method,StatusCode},response::Response as WireResponse};
    use http_body_util::BodyExt;
    use tokio_rustls::{TlsAcceptor,rustls::{self,pki_types::{CertificateDer,PrivateKeyDer,pem::PemObject}}};
    use super::tsrust_native_futures as tasks;
    type Pending = Pin<Box<dyn Future<Output=Response>+Send>>;
    type Handler = Arc<dyn Fn(Request)->Pending+Send+Sync>;
    type Middleware = Arc<dyn Fn(Request,Next)->Pending+Send+Sync>;
    #[derive(Debug)]
    pub struct Error {pub code:&'static str,pub message:String}
    impl std::fmt::Display for Error{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(&self.message)}}
    impl std::error::Error for Error{}
    fn error(code:&'static str,message:impl Into<String>)->Error {Error{code,message:message.into()}}
    // Metadata can be borrowed across suspension. Body access still requires ownership;
    // Mutex supplies Sync without exposing a lock or shared access to the stream.
    struct ExclusiveBody(std::sync::Mutex<Body>);
    impl ExclusiveBody {
        fn new(body:Body)->Self{Self(std::sync::Mutex::new(body))}
        fn into_body(self)->Body{self.0.into_inner().unwrap_or_else(|poisoned|poisoned.into_inner())}
    }
    pub struct Request {inner:http::Request<ExclusiveBody>,params:BTreeMap<String,String>,limit:usize}
    pub struct Response {inner:http::Response<ExclusiveBody>}
    pub struct Stream {body:Body,remaining:usize}
    pub struct Client(reqwest::Client);
    pub struct Next {middleware:Arc<Vec<Middleware>>,index:usize,handler:Handler}
    impl Next {
        pub fn run(mut self,request:Request)->Pending {
            if let Some(middleware)=self.middleware.get(self.index).cloned(){self.index+=1;middleware(request,self)}else{(self.handler)(request)}
        }
    }
    type ContextPending<'a> = Pin<Box<dyn Future<Output=Option<Response>>+Send+'a>>;
    pub type ContextMiddleware<S=()> = Arc<dyn for<'a> Fn(&'a mut Context<S>,ContextNext)->ContextPending<'a>+Send+Sync>;

    // One exclusive context is reborrowed down the middleware stack. No mutex or
    // shared request handle can let a detached task outlive this request.
    pub struct Context<S=()> {
        pub field0:Request,
        pub field1:Arc<S>,
        variables:BTreeMap<String,Box<dyn std::any::Any+Send+Sync>>,
        pub field2:Response,
        finalized:bool,
        headers:http::HeaderMap,
        status:StatusCode,
        body_used:bool,
    }
    impl<S:Send+Sync+'static> Context<S> {
        fn new(request:Request,state:Arc<S>)->Self{Self{field0:request,field1:state,variables:BTreeMap::new(),field2:Response::text("Not Found".into(),404).unwrap(),finalized:false,headers:http::HeaderMap::new(),status:StatusCode::OK,body_used:false}}
        pub fn set<T:Send+Sync+'static>(&mut self,key:&str,value:T){self.variables.insert(key.to_owned(),Box::new(value));}
        pub fn get<T:Send+Sync+'static>(&self,key:&str)->Option<&T>{self.variables.get(key).map(|value|value.downcast_ref::<T>().expect("checked request variable type"))}
        pub fn header(&mut self,name:&str,value:&str)->Result<(),Error>{
            set_header(&mut self.headers,name,value)?;
            set_header(self.field2.inner.headers_mut(),name,value)?;
            Ok(())
        }
        pub fn status(&mut self,status:i64)->Result<(),Error>{self.status=status_code(status)?;Ok(())}
        pub fn content(&self,body:String,status:Option<i64>,content_type:&str)->Result<Response,Error>{
            let mut response=Response::content(body,status.unwrap_or(self.status.as_u16() as i64),content_type)?;
            for name in self.headers.keys(){response.inner.headers_mut().remove(name);for value in self.headers.get_all(name){response.inner.headers_mut().append(name,value.clone());}}
            Ok(response)
        }
        pub fn text(&self,body:String,status:Option<i64>)->Result<Response,Error>{self.content(body,status,"text/plain; charset=utf-8")}
        fn take_request_body(&mut self)->Result<Request,Error>{
            if self.body_used{return Err(error("body","Request body has already been consumed"));}
            self.body_used=true;
            let body=std::mem::replace(self.field0.inner.body_mut(),ExclusiveBody::new(Body::empty()));
            let mut request=Request::new(&self.field0.method(),&self.field0.url(),String::new())?;
            *request.inner.body_mut()=body;*request.inner.headers_mut()=self.field0.inner.headers().clone();request.limit=self.field0.limit;
            Ok(request)
        }
        pub async fn request_text(&mut self)->Result<String,Error>{self.take_request_body()?.text().await}
        pub async fn request_form(&mut self)->Result<Vec<FormField>,Error>{self.take_request_body()?.form().await}
        pub async fn request_multipart(&mut self)->Result<Vec<MultipartPart>,Error>{self.take_request_body()?.multipart().await}
        fn finish(self)->Response{if self.finalized{self.field2}else{failed()}}
    }
    #[derive(Clone)]
    struct ContextPattern {router:Option<matchit::Router<()>>}
    impl ContextPattern {
        fn new(path:&str)->Result<Self,Error>{
            if matches!(path,"*"|"/*"){return Ok(Self{router:None});}
            let wildcard=path.ends_with("/*");let base=if wildcard{&path[..path.len()-2]}else{path};
            let (normalized,_)=route_path(base)?;let mut router=matchit::Router::new();
            router.insert(normalized.clone(),()).map_err(|_|error("route","Invalid route pattern"))?;
            if wildcard{
                router.insert(format!("{normalized}/{{*__router_tail}}"),()).map_err(|_|error("route","Invalid wildcard route"))?;
                router.insert(format!("{normalized}/"),()).map_err(|_|error("route","Invalid wildcard route"))?;
            }
            Ok(Self{router:Some(router)})
        }
        fn captures(&self,path:&str)->Result<Option<BTreeMap<String,String>>,Error>{
            let Some(router)=&self.router else{return Ok(Some(BTreeMap::new()));};
            let Ok(matched)=router.at(path)else{return Ok(None);};let mut params=BTreeMap::new();
            for (name,value) in matched.params.iter().filter(|(name,_)|*name!="__router_tail"){
                params.insert(name.to_owned(),decode_component(value,false).map_err(|_|error("route","Invalid UTF-8 route parameter"))?);
            }
            Ok(Some(params))
        }
    }
    struct ContextLayer<S> {method:Option<Method>,pattern:ContextPattern,callback:ContextMiddleware<S>}
    pub struct ContextNext {layers:Arc<dyn std::any::Any+Send+Sync>,index:usize}
    impl ContextNext {
        pub fn run<'a,S:Send+Sync+'static>(mut self,context:&'a mut Context<S>)->Pin<Box<dyn Future<Output=()>+Send+'a>>{
            Box::pin(async move {
                let layers=self.layers.clone().downcast::<Vec<ContextLayer<S>>>().unwrap_or_else(|_|unreachable!("checked router state"));
                while let Some(layer)=layers.get(self.index){
                    self.index+=1;
                    if layer.method.as_ref().is_some_and(|method|method!=context.field0.inner.method() && !(method==Method::GET && context.field0.inner.method()==Method::HEAD)){continue;}
                    let params=match layer.pattern.captures(context.field0.inner.uri().path()){
                        Ok(Some(params))=>params,Ok(None)=>continue,Err(_)=>{context.field2=context.text("Bad Request".into(),Some(400)).unwrap();context.finalized=true;return;}
                    };
                    context.field0.params=params;
                    let callback=layer.callback.clone();
                    if let Some(response)=callback(context,self).await{context.field2=response;context.finalized=true;}
                    return;
                }
                context.field2=context.text("Not Found".into(),Some(404)).unwrap();context.finalized=true;
            })
        }
    }
    pub struct Router<S=()> {app:App,layers:Vec<ContextLayer<S>>,state:Arc<S>}
    impl Router<()> {pub fn new()->Self{Self::with_state(())}}
    impl<S:Send+Sync+'static> Router<S> {
        pub fn with_state(state:S)->Self{Self{app:App::new(),layers:Vec::new(),state:Arc::new(state)}}
        pub fn configure(&mut self,values:[i64;7])->Result<(),Error>{self.app.configure(values)}
        pub fn use_middleware<F>(&mut self,middleware:F) where F:for<'a> Fn(&'a mut Context<S>,ContextNext)->ContextPending<'a>+Send+Sync+'static{self.use_many("*",vec![Arc::new(middleware)]).expect("the global pattern is valid");}
        pub fn use_many(&mut self,path:&str,callbacks:Vec<ContextMiddleware<S>>)->Result<(),Error>{
            let pattern=ContextPattern::new(path)?;
            self.layers.extend(callbacks.into_iter().map(|callback|ContextLayer{method:None,pattern:pattern.clone(),callback}));Ok(())
        }
        pub fn route<F>(&mut self,method:&str,path:&str,handler:F)->Result<(),Error> where F:for<'a> Fn(&'a mut Context<S>)->ContextPending<'a>+Send+Sync+'static{
            self.route_many(method,path,vec![Arc::new(move|context,_|handler(context))])
        }
        pub fn route_many(&mut self,method:&str,path:&str,callbacks:Vec<ContextMiddleware<S>>)->Result<(),Error>{
            let method=Method::from_bytes(method.as_bytes()).map_err(|_|error("method","Invalid HTTP method"))?;
            if !matches!(method,Method::GET|Method::HEAD|Method::POST|Method::PUT|Method::PATCH|Method::DELETE|Method::OPTIONS){return Err(error("method","Unsupported HTTP method"));}
            let pattern=ContextPattern::new(path)?;
            self.layers.extend(callbacks.into_iter().map(|callback|ContextLayer{method:Some(method.clone()),pattern:pattern.clone(),callback}));Ok(())
        }
        fn finalize(mut self)->App{
            let layers=Arc::new(self.layers);let state=self.state;
            self.app.fallback(move|request|{let next=ContextNext{layers:layers.clone(),index:0};let state=state.clone();async move{
                let head=request.inner.method()==Method::HEAD;let mut context=Context::new(request,state);next.run(&mut context).await;
                let mut response=context.finish();if head{*response.inner.body_mut()=ExclusiveBody::new(Body::empty());}response
            }});
            self.app
        }
        pub async fn listen(self,address:&str)->Result<Server,Error>{self.finalize().listen(address).await}
        pub async fn listen_tls(self,address:&str,cert:&str,key:&str)->Result<Server,Error>{self.finalize().listen_tls(address,cert,key).await}
    }
    #[derive(Clone,Copy)]
    struct Limits {body:usize,connections:usize,requests:usize,request_ms:u64,shutdown_ms:u64,handshake_ms:u64,rate:u64}
    impl Default for Limits {fn default()->Self{Self{body:1048576,connections:128,requests:128,request_ms:30000,shutdown_ms:5000,handshake_ms:5000,rate:0}}}
    struct Admission {permits:Arc<tokio::sync::Semaphore>,window:std::sync::Mutex<(std::time::Instant,u64)>}
    impl Admission {
        fn admit(&self,rate:u64)->bool{
            if rate==0{return true;}
            let Ok(mut window)=self.window.lock()else{return false;};
            if window.0.elapsed()>=Duration::from_secs(1){*window=(std::time::Instant::now(),0);}
            if window.1>=rate{return false;}window.1+=1;true
        }
    }
    pub struct App {limits:Limits,routes:Vec<(Method,String,Handler)>,identities:BTreeMap<String,String>,methods:BTreeSet<(String,String)>,middleware:Vec<Middleware>,fallback:Handler}
    fn fixed(status:u16,text:&'static str)->Response {Response{inner:http::Response::builder().status(status).body(ExclusiveBody::new(Body::from(text))).unwrap()}}
    pub fn failed()->Response{fixed(500,"Internal Server Error")}
    impl App {
        pub fn new()->Self{Self{limits:Limits::default(),routes:Vec::new(),identities:BTreeMap::new(),methods:BTreeSet::new(),middleware:Vec::new(),fallback:Arc::new(|_|Box::pin(async{fixed(404,"Not Found")}))}}
        pub fn configure(&mut self,values:[i64;7])->Result<(),Error>{
            let [body,connections,requests,request_ms,shutdown_ms,handshake_ms,rate]=values;
            if !(0..=1000000).contains(&rate)||!(1..=67108864).contains(&body)||!(1..=65536).contains(&connections)||!(1..=65536).contains(&requests)||[request_ms,shutdown_ms,handshake_ms].iter().any(|value|!(1..=3600000).contains(value)){return Err(error("options","Invalid HTTP server limits"));}
            self.limits=Limits{body:body as usize,connections:connections as usize,requests:requests as usize,request_ms:request_ms as u64,shutdown_ms:shutdown_ms as u64,handshake_ms:handshake_ms as u64,rate:rate as u64};Ok(())
        }
        pub fn route<F,Fut>(&mut self,method:&str,path:&str,handler:F)->Result<(),Error> where F:Fn(Request)->Fut+Send+Sync+'static,Fut:Future<Output=Response>+Send+'static {
            let method=Method::from_bytes(method.as_bytes()).map_err(|_|error("method","Invalid HTTP method"))?;
            if !matches!(method,Method::GET|Method::HEAD|Method::POST|Method::PUT|Method::PATCH|Method::DELETE|Method::OPTIONS){return Err(error("method","Unsupported HTTP method"));}
            let (path,identity)=route_path(path)?;
            if self.identities.get(&identity).is_some_and(|old|old!=&path){return Err(error("route","Equivalent routes must use the same parameter names"));}
            let methods=if method==Method::GET{vec!["GET","HEAD"]}else{vec![method.as_str()]};
            if methods.iter().any(|method|self.methods.contains(&(identity.clone(),(*method).into()))){return Err(error("route","Duplicate method and path"));}
            self.identities.insert(identity.clone(),path.clone());
            for method in methods {self.methods.insert((identity.clone(),method.into()));}
            self.routes.push((method,path,Arc::new(move|request|Box::pin(handler(request)))));Ok(())
        }
        pub fn use_middleware<F,Fut>(&mut self,handler:F) where F:Fn(Request,Next)->Fut+Send+Sync+'static,Fut:Future<Output=Response>+Send+'static {self.middleware.push(Arc::new(move|request,next|Box::pin(handler(request,next))));}
        pub fn fallback<F,Fut>(&mut self,handler:F) where F:Fn(Request)->Fut+Send+Sync+'static,Fut:Future<Output=Response>+Send+'static {self.fallback=Arc::new(move|request|Box::pin(handler(request)));}
        pub async fn listen(self,address:&str)->Result<Server,Error> {self.listen_with_tls(address,None).await}
        pub async fn listen_tls(self,address:&str,cert:&str,key:&str)->Result<Server,Error> {
            let certificates=certificates(cert)?;
            let key=PrivateKeyDer::from_pem_slice(key.as_bytes()).map_err(|_|error("tls","Invalid PEM private key"))?;
            // SQL can also enable ring; choose HTTP crypto without changing process defaults.
            let mut config=rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider())).with_safe_default_protocol_versions().map_err(|_|error("tls","Invalid TLS protocol configuration"))?.with_no_client_auth().with_single_cert(certificates,key).map_err(|_|error("tls","Invalid certificate chain or mismatched private key"))?;
            config.alpn_protocols=vec![b"h2".to_vec(),b"http/1.1".to_vec()];
            self.listen_with_tls(address,Some(TlsAcceptor::from(Arc::new(config)))).await
        }
        async fn listen_with_tls(self,address:&str,tls:Option<TlsAcceptor>)->Result<Server,Error> {
            let address:std::net::SocketAddr=address.parse().map_err(|_|error("address","Use an IP address and port"))?;
            let listener=tokio::net::TcpListener::bind(address).await.map_err(|_|error("listen","Cannot bind HTTP listener"))?;
            let address=listener.local_addr().map_err(|_|error("listen","Cannot read listener address"))?.to_string();
            let limits=self.limits;let permits=Arc::new(Admission{permits:Arc::new(tokio::sync::Semaphore::new(limits.requests)),window:std::sync::Mutex::new((std::time::Instant::now(),0))});
            let scope=tasks::Scope::group();let middleware=Arc::new(self.middleware);let mut router=axum::Router::new();
            for (method,path,handler) in self.routes {
                let middleware=middleware.clone();let scope=scope.clone();let permits=permits.clone();
                let filter=axum::routing::MethodFilter::try_from(method).map_err(|_|error("method","Unsupported HTTP method"))?;
                router=router.route(&path,axum::routing::on(filter,move|request|dispatch(request,handler.clone(),middleware.clone(),scope.clone(),limits,permits.clone())));
            }
            let missing_permits=permits.clone();let fallback=self.fallback;let missing_middleware=middleware.clone();let missing_scope=scope.clone();
            router=router.fallback(move|request|dispatch(request,fallback.clone(),missing_middleware.clone(),missing_scope.clone(),limits,missing_permits.clone()));
            let method_scope=scope.clone();
            router=router.method_not_allowed_fallback(move|request|dispatch(request,Arc::new(|_|Box::pin(async{fixed(405,"Method Not Allowed")})),middleware.clone(),method_scope.clone(),limits,permits.clone()));
            let (stop,stopped)=tokio::sync::oneshot::channel();let stop=Arc::new(std::sync::Mutex::new(Some(stop)));let cancel=stop.clone();
            let job=tasks::start_with_shutdown(async move{run(listener,router,stopped,scope,tls,limits).await;Ok::<_,tasks::Failure<()>>(())},Some(Box::new(move||{if let Some(stop)=cancel.lock().unwrap().take(){let _=stop.send(());}})));
            Ok(Server{address,stop,job:Some(job)})
        }
    }
    fn route_path(path:&str)->Result<(String,String),Error>{
        if !path.starts_with('/') || path.contains(['?','#','%','\\','{','}']) || path.chars().any(char::is_control) || path.contains("//") || path.len()>4096{return Err(error("route","Use an absolute literal path with optional :name segments"));}
        if path=="/" {return Ok((path.into(),path.into()));}
        let mut names=BTreeSet::new();let mut output=String::new();let mut identity=String::new();
        for part in path[1..].split('/') {
            if part.is_empty()||matches!(part,"."|"..")||part.contains('*'){return Err(error("route","Empty, dot and catch-all segments are unsupported"));}
            output.push('/');identity.push('/');
            if let Some(name)=part.strip_prefix(':'){
                if name.is_empty()||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')||!names.insert(name){return Err(error("route","Parameter names must be unique alphanumeric identifiers"));}
                output.push('{');output.push_str(name);output.push('}');identity.push(':');
            }else{output.push_str(part);identity.push_str(part);}
        }Ok((output,identity))
    }
    async fn dispatch(request:http::Request<Body>,handler:Handler,middleware:Arc<Vec<Middleware>>,scope:Arc<tasks::Scope>,limits:Limits,permits:Arc<Admission>)->WireResponse {
        use axum::extract::FromRequestParts;
        if !permits.admit(limits.rate){let mut response=fixed(429,"Too Many Requests");let _=response.set_header("retry-after","1");return response.into_wire();}
        let Ok(permit)=permits.permits.clone().try_acquire_owned()else{return fixed(503,"Service Unavailable").into_wire();};
        if request.headers().get(http::header::CONTENT_LENGTH).and_then(|value|value.to_str().ok()).and_then(|value|value.parse::<u64>().ok()).is_some_and(|size|size>limits.body as u64){return fixed(413,"Payload Too Large").into_wire();}
        let (mut parts,body)=request.into_parts();
        let params=match axum::extract::RawPathParams::from_request_parts(&mut parts,&()).await {Ok(params)=>params.iter().map(|(key,value)|(key.to_owned(),value.to_owned())).collect(),Err(axum::extract::rejection::RawPathParamsRejection::MissingPathParams(_))=>BTreeMap::new(),Err(_)=>return fixed(400,"Invalid path parameters").into_wire()};
        let request=Request{inner:http::Request::from_parts(parts,ExclusiveBody::new(body)),params,limit:limits.body};
        tasks::in_scope(scope,async move {
            let job=tasks::start(async move{let _permit=permit;Ok::<_,tasks::Failure<()>>(Next{middleware,index:0,handler}.run(request).await)});
            match tokio::time::timeout(Duration::from_millis(limits.request_ms),job.join()).await{Ok(Ok(response))=>response.into_wire(),Ok(Err(_))=>failed().into_wire(),Err(_)=>fixed(504,"Gateway Timeout").into_wire()}
        }).await
    }
    pub struct Server {address:String,stop:Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,job:Option<tasks::Job<(),()>>}
    impl Server {
        pub fn address(&self)->String{self.address.clone()}
        pub async fn wait(mut self){if let Some(job)=self.job.take(){let _=job.join().await;}}
        pub async fn shutdown(mut self){self.signal();if let Some(job)=self.job.take(){let _=job.join().await;}}
        fn signal(&self){if let Some(stop)=self.stop.lock().unwrap().take(){let _=stop.send(());}}
    }
    impl Drop for Server{fn drop(&mut self){self.signal();}}
    async fn run(listener:tokio::net::TcpListener,router:axum::Router,mut stopped:tokio::sync::oneshot::Receiver<()>,scope:Arc<tasks::Scope>,tls:Option<TlsAcceptor>,limits:Limits){
        let mut connections=tokio::task::JoinSet::new();
        let (shutdown,_)=tokio::sync::watch::channel(false);
        loop {
            enum Event{Stop,Accepted(std::io::Result<(tokio::net::TcpStream,std::net::SocketAddr)>),Finished}
            let event=std::future::poll_fn(|cx|{
                if Pin::new(&mut stopped).poll(cx).is_ready(){return Poll::Ready(Event::Stop);}
                if matches!(connections.poll_join_next(cx),Poll::Ready(Some(_))){return Poll::Ready(Event::Finished);}
                if connections.len()<limits.connections{if let Poll::Ready(value)=listener.poll_accept(cx){return Poll::Ready(Event::Accepted(value));}}
                Poll::Pending
            }).await;
            match event {Event::Stop=>break,Event::Finished=>{},Event::Accepted(Err(_))=>break,Event::Accepted(Ok((stream,_)))=>{
                let router=router.clone();let tls=tls.clone();let mut shutdown=shutdown.subscribe();
                connections.spawn(async move{
                    if let Some(tls)=tls {
                        let handshake=tokio::time::timeout(Duration::from_millis(limits.handshake_ms),tls.accept(stream));tokio::pin!(handshake);
                        let accepted={let changed=shutdown.changed();tokio::pin!(changed);std::future::poll_fn(|cx|{if changed.as_mut().poll(cx).is_ready(){return Poll::Ready(None);}handshake.as_mut().poll(cx).map(Some)}).await};
                        if let Some(Ok(Ok(stream)))=accepted{serve_connection(stream,router,shutdown).await;}
                    }else{serve_connection(stream,router,shutdown).await;}
                });
            }}
        }
        drop(listener);
        let _=shutdown.send(true);
        let _=tokio::time::timeout(Duration::from_millis(limits.shutdown_ms),async{while connections.join_next().await.is_some(){}}).await;
        // Bound graceful shutdown, including clients stalled while sending or receiving.
        connections.abort_all();while connections.join_next().await.is_some(){}
        scope.drain(true).await;
    }
    fn certificates(pem:&str)->Result<Vec<CertificateDer<'static>>,Error>{
        let certificates=CertificateDer::pem_slice_iter(pem.as_bytes()).collect::<Result<Vec<_>,_>>().map_err(|_|error("tls","Invalid PEM certificate chain"))?;
        if certificates.is_empty(){return Err(error("tls","Certificate chain cannot be empty"));}Ok(certificates)
    }
    async fn serve_connection<I>(stream:I,router:axum::Router,mut shutdown:tokio::sync::watch::Receiver<bool>) where I:tokio::io::AsyncRead+tokio::io::AsyncWrite+Unpin+Send+'static {
        let service=hyper_util::service::TowerToHyperService::new(router);
        let io=hyper_util::rt::TokioIo::new(stream);let builder=hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new());let connection=builder.serve_connection(io,service);tokio::pin!(connection);
        let changed=shutdown.changed();tokio::pin!(changed);let mut draining=false;
        let _=std::future::poll_fn(|cx|{if !draining && changed.as_mut().poll(cx).is_ready(){connection.as_mut().graceful_shutdown();draining=true;}connection.as_mut().poll(cx)}).await;
    }
    pub struct FormField {pub name:String,pub value:String}
    pub struct MultipartPart {pub name:String,pub file_name:Option<String>,pub content_type:Option<String>,pub data:Vec<u8>}
    fn form_component(text:&str)->Result<String,Error>{decode_component(text,true)}
    fn decode_component(text:&str,plus:bool)->Result<String,Error>{
        let mut bytes=Vec::with_capacity(text.len());let mut input=text.bytes();
        while let Some(byte)=input.next(){match byte{b'+' if plus=>bytes.push(b' '),b'%'=>{
            let hex=|value:u8|(value as char).to_digit(16).map(|value|value as u8);
            let high=input.next().and_then(hex).ok_or_else(||error("form","Malformed percent escape"))?;
            let low=input.next().and_then(hex).ok_or_else(||error("form","Malformed percent escape"))?;bytes.push(high*16+low);
        },byte=>bytes.push(byte)}}
        String::from_utf8(bytes).map_err(|_|error("encoding","Form field is not UTF-8"))
    }
    impl Request {
        pub async fn form(self)->Result<Vec<FormField>,Error>{
            if self.header("content-type")?.as_deref().is_none_or(|value|!value.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/x-www-form-urlencoded")){return Err(error("content-type","Expected application/x-www-form-urlencoded"));}
            let text=self.text().await?;let mut fields=Vec::new();
            for part in text.split('&').filter(|part|!part.is_empty()){
                if fields.len()==128{return Err(error("limit","Form exceeds 128 fields"));}
                let (name,value)=part.split_once('=').unwrap_or((part,""));fields.push(FormField{name:form_component(name)?,value:form_component(value)?});
            }Ok(fields)
        }
        pub async fn multipart(self)->Result<Vec<MultipartPart>,Error>{
            use axum::extract::FromRequest;
            let limit=self.limit;let (parts,body)=self.inner.into_parts();
            let mut request=http::Request::from_parts(parts,Body::new(Stream{body:body.into_body(),remaining:limit}));
            request.extensions_mut().insert(axum::extract::DefaultBodyLimit::disable());
            let mut multipart=axum::extract::Multipart::from_request(request,&()).await.map_err(|_|error("multipart","Invalid multipart content type or boundary"))?;
            let mut output=Vec::new();
            while let Some(field)=multipart.next_field().await.map_err(|_|error("multipart","Malformed or oversized multipart body"))?{
                if output.len()==128{return Err(error("limit","Multipart exceeds 128 parts"));}
                let name=field.name().ok_or_else(||error("multipart","Multipart part requires a name"))?.to_owned();
                let file_name=field.file_name().map(str::to_owned);let content_type=field.content_type().map(str::to_owned);
                let data=field.bytes().await.map_err(|_|error("multipart","Malformed or oversized multipart body"))?.to_vec();
                output.push(MultipartPart{name,file_name,content_type,data});
            }Ok(output)
        }

        pub fn new(method:&str,url:&str,body:String)->Result<Self,Error>{let method=Method::from_bytes(method.as_bytes()).map_err(|_|error("method","Invalid HTTP method"))?;let inner=http::Request::builder().method(method).uri(url).body(ExclusiveBody::new(Body::from(body))).map_err(|_|error("url","Invalid request URL"))?;Ok(Self{inner,params:BTreeMap::new(),limit:1048576})}
        pub fn method(&self)->String{self.inner.method().to_string()}
        pub fn url(&self)->String{self.inner.uri().to_string()}
        pub fn param(&self,name:&str)->Option<String>{self.params.get(name).cloned()}
        pub fn header(&self,name:&str)->Result<Option<String>,Error>{header(self.inner.headers(),name)}
        pub fn set_header(&mut self,name:&str,value:&str)->Result<(),Error>{set_header(self.inner.headers_mut(),name,value)}
        pub fn body(self)->Stream{Stream{body:self.inner.into_body().into_body(),remaining:self.limit}}
        pub async fn text(self)->Result<String,Error>{self.body().text().await}
    }
    impl Response {
        fn into_wire(self)->WireResponse{self.inner.map(ExclusiveBody::into_body)}
        pub fn text(text:String,status:i64)->Result<Self,Error>{Self::content(text,status,"text/plain; charset=utf-8")}
        pub fn content(text:String,status:i64,content_type:&str)->Result<Self,Error>{let status=status_code(status)?;Ok(Self{inner:http::Response::builder().status(status).header("content-type",content_type).body(ExclusiveBody::new(Body::from(text))).map_err(|_|error("header","Invalid content type"))?})}
        pub fn stream(body:Stream,status:i64)->Result<Self,Error>{Ok(Self{inner:http::Response::builder().status(status_code(status)?).body(ExclusiveBody::new(Body::new(body))).unwrap()})}
        pub fn status(&self)->i64{self.inner.status().as_u16() as i64}
        pub fn header(&self,name:&str)->Result<Option<String>,Error>{header(self.inner.headers(),name)}
        pub fn set_header(&mut self,name:&str,value:&str)->Result<(),Error>{set_header(self.inner.headers_mut(),name,value)}
        pub fn body(self)->Stream{Stream{body:self.inner.into_body().into_body(),remaining:1048576}}
        pub async fn text_body(self)->Result<String,Error>{self.body().text().await}
    }
    fn status_code(status:i64)->Result<StatusCode,Error>{if !(200..=599).contains(&status){return Err(error("status","Response status must be 200 through 599"));}StatusCode::from_u16(status as u16).map_err(|_|error("status","Invalid response status"))}
    fn header(headers:&http::HeaderMap,name:&str)->Result<Option<String>,Error>{let name=HeaderName::try_from(name).map_err(|_|error("header","Invalid header name"))?;headers.get(name).map(|value|value.to_str().map(str::to_owned).map_err(|_|error("header","Header is not text"))).transpose()}
    fn set_header(headers:&mut http::HeaderMap,name:&str,value:&str)->Result<(),Error>{let name=HeaderName::try_from(name).map_err(|_|error("header","Invalid header name"))?;if matches!(name.as_str(),"content-length"|"transfer-encoding"|"connection"|"upgrade"){return Err(error("header","Transport framing headers are managed by HTTP"));}let value=HeaderValue::try_from(value).map_err(|_|error("header","Invalid header value"))?;headers.insert(name,value);Ok(())}
    impl axum::body::HttpBody for Stream {
        type Data=axum::body::Bytes;type Error=Error;
        fn poll_frame(mut self:Pin<&mut Self>,cx:&mut std::task::Context<'_>)->Poll<Option<Result<http_body::Frame<Self::Data>,Error>>>{
            match Pin::new(&mut self.body).poll_frame(cx){
                Poll::Ready(Some(Ok(frame)))=>{if let Some(data)=frame.data_ref(){if data.len()>self.remaining{self.body=Body::empty();return Poll::Ready(Some(Err(error("limit","Body exceeds configured byte limit"))));}self.remaining-=data.len();}Poll::Ready(Some(Ok(frame)))},
                Poll::Ready(Some(Err(_)))=>Poll::Ready(Some(Err(error("body","Body read failed")))),other=>other.map(|value|value.map(|value|value.map_err(|_|error("body","Body read failed")))),
            }
        }
    }
    impl Stream {
        pub async fn next(&mut self)->Result<Option<Vec<u8>>,Error>{
            while let Some(frame)=self.body.frame().await {let frame=frame.map_err(|_|error("body","Body read failed"))?;if let Ok(data)=frame.into_data(){if data.len()>self.remaining{self.body=Body::empty();return Err(error("limit","Body exceeds configured byte limit"));}self.remaining-=data.len();return Ok(Some(data.to_vec()));}}Ok(None)
        }
        pub async fn text(mut self)->Result<String,Error>{let mut bytes=Vec::new();while let Some(frame)=self.frame().await{if let Ok(chunk)=frame?.into_data(){bytes.extend_from_slice(&chunk);}}String::from_utf8(bytes).map_err(|_|error("encoding","Body is not UTF-8"))}
    }
    impl Client {
        pub fn new()->Result<Self,Error>{Self::build(None)}
        pub fn with_ca(ca:&str)->Result<Self,Error>{
            let mut validated=rustls::RootCertStore::empty();
            let roots=certificates(ca)?.into_iter().map(|cert|{validated.add(cert.clone()).map_err(|_|error("tls","Invalid CA certificate"))?;reqwest::Certificate::from_der(cert.as_ref()).map_err(|_|error("tls","Invalid CA certificate"))}).collect::<Result<Vec<_>,_>>()?;
            Self::build(Some(roots))
        }
        fn build(roots:Option<Vec<reqwest::Certificate>>)->Result<Self,Error>{
            let mut builder=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).no_proxy().timeout(Duration::from_secs(30));
            if let Some(roots)=roots{builder=builder.tls_certs_only(roots);}
            builder.build().map(Self).map_err(|_|error("client","Cannot create HTTP client"))
        }
        pub async fn send(&self,request:Request)->Result<Response,Error>{
            let (parts,body)=request.inner.into_parts();let url=reqwest::Url::parse(&parts.uri.to_string()).map_err(|_|error("url","Use an absolute HTTP or HTTPS URL"))?;
            if !matches!(url.scheme(),"http"|"https")||!url.username().is_empty()||url.password().is_some(){return Err(error("url","HTTP URLs cannot contain credentials"));}
            let result=self.0.request(parts.method,url).headers(parts.headers).body(reqwest::Body::wrap_stream(body.into_body().into_data_stream())).send().await.map_err(|e|error(if e.is_timeout(){"timeout"}else{"request"},"HTTP request failed"))?;
            let mut response=http::Response::builder().status(result.status());*response.headers_mut().unwrap()=result.headers().clone();Ok(Response{inner:response.body(ExclusiveBody::new(Body::from_stream(result.bytes_stream()))).unwrap()})
        }
    }
    #[derive(Default)]
    pub struct CookieOptions {pub path:Option<String>,pub domain:Option<String>,pub max_age:Option<i64>,pub secure:Option<bool>,pub http_only:Option<bool>,pub same_site:Option<String>,pub partitioned:Option<bool>}
    fn cookie_name(name:&str)->bool{!name.is_empty()&&name.bytes().all(|byte|byte.is_ascii_alphanumeric()||b"!#$%&'*+-.^_`|~".contains(&byte))}
    fn cookie_value(value:&str)->bool{value.bytes().all(|byte|matches!(byte,0x21|0x23..=0x2b|0x2d..=0x3a|0x3c..=0x5b|0x5d..=0x7e))}
    impl<S:Send+Sync+'static> Context<S> {
        pub fn cookie(&self,name:&str)->Option<String>{
            if !cookie_name(name){return None;}
            for header in self.field0.inner.headers().get_all(http::header::COOKIE){
                let Ok(header)=header.to_str()else{continue;};
                for pair in header.split(';'){
                    let Some((key,value))=pair.split_once('=')else{continue;};let key=key.trim();let mut value=value.trim();
                    if key!=name||!cookie_name(key){continue;}
                    if value.len()>=2&&value.starts_with('"')&&value.ends_with('"'){value=&value[1..value.len()-1];}
                    if !cookie_value(value){continue;}
                    return Some(decode_component(value,false).unwrap_or_else(|_|value.to_owned()));
                }
            }None
        }
        pub fn set_cookie(&mut self,name:&str,value:&str,options:CookieOptions)->Result<(),Error>{
            if !cookie_name(name){return Err(error("cookie","Invalid cookie name"));}
            let path=options.path.as_deref().unwrap_or("/");let secure=options.secure.unwrap_or(false);let http_only=options.http_only.unwrap_or(false);
            if !path.starts_with('/')||path.bytes().any(|byte|!(0x20..=0x7e).contains(&byte)||byte==b';'){return Err(error("cookie","Invalid cookie path"));}
            if options.domain.as_ref().is_some_and(|domain|domain.is_empty()||!domain.bytes().all(|byte|byte.is_ascii_alphanumeric()||matches!(byte,b'.'|b'-'))){return Err(error("cookie","Invalid cookie domain"));}
            if options.max_age.is_some_and(|age|!(0..=34560000).contains(&age)){return Err(error("cookie","Cookie maxAge must be between 0 and 34560000 seconds"));}
            if name.starts_with("__Secure-")&&!secure || name.starts_with("__Host-")&&(!secure||path!="/"||options.domain.is_some()){return Err(error("cookie","Cookie prefix requires Secure and host-only Path=/ where applicable"));}
            let same_site=match options.same_site.as_deref(){None=>None,Some("Strict"|"strict")=>Some("Strict"),Some("Lax"|"lax")=>Some("Lax"),Some("None"|"none")=>Some("None"),_=>return Err(error("cookie","sameSite must be Strict, Lax or None"))};
            if options.partitioned.unwrap_or(false)&&!secure{return Err(error("cookie","Partitioned cookies require Secure"));}
            let mut encoded=String::new();for byte in value.bytes(){if byte.is_ascii_alphanumeric()||b"-_.!~*'()".contains(&byte){encoded.push(char::from(byte));}else{use std::fmt::Write;write!(encoded,"%{byte:02X}").unwrap();}}
            let mut text=format!("{name}={encoded}; Path={path}");
            if let Some(domain)=options.domain{text.push_str("; Domain=");text.push_str(&domain);}
            if let Some(age)=options.max_age{text.push_str("; Max-Age=");text.push_str(&age.to_string());}
            if secure{text.push_str("; Secure");}if http_only{text.push_str("; HttpOnly");}
            if let Some(site)=same_site{text.push_str("; SameSite=");text.push_str(site);}
            if options.partitioned.unwrap_or(false){text.push_str("; Partitioned");}
            let header=HeaderValue::from_str(&text).map_err(|_|error("cookie","Invalid cookie header"))?;
            self.headers.append(http::header::SET_COOKIE,header.clone());self.field2.inner.headers_mut().append(http::header::SET_COOKIE,header);Ok(())
        }
        pub fn delete_cookie(&mut self,name:&str,mut options:CookieOptions)->Result<(),Error>{options.max_age=Some(0);self.set_cookie(name,"",options)}
    }

}
