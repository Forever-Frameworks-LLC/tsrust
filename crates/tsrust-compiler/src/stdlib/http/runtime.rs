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
    enum BodySource {Buffered(axum::body::Bytes),Streaming(Body),Multipart(reqwest::multipart::Form)}
    struct ExclusiveBody(std::sync::Mutex<BodySource>);
    impl ExclusiveBody {
        fn new(body:Body)->Self{Self(std::sync::Mutex::new(BodySource::Streaming(body)))}
        fn buffered(body:String)->Self{Self(std::sync::Mutex::new(BodySource::Buffered(body.into())))}
        fn into_source(self)->BodySource{self.0.into_inner().unwrap_or_else(|poisoned|poisoned.into_inner())}
        fn into_body(self)->Body{match self.into_source(){BodySource::Buffered(bytes)=>Body::from(bytes),BodySource::Streaming(body)=>body,BodySource::Multipart(form)=>Body::from_stream(form.into_stream())}}
        fn into_request_body(self)->reqwest::Body{match self.into_source(){
            BodySource::Buffered(bytes)=>reqwest::Body::from(bytes),
            BodySource::Streaming(body)=>reqwest::Body::wrap_stream(body.into_data_stream()),
            BodySource::Multipart(form)=>reqwest::Body::wrap_stream(form.into_stream()),
        }}
    }
    #[derive(Clone)]
    struct RouteTemplate(Arc<str>);
    pub struct Request {inner:http::Request<ExclusiveBody>,params:BTreeMap<String,String>,limit:Option<usize>}
    pub struct Response {inner:http::Response<ExclusiveBody>,limit:Option<usize>}
    pub struct Stream {body:Body,remaining:Option<usize>}
    struct ChannelBody<T,F> {receiver:tokio::sync::mpsc::Receiver<T>,bytes:F}
    impl<T:Send+'static,F:Fn(T)->Vec<u8>+Send+Unpin+'static> axum::body::HttpBody for ChannelBody<T,F> {
        type Data=axum::body::Bytes;type Error=Error;
        fn poll_frame(mut self:Pin<&mut Self>,cx:&mut std::task::Context<'_>)->Poll<Option<Result<http_body::Frame<Self::Data>,Error>>>{
            self.receiver.poll_recv(cx).map(|item|item.map(|item|Ok(http_body::Frame::data((self.bytes)(item).into()))))
        }
    }
    pub struct Client {inner:reqwest::Client,body_limit:Option<usize>}
    fn bound(value:Option<i64>)->Result<Option<usize>,Error>{
        value.map(|value|usize::try_from(value).map_err(|_|error("options","HTTP bound must be nonnegative and target-representable"))).transpose()
    }
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
        fn apply_headers(&self,response:&mut Response){
            for name in self.headers.keys(){response.inner.headers_mut().remove(name);for value in self.headers.get_all(name){response.inner.headers_mut().append(name,value.clone());}}
        }
        pub fn content(&self,body:String,status:Option<i64>,content_type:&str)->Result<Response,Error>{
            let mut response=Response::content(body,status.unwrap_or(self.status.as_u16() as i64),content_type)?;
            self.apply_headers(&mut response);
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
        pub fn request_body(&mut self)->Result<Stream,Error>{Ok(self.take_request_body()?.body())}
        pub async fn request_text(&mut self)->Result<String,Error>{self.take_request_body()?.text().await}
        pub async fn request_form(&mut self,max_fields:Option<i64>)->Result<Vec<FormField>,Error>{bound(max_fields)?;self.take_request_body()?.form(max_fields).await}
        pub async fn request_multipart(&mut self,max_parts:Option<i64>)->Result<Vec<MultipartPart>,Error>{bound(max_parts)?;self.take_request_body()?.multipart(max_parts).await}
        fn finish(self)->Response{if self.finalized{self.field2}else{failed()}}
    }
    #[derive(Clone)]
    struct ContextPattern {router:Option<matchit::Router<()>>,source:Arc<str>}
    impl ContextPattern {
        fn new(path:&str)->Result<Self,Error>{
            if matches!(path,"*"|"/*"){return Ok(Self{router:None,source:Arc::from(path)});}
            let wildcard=path.ends_with("/*");let base=if wildcard{&path[..path.len()-2]}else{path};
            let (normalized,_)=route_path(base)?;let mut router=matchit::Router::new();
            router.insert(normalized.clone(),()).map_err(|_|error("route","Invalid route pattern"))?;
            if wildcard{
                router.insert(format!("{normalized}/{{*__router_tail}}"),()).map_err(|_|error("route","Invalid wildcard route"))?;
                router.insert(format!("{normalized}/"),()).map_err(|_|error("route","Invalid wildcard route"))?;
            }
            Ok(Self{router:Some(router),source:Arc::from(path)})
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
                    if layer.method.is_some(){context.field0.inner.extensions_mut().insert(RouteTemplate(layer.pattern.source.clone()));}
                    let callback=layer.callback.clone();
                    if let Some(mut response)=callback(context,self).await{context.apply_headers(&mut response);context.field2=response;context.finalized=true;}
                    return;
                }
                context.field2=context.text("Not Found".into(),Some(404)).unwrap();context.finalized=true;
            })
        }
    }
    // Static roots are deployment-owned, read-only public assets, never uploads.
    // Canonical paths reject symlink escapes; local writers must not race serving.
    struct StaticMount {prefix:String,root:std::path::PathBuf,fallback:Option<std::path::PathBuf>}
    fn static_segments(path:&str)->Result<std::path::PathBuf,Error>{
        let mut result=std::path::PathBuf::new();
        if path.is_empty(){return Ok(result);}
        for raw in path.split('/'){
            let part=decode_component(raw,false).map_err(|_|error("static","Invalid static path"))?;
            let stem=part.split('.').next().unwrap_or("").to_ascii_lowercase();
            let device=matches!(stem.as_str(),"con"|"prn"|"aux"|"nul") || (stem.starts_with("com")||stem.starts_with("lpt")) && stem.len()==4 && matches!(stem.as_bytes()[3],b'1'..=b'9');
            if part.is_empty()||part.starts_with('.')||part.ends_with(['.',' '])||device||part.chars().any(|c|c.is_control()||"/\\:%?\"<>|*#".contains(c)){return Err(error("static","Invalid static path"));}
            result.push(part);
        }Ok(result)
    }
    impl StaticMount {
        fn new(prefix:&str,directory:&str,fallback:&str)->Result<Self,Error>{
            if !prefix.starts_with('/')||prefix.ends_with('/')||prefix[1..].split('/').any(|part|part.is_empty()||!part.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_')){return Err(error("static","Mount prefix requires nonempty literal URL segments without a trailing slash"));}
            let root=std::fs::canonicalize(directory).map_err(|_|error("static","Static root must be an existing readable directory"))?;
            if !root.is_dir(){return Err(error("static","Static root must be a directory"));}
            let fallback=if fallback.is_empty(){None}else{
                let relative=static_segments(fallback)?;
                if relative.extension().and_then(|v|v.to_str())!=Some("html"){return Err(error("static","SPA fallback must be an HTML file"));}
                let file=std::fs::canonicalize(root.join(relative)).map_err(|_|error("static","SPA fallback must be an existing file within its root"))?;
                if !file.starts_with(&root)||!file.is_file(){return Err(error("static","SPA fallback must be an existing file within its root"));}Some(file)
            };Ok(Self{prefix:prefix.into(),root,fallback})
        }
        async fn resolve(&self,path:&std::path::Path)->Result<Option<std::path::PathBuf>,()>{
            match tokio::fs::canonicalize(self.root.join(path)).await{
                Ok(file)=>{
                    if !file.starts_with(&self.root){return Err(());}
                    let metadata=tokio::fs::metadata(&file).await.map_err(|_|())?;
                    if metadata.is_file(){Ok(Some(file))}
                    else if metadata.is_dir(){
                        let index=tokio::fs::canonicalize(file.join("index.html")).await;
                        match index{Ok(file) if file.starts_with(&self.root)&&tokio::fs::metadata(&file).await.map_err(|_|())?.is_file()=>Ok(Some(file)),Ok(_)=>Err(()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(_)=>Err(())}
                    }else{Err(())}
                },
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(_)=>Err(())
            }
        }
        async fn respond(&self,request:&Request)->Response{
            let path=request.inner.uri().path();
            if path==self.prefix{
                return Response{inner:http::Response::builder().status(307).header("location",format!("{}/{}",self.prefix,request.inner.uri().query().map(|q|format!("?{q}")).unwrap_or_default())).body(ExclusiveBody::new(Body::empty())).unwrap(),limit:None};
            }
            let Some(relative)=path.strip_prefix(&format!("{}/",self.prefix))else{return fixed(404,"Not Found");};
            let relative=relative.strip_suffix('/').unwrap_or(relative);
            let decoded=match static_segments(relative){Ok(path)=>path,Err(_)=>return fixed(400,"Bad Request")};
            let file=match self.resolve(&decoded).await{
                Ok(Some(file))=>file,Err(())=>return fixed(404,"Not Found"),Ok(None)=>{
                    let html=request.inner.headers().get("accept").and_then(|v|v.to_str().ok()).is_some_and(|value|value.split(',').any(|item|{
                        let mut parts=item.split(';');if parts.next().unwrap_or("").trim()!="text/html"{return false;}
                        parts.all(|part|part.trim().strip_prefix("q=").is_none_or(|q|q.parse::<f32>().is_ok_and(|q|q>0.0&&q<=1.0)))
                    }));
                    if !html||decoded.extension().is_some(){return fixed(404,"Not Found");}
                    let Some(file)=&self.fallback else{return fixed(404,"Not Found");};
                    // Recheck after deployment changes; never fall into another mount.
                    match self.resolve(file.strip_prefix(&self.root).unwrap()).await{Ok(Some(file))=>file,_=>return fixed(404,"Not Found")}
                }
            };
            let mut input=http::Request::builder().method(request.inner.method().clone()).uri(request.inner.uri().clone()).body(Body::empty()).unwrap();
            *input.headers_mut()=request.inner.headers().clone();
            match tower_http::services::ServeFile::new(file).try_call(input).await{
                Ok(response)=>Response{inner:response.map(|body|ExclusiveBody::new(Body::new(body))),limit:None},
                Err(_)=>fixed(500,"Internal Server Error")
            }
        }
    }
    pub struct Router<S=()> {app:App,layers:Vec<ContextLayer<S>>,state:Arc<S>,static_prefixes:Vec<String>}
    impl Router<()> {pub fn new()->Self{Self::with_state(())}}
    impl<S:Send+Sync+'static> Router<S> {
        pub fn with_state(state:S)->Self{Self{app:App::new(),layers:Vec::new(),state:Arc::new(state),static_prefixes:Vec::new()}}
        pub fn configure(&mut self,values:[i64;7])->Result<(),Error>{self.app.configure(values)}
        pub fn serve_static(&mut self,prefix:&str,directory:&str,fallback:&str)->Result<(),Error>{
            let mount=Arc::new(StaticMount::new(prefix,directory,fallback)?);
            if self.static_prefixes.iter().any(|other|other==prefix||other.starts_with(&format!("{prefix}/"))||prefix.starts_with(&format!("{other}/"))){return Err(error("static","Static mount prefixes must not overlap"));}
            let pattern=ContextPattern::new(&format!("{prefix}/*"))?;
            let callback:ContextMiddleware<S>=Arc::new(move|context,_|{let mount=mount.clone();Box::pin(async move{Some(mount.respond(&context.field0).await)})});
            self.layers.push(ContextLayer{method:Some(Method::GET),pattern,callback});self.static_prefixes.push(prefix.into());Ok(())
        }
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
    async fn timed<F:Future>(milliseconds:Option<u64>,future:F)->Result<F::Output,()> {
        match milliseconds{Some(value)=>tokio::time::timeout(Duration::from_millis(value),future).await.map_err(|_|()),None=>Ok(future.await)}
    }
    struct Admission {permits:Arc<tokio::sync::Semaphore>,window:Option<std::sync::Mutex<(std::time::Instant,u64)>>}
    impl Admission {
        fn admit(&self,rate:u64)->bool{
            let Some(window)=&self.window else{return true;};
            let Ok(mut window)=window.lock()else{return false;};
            if window.0.elapsed()>=Duration::from_secs(1){*window=(std::time::Instant::now(),0);}
            if window.1>=rate{return false;}window.1+=1;true
        }
    }
    pub struct App {limits:Option<Limits>,routes:Vec<(Method,String,Handler)>,identities:BTreeMap<String,String>,methods:BTreeSet<(String,String)>,middleware:Vec<Middleware>,fallback:Handler}
    fn fixed(status:u16,text:&'static str)->Response {Response{inner:http::Response::builder().status(status).body(ExclusiveBody::new(Body::from(text))).unwrap(),limit:None}}
    pub fn failed()->Response{fixed(500,"Internal Server Error")}
    impl App {
        pub fn new()->Self{Self{limits:None,routes:Vec::new(),identities:BTreeMap::new(),methods:BTreeSet::new(),middleware:Vec::new(),fallback:Arc::new(|_|Box::pin(async{fixed(404,"Not Found")}))}}
        pub fn configure(&mut self,values:[i64;7])->Result<(),Error>{
            let [body,connections,requests,request_ms,shutdown_ms,handshake_ms,rate]=values;
            let positive=|value:i64|usize::try_from(value).ok().filter(|value|*value>0);
            let (Some(body),Some(connections),Some(requests))=(positive(body),positive(connections),positive(requests))else{return Err(error("options","HTTP byte and concurrency limits must be positive and target-representable"));};
            if requests>tokio::sync::Semaphore::MAX_PERMITS || rate<0 || [request_ms,shutdown_ms,handshake_ms].iter().any(|value|*value<=0 || tokio::time::Instant::now().checked_add(Duration::from_millis(*value as u64)).is_none()){return Err(error("options","HTTP limits exceed backend ranges"));}
            self.limits=Some(Limits{body,connections,requests,request_ms:request_ms as u64,shutdown_ms:shutdown_ms as u64,handshake_ms:handshake_ms as u64,rate:rate as u64});Ok(())
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
            let limits=self.limits;let permits=limits.map(|limits|Arc::new(Admission{permits:Arc::new(tokio::sync::Semaphore::new(limits.requests)),window:(limits.rate>0).then(||std::sync::Mutex::new((std::time::Instant::now(),0)))}));
            let scope=tasks::Scope::group();let middleware=Arc::new(self.middleware);let mut router=axum::Router::new();
            for (method,path,handler) in self.routes {
                let middleware=middleware.clone();let scope=scope.clone();let permits=permits.clone();
                let filter=axum::routing::MethodFilter::try_from(method).map_err(|_|error("method","Unsupported HTTP method"))?;
                let template=RouteTemplate(Arc::from(path.as_str()));
                router=router.route(&path,axum::routing::on(filter,move|mut request: http::Request<Body>|{request.extensions_mut().insert(template.clone());dispatch(request,handler.clone(),middleware.clone(),scope.clone(),limits,permits.clone())}));
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
    async fn dispatch(request:http::Request<Body>,handler:Handler,middleware:Arc<Vec<Middleware>>,scope:Arc<tasks::Scope>,limits:Option<Limits>,permits:Option<Arc<Admission>>)->WireResponse {
        use axum::extract::FromRequestParts;
        if let (Some(limits),Some(permits))=(limits,&permits) && !permits.admit(limits.rate){let mut response=fixed(429,"Too Many Requests");let _=response.set_header("retry-after","1");return response.into_wire();}
        let permit=match &permits{Some(admission)=>match admission.permits.clone().try_acquire_owned(){Ok(permit)=>Some(permit),Err(_)=>return fixed(503,"Service Unavailable").into_wire()},None=>None};
        if let Some(limits)=limits && request.headers().get(http::header::CONTENT_LENGTH).and_then(|value|value.to_str().ok()).and_then(|value|value.parse::<u64>().ok()).is_some_and(|size|size>limits.body as u64){return fixed(413,"Payload Too Large").into_wire();}
        let (mut parts,body)=request.into_parts();
        let params=match axum::extract::RawPathParams::from_request_parts(&mut parts,&()).await {Ok(params)=>params.iter().map(|(key,value)|(key.to_owned(),value.to_owned())).collect(),Err(axum::extract::rejection::RawPathParamsRejection::MissingPathParams(_))=>BTreeMap::new(),Err(_)=>return fixed(400,"Invalid path parameters").into_wire()};
        let request=Request{inner:http::Request::from_parts(parts,ExclusiveBody::new(body)),params,limit:limits.map(|limits|limits.body)};
        tasks::in_scope(scope,async move {
            let job=tasks::start(async move{let _permit=permit;Ok::<_,tasks::Failure<()>>(Next{middleware,index:0,handler}.run(request).await)});
            match timed(limits.map(|limits|limits.request_ms),job.join()).await{Ok(Ok(response))=>response.into_wire(),Ok(Err(_))=>failed().into_wire(),Err(_)=>fixed(504,"Gateway Timeout").into_wire()}
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
    async fn run(listener:tokio::net::TcpListener,router:axum::Router,mut stopped:tokio::sync::oneshot::Receiver<()>,scope:Arc<tasks::Scope>,tls:Option<TlsAcceptor>,limits:Option<Limits>){
        let mut connections=tokio::task::JoinSet::new();
        let (shutdown,_)=tokio::sync::watch::channel(false);
        loop {
            enum Event{Stop,Accepted(std::io::Result<(tokio::net::TcpStream,std::net::SocketAddr)>),Finished}
            let event=std::future::poll_fn(|cx|{
                if Pin::new(&mut stopped).poll(cx).is_ready(){return Poll::Ready(Event::Stop);}
                if matches!(connections.poll_join_next(cx),Poll::Ready(Some(_))){return Poll::Ready(Event::Finished);}
                if limits.is_none_or(|limits|connections.len()<limits.connections){if let Poll::Ready(value)=listener.poll_accept(cx){return Poll::Ready(Event::Accepted(value));}}
                Poll::Pending
            }).await;
            match event {Event::Stop=>break,Event::Finished=>{},Event::Accepted(Err(_))=>break,Event::Accepted(Ok((stream,_)))=>{
                let router=router.clone();let tls=tls.clone();let mut shutdown=shutdown.subscribe();
                let connection_scope=scope.clone();
                connections.spawn(tasks::in_scope(connection_scope,async move{
                    if let Some(tls)=tls {
                        let handshake=timed(limits.map(|limits|limits.handshake_ms),tls.accept(stream));tokio::pin!(handshake);
                        let accepted={let changed=shutdown.changed();tokio::pin!(changed);std::future::poll_fn(|cx|{if changed.as_mut().poll(cx).is_ready(){return Poll::Ready(None);}handshake.as_mut().poll(cx).map(Some)}).await};
                        if let Some(Ok(Ok(stream)))=accepted{serve_connection(stream,router,shutdown).await;}
                    }else{serve_connection(stream,router,shutdown).await;}
                }));
            }}
        }
        drop(listener);
        let _=shutdown.send(true);
        let _=timed(limits.map(|limits|limits.shutdown_ms),async{while connections.join_next().await.is_some(){}}).await;
        // Only an explicitly configured deadline bounds graceful draining.
        connections.abort_all();while connections.join_next().await.is_some(){}
        scope.drain(true).await;
    }
    fn certificates(pem:&str)->Result<Vec<CertificateDer<'static>>,Error>{
        let certificates=CertificateDer::pem_slice_iter(pem.as_bytes()).collect::<Result<Vec<_>,_>>().map_err(|_|error("tls","Invalid PEM certificate chain"))?;
        if certificates.is_empty(){return Err(error("tls","Certificate chain cannot be empty"));}Ok(certificates)
    }
    async fn serve_connection<I>(stream:I,router:axum::Router,mut shutdown:tokio::sync::watch::Receiver<bool>) where I:tokio::io::AsyncRead+tokio::io::AsyncWrite+Unpin+Send+'static {
        let service=hyper_util::service::TowerToHyperService::new(router);
        let io=hyper_util::rt::TokioIo::new(stream);let builder=hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new());let connection=builder.serve_connection_with_upgrades(io,service);tokio::pin!(connection);
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
    // Preserve undecodable escape runs rather than replacing bytes with U+FFFD.
    fn query_component(text:&str)->String{
        let text=text.replace('+'," ");
        if let Ok(decoded)=decode_component(&text,false){return decoded;}
        let bytes=text.as_bytes();let mut output=String::new();let mut cursor=0;
        while cursor<bytes.len(){
            let start=cursor;
            while cursor+2<bytes.len() && bytes[cursor]==b'%' && bytes[cursor+1].is_ascii_hexdigit() && bytes[cursor+2].is_ascii_hexdigit(){cursor+=3;}
            if cursor>start{let run=&text[start..cursor];output.push_str(&decode_component(run,false).unwrap_or_else(|_|run.to_owned()));}
            else{let ch=text[cursor..].chars().next().unwrap();output.push(ch);cursor+=ch.len_utf8();}
        }output
    }
    impl Request {
        pub async fn form(self,max_fields:Option<i64>)->Result<Vec<FormField>,Error>{
            let max_fields=bound(max_fields)?;
            if self.header("content-type")?.as_deref().is_none_or(|value|!value.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/x-www-form-urlencoded")){return Err(error("content-type","Expected application/x-www-form-urlencoded"));}
            let text=self.text().await?;let mut fields=Vec::new();
            for part in text.split('&').filter(|part|!part.is_empty()){
                if max_fields.is_some_and(|maximum|fields.len()==maximum){return Err(error("limit","Form exceeds configured field limit"));}
                let (name,value)=part.split_once('=').unwrap_or((part,""));fields.push(FormField{name:form_component(name)?,value:form_component(value)?});
            }Ok(fields)
        }
        pub async fn multipart(self,max_parts:Option<i64>)->Result<Vec<MultipartPart>,Error>{
            let max_parts=bound(max_parts)?;
            use axum::extract::FromRequest;
            let limit=self.limit;let (parts,body)=self.inner.into_parts();
            let mut request=http::Request::from_parts(parts,Body::new(Stream{body:body.into_body(),remaining:limit}));
            if let Some(limit)=limit{axum::extract::DefaultBodyLimit::max(limit).apply(&mut request);}
            let mut multipart=axum::extract::Multipart::from_request(request,&()).await.map_err(|_|error("multipart","Invalid multipart content type or boundary"))?;
            let mut output=Vec::new();
            while let Some(field)=multipart.next_field().await.map_err(|_|error("multipart","Malformed or oversized multipart body"))?{
                if max_parts.is_some_and(|maximum|output.len()==maximum){return Err(error("limit","Multipart exceeds configured part limit"));}
                let name=field.name().ok_or_else(||error("multipart","Multipart part requires a name"))?.to_owned();
                let file_name=field.file_name().map(str::to_owned);let content_type=field.content_type().map(str::to_owned);
                let data=field.bytes().await.map_err(|_|error("multipart","Malformed or oversized multipart body"))?.to_vec();
                output.push(MultipartPart{name,file_name,content_type,data});
            }Ok(output)
        }

        fn from_body(method:&str,url:&str,body:ExclusiveBody)->Result<Self,Error>{let method=Method::from_bytes(method.as_bytes()).map_err(|_|error("method","Invalid HTTP method"))?;let inner=http::Request::builder().method(method).uri(url).body(body).map_err(|_|error("url","Invalid request URL"))?;Ok(Self{inner,params:BTreeMap::new(),limit:None})}
        pub fn new(method:&str,url:&str,body:String)->Result<Self,Error>{Self::from_body(method,url,ExclusiveBody::buffered(body))}
        pub fn from_form<'a>(method:&str,url:&str,fields:impl IntoIterator<Item=(&'a str,&'a str)>)->Result<Self,Error>{
            let mut request=Self::new(method,url,String::new())?;let mut encoded=form_urlencoded::Serializer::new(String::new());for(name,value)in fields{encoded.append_pair(name,value);}*request.inner.body_mut()=ExclusiveBody::buffered(encoded.finish());request.inner.headers_mut().insert(http::header::CONTENT_TYPE,HeaderValue::from_static("application/x-www-form-urlencoded"));Ok(request)
        }
        pub fn from_multipart(method:&str,url:&str,parts:impl IntoIterator<Item=MultipartPart>)->Result<Self,Error>{
            let mut request=Self::new(method,url,String::new())?;let mut form=reqwest::multipart::Form::new();for field in parts{let mut part=reqwest::multipart::Part::bytes(field.data);if let Some(file_name)=field.file_name{part=part.file_name(file_name);}if let Some(content_type)=field.content_type{part=part.mime_str(&content_type).map_err(|_|error("multipart","Invalid multipart part content type"))?;}form=form.part(field.name,part);}
            let content_type=HeaderValue::from_str(&format!("multipart/form-data; boundary={}",form.boundary())).map_err(|_|error("multipart","Invalid multipart boundary"))?;request.inner.headers_mut().insert(http::header::CONTENT_TYPE,content_type);*request.inner.body_mut()=ExclusiveBody(std::sync::Mutex::new(BodySource::Multipart(form)));Ok(request)
        }
        pub fn from_bytes(method:&str,url:&str,body:Vec<u8>)->Result<Self,Error>{Self::from_body(method,url,ExclusiveBody(std::sync::Mutex::new(BodySource::Buffered(body.into()))))}
        pub fn from_stream(method:&str,url:&str,body:Stream)->Result<Self,Error>{Self::from_body(method,url,ExclusiveBody::new(Body::new(body)))}
        pub fn from_channel<T:Send+'static,F:Fn(T)->Vec<u8>+Send+Unpin+'static>(method:&str,url:&str,channel:tasks::Channel<T>,bytes:F)->Result<Self,Error>{Self::from_body(method,url,ExclusiveBody::new(Body::new(ChannelBody{receiver:channel.into_receiver(),bytes})))}
        pub fn method(&self)->String{self.inner.method().to_string()}
        pub fn url(&self)->String{self.inner.uri().to_string()}
        pub fn param(&self,name:&str)->Option<String>{self.params.get(name).cloned()}
        fn query_values<'a>(&'a self,name:&'a str)->impl Iterator<Item=String>+'a{
            self.inner.uri().query().into_iter().flat_map(|query|query.split('&')).filter_map(move|part|{
                let (key,value)=part.split_once('=').unwrap_or((part,""));
                (!key.is_empty() && query_component(key)==name).then(||query_component(value))
            })
        }
        pub fn query(&self,name:&str)->Option<String>{self.query_values(name).next()}
        pub fn queries(&self,name:&str)->Option<Vec<String>>{let values:Vec<_>=self.query_values(name).collect();(!values.is_empty()).then_some(values)}
        pub fn header(&self,name:&str)->Result<Option<String>,Error>{header(self.inner.headers(),name)}
        pub fn set_header(&mut self,name:&str,value:&str)->Result<(),Error>{set_header(self.inner.headers_mut(),name,value)}
        pub fn body(self)->Stream{Stream{body:self.inner.into_body().into_body(),remaining:self.limit}}
        pub async fn text(self)->Result<String,Error>{self.body().text().await}
    }
    impl Response {
        fn into_wire(self)->WireResponse{self.inner.map(ExclusiveBody::into_body)}
        pub fn text(text:String,status:i64)->Result<Self,Error>{Self::content(text,status,"text/plain; charset=utf-8")}
        pub fn content(text:String,status:i64,content_type:&str)->Result<Self,Error>{let status=status_code(status)?;Ok(Self{inner:http::Response::builder().status(status).header("content-type",content_type).body(ExclusiveBody::new(Body::from(text))).map_err(|_|error("header","Invalid content type"))?,limit:None})}
        pub fn stream(body:Stream,status:i64)->Result<Self,Error>{Ok(Self{inner:http::Response::builder().status(status_code(status)?).body(ExclusiveBody::new(Body::new(body))).unwrap(),limit:None})}
        pub fn stream_channel<T:Send+'static,F:Fn(T)->Vec<u8>+Send+Unpin+'static>(channel:tasks::Channel<T>,status:i64,bytes:F)->Result<Self,Error>{
            let status=status_code(status)?;
            let body=ChannelBody{receiver:channel.into_receiver(),bytes};
            Ok(Self{inner:http::Response::builder().status(status).body(ExclusiveBody::new(Body::new(body))).unwrap(),limit:None})
        }
        pub fn status(&self)->i64{self.inner.status().as_u16() as i64}
        pub fn header(&self,name:&str)->Result<Option<String>,Error>{header(self.inner.headers(),name)}
        pub fn set_header(&mut self,name:&str,value:&str)->Result<(),Error>{set_header(self.inner.headers_mut(),name,value)}
        pub fn body(self)->Stream{Stream{body:self.inner.into_body().into_body(),remaining:self.limit}}
        pub async fn text_body(self)->Result<String,Error>{self.body().text().await}
    }
    fn status_code(status:i64)->Result<StatusCode,Error>{if !(200..=599).contains(&status){return Err(error("status","Response status must be 200 through 599"));}StatusCode::from_u16(status as u16).map_err(|_|error("status","Invalid response status"))}
    fn header(headers:&http::HeaderMap,name:&str)->Result<Option<String>,Error>{let name=HeaderName::try_from(name).map_err(|_|error("header","Invalid header name"))?;headers.get(name).map(|value|value.to_str().map(str::to_owned).map_err(|_|error("header","Header is not text"))).transpose()}
    fn set_header(headers:&mut http::HeaderMap,name:&str,value:&str)->Result<(),Error>{let name=HeaderName::try_from(name).map_err(|_|error("header","Invalid header name"))?;if matches!(name.as_str(),"content-length"|"transfer-encoding"|"connection"|"upgrade"){return Err(error("header","Transport framing headers are managed by HTTP"));}let value=HeaderValue::try_from(value).map_err(|_|error("header","Invalid header value"))?;headers.insert(name,value);Ok(())}
    impl axum::body::HttpBody for Stream {
        type Data=axum::body::Bytes;type Error=Error;
        fn poll_frame(mut self:Pin<&mut Self>,cx:&mut std::task::Context<'_>)->Poll<Option<Result<http_body::Frame<Self::Data>,Error>>>{
            match Pin::new(&mut self.body).poll_frame(cx){
                Poll::Ready(Some(Ok(frame)))=>{if let Some(data)=frame.data_ref(){if self.remaining.is_some_and(|remaining|data.len()>remaining){self.body=Body::empty();return Poll::Ready(Some(Err(error("limit","Body exceeds configured byte limit"))));}if let Some(remaining)=&mut self.remaining{*remaining-=data.len();}}Poll::Ready(Some(Ok(frame)))},
                Poll::Ready(Some(Err(_)))=>Poll::Ready(Some(Err(error("body","Body read failed")))),other=>other.map(|value|value.map(|value|value.map_err(|_|error("body","Body read failed")))),
            }
        }
    }
    impl Stream {
        pub fn unbounded(body:Body)->Self{Self{body,remaining:None}}
        pub async fn next(&mut self)->Result<Option<Vec<u8>>,Error>{
            while let Some(frame)=self.body.frame().await {let frame=frame.map_err(|_|error("body","Body read failed"))?;if let Ok(data)=frame.into_data(){if self.remaining.is_some_and(|remaining|data.len()>remaining){self.body=Body::empty();return Err(error("limit","Body exceeds configured byte limit"));}if let Some(remaining)=&mut self.remaining{*remaining-=data.len();}return Ok(Some(data.to_vec()));}}Ok(None)
        }
        pub async fn text(mut self)->Result<String,Error>{let mut bytes=Vec::new();while let Some(frame)=self.frame().await{if let Ok(chunk)=frame?.into_data(){bytes.extend_from_slice(&chunk);}}String::from_utf8(bytes).map_err(|_|error("encoding","Body is not UTF-8"))}
    }
    impl Client {
        pub fn new()->Result<Self,Error>{Self::build(None,None)}
        pub fn with_options(ca:Option<&str>,body_limit:Option<i64>)->Result<Self,Error>{
            let body_limit=bound(body_limit)?;
            match ca{Some(ca)=>Self::with_ca_limit(ca,body_limit),None=>Self::build(None,body_limit)}
        }
        pub fn with_ca(ca:&str)->Result<Self,Error>{Self::with_ca_limit(ca,None)}
        fn roots(ca:Option<&str>)->Result<Option<Vec<reqwest::Certificate>>,Error>{
            let Some(ca)=ca else{return Ok(None);};let mut validated=rustls::RootCertStore::empty();
            certificates(ca)?.into_iter().map(|cert|{validated.add(cert.clone()).map_err(|_|error("tls","Invalid CA certificate"))?;reqwest::Certificate::from_der(cert.as_ref()).map_err(|_|error("tls","Invalid CA certificate"))}).collect::<Result<Vec<_>,_>>().map(Some)
        }
        fn with_ca_limit(ca:&str,body_limit:Option<usize>)->Result<Self,Error>{Self::build(Self::roots(Some(ca))?,body_limit)}
        fn build(roots:Option<Vec<reqwest::Certificate>>,body_limit:Option<usize>)->Result<Self,Error>{Self::build_transport(roots,body_limit,None,None,None,None,None,None)}
        pub fn with_transport_options(ca:Option<&str>,body_limit:Option<i64>,timeout_ms:Option<i64>,connect_timeout_ms:Option<i64>,follow_redirects:Option<bool>,max_redirects:Option<i64>,proxy:Option<&str>,no_proxy:Option<bool>)->Result<Self,Error>{
            let duration=|value:Option<i64>|->Result<Option<Duration>,Error>{value.map(|value|{
                let milliseconds=u64::try_from(value).map_err(|_|error("options","HTTP timeouts must be nonnegative milliseconds"))?;let duration=Duration::from_millis(milliseconds);
                std::time::Instant::now().checked_add(duration).ok_or_else(||error("options","HTTP timeout exceeds backend timer range"))?;Ok(duration)
            }).transpose()};
            let body_limit=bound(body_limit)?;let timeout=duration(timeout_ms)?;let connect_timeout=duration(connect_timeout_ms)?;let max_redirects=bound(max_redirects)?;
            if follow_redirects==Some(false)&&max_redirects.is_some(){return Err(error("options","maxRedirects conflicts with followRedirects false"));}
            if no_proxy==Some(true)&&proxy.is_some(){return Err(error("options","proxy conflicts with noProxy true"));}
            Self::build_transport(Self::roots(ca)?,body_limit,timeout,connect_timeout,follow_redirects,max_redirects,proxy,no_proxy)
        }
        fn build_transport(roots:Option<Vec<reqwest::Certificate>>,body_limit:Option<usize>,timeout:Option<Duration>,connect_timeout:Option<Duration>,follow_redirects:Option<bool>,max_redirects:Option<usize>,proxy:Option<&str>,no_proxy:Option<bool>)->Result<Self,Error>{
            // Absent options preserve the selected backend's ordinary behavior.
            let mut builder=reqwest::Client::builder();
            if let Some(roots)=roots{builder=builder.tls_certs_only(roots);}
            if let Some(timeout)=timeout{builder=builder.timeout(timeout);}
            if let Some(timeout)=connect_timeout{builder=builder.connect_timeout(timeout);}
            if follow_redirects==Some(false){builder=builder.redirect(reqwest::redirect::Policy::none());}
            else if let Some(maximum)=max_redirects{builder=builder.redirect(reqwest::redirect::Policy::limited(maximum));}
            if let Some(proxy)=proxy{builder=builder.proxy(reqwest::Proxy::all(proxy).map_err(|_|error("options","Invalid HTTP proxy configuration"))?);}
            else if no_proxy==Some(true){builder=builder.no_proxy();}
            builder.build().map(|inner|Self{inner,body_limit}).map_err(|_|error("client","Cannot create HTTP client"))
        }
        pub async fn send(&self,request:Request)->Result<Response,Error>{
            let (parts,body)=request.inner.into_parts();let url=reqwest::Url::parse(&parts.uri.to_string()).map_err(|_|error("url","Use an absolute HTTP or HTTPS URL"))?;
            if !matches!(url.scheme(),"http"|"https")||!url.username().is_empty()||url.password().is_some(){return Err(error("url","HTTP URLs cannot contain credentials"));}
            let builder=self.inner.request(parts.method,url);let builder=match body.into_source(){BodySource::Multipart(form)=>builder.multipart(form),BodySource::Buffered(bytes)=>builder.body(reqwest::Body::from(bytes)),BodySource::Streaming(body)=>builder.body(reqwest::Body::wrap_stream(body.into_data_stream()))};
            // Merge authored headers after the backend computes multipart framing;
            // this keeps explicit overrides and avoids duplicate Content-Type values.
            let result=builder.headers(parts.headers).send().await.map_err(|e|error(if e.is_timeout(){"timeout"}else{"request"},"HTTP request failed"))?;
            let mut response=http::Response::builder().status(result.status());*response.headers_mut().unwrap()=result.headers().clone();Ok(Response{inner:response.body(ExclusiveBody::new(Body::from_stream(result.bytes_stream()))).unwrap(),limit:self.body_limit})
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

    #[cfg(test)]
    mod client_default_tests {
        use super::*;
        #[test]
        fn default_public_tls_client_and_server_negotiate_http2(){
            tasks::block_on(async{
                let ca=include_str!("../../../tests/fixtures/tls/ca.pem");
                let cert=include_str!("../../../tests/fixtures/tls/server.pem");
                let key=include_str!("../../../tests/fixtures/tls/server.key");
                let mut app=App::new();
                app.route("GET","/version",|request|async move{
                    // Observe the protocol decoded by Axum, independently of
                    // the public Response wrapper, which exposes no version.
                    Response::text(format!("{:?}",request.inner.version()),200).unwrap()
                }).unwrap();
                let server=app.listen_tls("127.0.0.1:0",cert,key).await.unwrap();
                let address=server.address();let url=format!("https://{address}/version");
                let observed=tokio::time::timeout(Duration::from_secs(5),async{
                    let mut roots=rustls::RootCertStore::empty();
                    roots.add(CertificateDer::from_pem_slice(ca.as_bytes())?)?;
                    let mut config=rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
                        .with_safe_default_protocol_versions()?.with_root_certificates(roots).with_no_client_auth();
                    config.alpn_protocols=vec![b"h2".to_vec(),b"http/1.1".to_vec()];
                    let socket=tokio::net::TcpStream::connect(&address).await?;
                    let stream=tokio_rustls::TlsConnector::from(Arc::new(config)).connect(rustls::pki_types::ServerName::try_from("127.0.0.1")?,socket).await?;
                    let alpn=stream.get_ref().1.alpn_protocol().map(<[u8]>::to_vec);drop(stream);
                    // Only trust roots and loopback proxy isolation are explicit;
                    // neither client forces HTTP/1 nor HTTP/2 prior knowledge.
                    let client=Client::with_transport_options(Some(ca),None,None,None,None,None,None,Some(true))?;
                    let response=client.send(Request::new("GET",&url,String::new())?).await?;
                    let public_status=response.status();let public_version=response.text_body().await?;drop(client);
                    let direct=reqwest::Client::builder().no_proxy().tls_certs_only([reqwest::Certificate::from_pem(ca.as_bytes())?]).build()?;
                    let response=direct.get(&url).send().await?;let version=response.version();let status=response.status();let body=response.text().await?;drop(direct);
                    Ok::<_,Box<dyn std::error::Error>>((alpn,public_status,public_version,version,status,body))
                }).await;
                // Settle the public server before reporting any protocol or
                // request failure; a failed assertion must not strand it.
                server.shutdown().await;
                let(alpn,public_status,public_version,version,status,body)=observed.expect("TLS negotiation/request witness timed out").unwrap();
                assert_eq!(alpn.as_deref(),Some(b"h2".as_slice()));
                assert_eq!(public_status,200);assert_eq!(public_version,"HTTP/2.0");
                assert_eq!(version,reqwest::Version::HTTP_2);assert_eq!(status,reqwest::StatusCode::OK);assert_eq!(body,"HTTP/2.0");
            });
        }
        #[test]
        fn explicit_client_policy_matches_direct_backend_settings(){
            let actual=Client::with_transport_options(None,None,Some(1234),Some(456),Some(false),None,None,Some(true)).unwrap();
            let expected=reqwest::Client::builder().timeout(Duration::from_millis(1234)).connect_timeout(Duration::from_millis(456)).redirect(reqwest::redirect::Policy::none()).no_proxy().build().unwrap();
            assert_eq!(format!("{:?}",actual.inner),format!("{expected:?}"));
            let value=vec![1u8,0,255];let pointer=value.as_ptr();let request=Request::from_bytes("POST","http://localhost/",value).unwrap();let body=request.inner.into_body().into_request_body();assert_eq!(body.as_bytes().unwrap().as_ptr(),pointer);
        }
        #[test]
        fn client_buffered_bodies_keep_the_owned_allocation_and_streams_remain_streaming() {
            let text=String::from("owned UTF-8 é payload");let pointer=text.as_ptr();let length=text.len();
            let body=ExclusiveBody::buffered(text).into_request_body();let bytes=body.as_bytes().expect("buffered request must be replayable");
            assert_eq!(bytes.as_ptr(),pointer);assert_eq!(bytes.len(),length);assert_eq!(bytes,b"owned UTF-8 \xc3\xa9 payload");
            let stream=ExclusiveBody::new(Body::from("stream payload")).into_request_body();
            assert!(stream.as_bytes().is_none(),"stream provenance must not be collected or reclassified");
        }
        #[test]
        fn server_policies_are_opt_in_and_backend_ranges_are_checked_atomically() {
            let mut app=App::new();assert!(app.limits.is_none());
            app.configure([67108865,65537,65537,3600001,3600001,3600001,1000001]).unwrap();
            let configured=app.limits.unwrap();assert_eq!(configured.body,67108865);assert_eq!(configured.connections,65537);assert_eq!(configured.requests,65537);assert_eq!(configured.rate,1000001);assert_eq!(configured.request_ms,3600001);
            let too_many=i64::try_from(tokio::sync::Semaphore::MAX_PERMITS+1).unwrap();
            for values in [[0,1,1,1,1,1,0],[1,0,1,1,1,1,0],[1,1,too_many,1,1,1,0],[1,1,1,0,1,1,0],[1,1,1,1,-1,1,0],[1,1,1,1,1,1,-1]]{
                assert_eq!(app.configure(values).err().unwrap().code,"options");assert_eq!(app.limits.unwrap().body,configured.body);
            }
            #[cfg(target_pointer_width="32")]
            assert!(app.configure([i64::from(u32::MAX)+1,1,1,1,1,1,0]).is_err());
        }
        #[test]
        fn multipart_preserves_backend_default_without_an_application_byte_budget() {
            tasks::block_on(async{
                for(size,limit,accepted)in[(1048577,None,true),(2097153,None,false),(2097153,Some(3145728),true)]{
                    let body=format!("--x\r\nContent-Disposition: form-data; name=\"data\"\r\n\r\n{}\r\n--x--\r\n","x".repeat(size));
                    let mut request=Request::new("POST","http://localhost/",body).unwrap();request.limit=limit;
                    request.set_header("content-type","multipart/form-data; boundary=x").unwrap();
                    let result=request.multipart(None).await;
                    if accepted{let parts=result.unwrap();assert_eq!(parts.len(),1);assert_eq!(parts[0].data.len(),size);assert!(parts[0].data.iter().all(|byte|*byte==b'x'));}
                    else{assert_eq!(result.err().unwrap().code,"multipart");}
                }
            });
        }
        #[test]
        fn client_settings_match_the_selected_backend_without_sleep_or_environment_mutation() {
            let backend=reqwest::Client::builder().build().unwrap();
            let actual=Client::new().unwrap();
            // Reqwest's Debug reports configured timeouts and nondefault redirect/proxy
            // settings. Compare the same backend and environment, rather than waiting 30s.
            let actual=format!("{:?}",actual.inner);assert_eq!(actual,format!("{backend:?}"));
            // Prove these backend fields are observable in this pinned Debug format.
            for changed in [
                reqwest::Client::builder().timeout(Duration::from_secs(30)).build().unwrap(),
                reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap(),
                reqwest::Client::builder().no_proxy().build().unwrap(),
            ]{assert_ne!(actual,format!("{changed:?}"));}
        }
        #[test]
        fn response_budget_is_cumulative_across_separate_frames() {
            tasks::block_on(async{
                let channel=tasks::Channel::new(2).unwrap();let sender=channel.sender();
                assert!(sender.try_send(vec![1u8,2]).is_ok());assert!(sender.try_send(vec![3u8,4]).is_ok());drop(sender);
                let mut response=Response::stream_channel(channel,200,|value|value).unwrap();response.limit=Some(3);
                let mut body=response.body();assert_eq!(body.next().await.unwrap(),Some(vec![1,2]));
                assert_eq!(body.next().await.err().unwrap().code,"limit");assert!(body.next().await.unwrap().is_none());
            });
        }
    }
    // Explicit source-installed policies; ordinary HTTP does not select this feature.
    #[cfg(any(test, feature = "http-middleware"))]
    pub use builtin_middleware::NativeMiddleware;
    #[cfg(any(test, feature = "http-middleware"))]
    mod builtin_middleware {
        use super::*;
        use std::convert::Infallible;
        use tower::{Layer,Service,ServiceBuilder,ServiceExt};
        pub enum NativeMiddleware {Cors(tower_http::cors::CorsLayer),Secure,Compress(tower_http::compression::CompressionLayer),
            #[cfg(feature="logging")] AccessLog(super::super::tsrust_native_logging::Logger),
        }
        impl NativeMiddleware {
            #[cfg(feature="logging")]
            pub fn access_log(logger:&super::super::tsrust_native_logging::Logger)->Self{Self::AccessLog(logger.share())}
            pub fn cors_default()->Self{Self::Cors(tower_http::cors::CorsLayer::new())}
            pub fn cors<'a,O,M,A,E>(origins:Option<O>,methods:Option<M>,allowed:Option<A>,exposed:Option<E>,credentials:Option<bool>,max_age:Option<i64>)->Result<Self,Error>
            where O:IntoIterator<Item=&'a str>,M:IntoIterator<Item=&'a str>,A:IntoIterator<Item=&'a str>,E:IntoIterator<Item=&'a str>{
                fn list<'a,I,T>(values:Option<I>,mut parse:impl FnMut(&str)->Result<T,Error>)->Result<(bool,Vec<T>),Error> where I:IntoIterator<Item=&'a str>{let mut wildcard=false;let mut count=0;let mut result=Vec::new();for value in values.into_iter().flatten(){count+=1;if value=="*"{wildcard=true;}else{result.push(parse(value)?);}}if wildcard&&count!=1{return Err(error("options","CORS wildcard must be the only list entry"));}Ok((wildcard,result))}
                let origins=list(origins,|v|HeaderValue::from_str(v).map_err(|_|error("options","Invalid CORS origin header value")))?;
                let methods=list(methods,|v|Method::from_bytes(v.as_bytes()).map_err(|_|error("options","Invalid CORS HTTP method")))?;
                let allowed=list(allowed,|v|HeaderName::from_bytes(v.as_bytes()).map_err(|_|error("options","Invalid CORS allowed header name")))?;
                let exposed=list(exposed,|v|HeaderName::from_bytes(v.as_bytes()).map_err(|_|error("options","Invalid CORS exposed header name")))?;
                let credentials=credentials.unwrap_or(false);if credentials&&(origins.0||methods.0||allowed.0||exposed.0){return Err(error("options","Credentialed CORS cannot use wildcard origins, methods or headers"));}
                let mut layer=tower_http::cors::CorsLayer::new().allow_credentials(credentials);
                layer=if origins.0{layer.allow_origin(tower_http::cors::Any)}else{layer.allow_origin(origins.1)};
                layer=if methods.0{layer.allow_methods(tower_http::cors::Any)}else{layer.allow_methods(methods.1)};
                layer=if allowed.0{layer.allow_headers(tower_http::cors::Any)}else{layer.allow_headers(allowed.1)};
                layer=if exposed.0{layer.expose_headers(tower_http::cors::Any)}else{layer.expose_headers(exposed.1)};
                if let Some(value)=max_age{let value=u64::try_from(value).map_err(|_|error("options","CORS maxAgeSeconds must be nonnegative"))?;layer=layer.max_age(Duration::from_secs(value));}Ok(Self::Cors(layer))
            }
            pub fn secure_headers()->Self{Self::Secure}
            pub fn compress()->Self{Self::Compress(tower_http::compression::CompressionLayer::new())}
            async fn apply<S>(&self,request:http::Request<Body>,service:S)->WireResponse where S:Service<http::Request<Body>,Response=WireResponse,Error=Infallible>+Send,S::Future:Send{
                match self {
                    Self::Cors(layer)=>layer.layer(service).oneshot(request).await.unwrap(),
                    Self::Secure=>ServiceBuilder::new()
                        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(http::header::X_CONTENT_TYPE_OPTIONS,HeaderValue::from_static("nosniff")))
                        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(http::header::X_FRAME_OPTIONS,HeaderValue::from_static("SAMEORIGIN")))
                        .layer(tower_http::set_header::SetResponseHeaderLayer::if_not_present(http::header::REFERRER_POLICY,HeaderValue::from_static("no-referrer")))
                        .service(service).oneshot(request).await.unwrap(),
                    Self::Compress(layer)=>layer.layer(service).oneshot(request).await.unwrap().map(Body::new),
                    #[cfg(feature="logging")]
                    Self::AccessLog(logger)=>{
                        static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1);
                        let id=NEXT.fetch_update(std::sync::atomic::Ordering::Relaxed,std::sync::atomic::Ordering::Relaxed,|n|n.checked_add(1)).expect("request identity exhausted");
                        let method=request.method().clone();let route=request.extensions().get::<RouteTemplate>().cloned();
                        logger.request(id,async move{
                            let mut guard=AccessLogGuard{logger:logger.share(),id,method,route,started:std::time::Instant::now(),finished:false};
                            let response=service.oneshot(request).await.unwrap();
                            if let Some(route)=response.extensions().get::<RouteTemplate>(){guard.route=Some(route.clone());}
                            guard.complete(response.status().as_u16());response
                        }).await
                    },
                }
            }
            async fn app(&self,request:Request,next:Next)->Response{
                let Request{inner,params,limit}=request;let mut response_limit=None;let mut owned=Some((next,params,&mut response_limit));
                let service=tower::service_fn(move|incoming:http::Request<Body>|{let(next,params,response_limit)=owned.take().expect("owned HTTP next is invoked once");async move{let response=next.run(Request{inner:incoming.map(ExclusiveBody::new),params,limit}).await;*response_limit=response.limit;Ok::<_,Infallible>(response.into_wire())}});
                let response=self.apply(inner.map(ExclusiveBody::into_body),service).await;Response{inner:response.map(ExclusiveBody::new),limit:response_limit}
            }
            async fn context<S:Send+Sync+'static>(&self,context:&mut Context<S>,next:ContextNext){
                // These selected Tower layers inspect request metadata, never its body.
                // Keep the sole real body in Context, including preflight short circuits.
                let mut request=http::Request::new(Body::empty());*request.method_mut()=context.field0.inner.method().clone();*request.uri_mut()=context.field0.inner.uri().clone();*request.version_mut()=context.field0.inner.version();*request.headers_mut()=context.field0.inner.headers().clone();
                let mut response_limit=None;let mut invoked=false;let mut owned=Some((&mut *context,next,&mut response_limit,&mut invoked));
                let service=tower::service_fn(move|_:http::Request<Body>|{let(context,next,response_limit,invoked)=owned.take().expect("owned context next is invoked once");async move{*invoked=true;next.run(context).await;let response=std::mem::replace(&mut context.field2,fixed(500,"Internal Server Error"));*response_limit=response.limit;let mut response=response.into_wire();if let Some(route)=context.field0.inner.extensions().get::<RouteTemplate>(){response.extensions_mut().insert(route.clone());}Ok::<_,Infallible>(response)}});
                let response=self.apply(request,service).await;
                let mut response=Response{inner:response.map(ExclusiveBody::new),limit:response_limit};
                if !invoked{context.apply_headers(&mut response);}
                context.field2=response;context.finalized=true;
            }
            pub fn into_context<S:Send+Sync+'static>(self)->ContextMiddleware<S>{let middleware=Arc::new(self);Arc::new(move|context,next|{let middleware=middleware.clone();Box::pin(async move{middleware.context(context,next).await;None})})}
        }
        #[cfg(test)]
        mod stream_owner_tests {
            use super::*;
            use std::{sync::atomic::{AtomicUsize,Ordering},task::{Context as TaskContext,Waker},future::Future};
            struct PendingBody {polled:Arc<AtomicUsize>,dropped:Arc<AtomicUsize>}
            impl Drop for PendingBody{fn drop(&mut self){self.dropped.fetch_add(1,Ordering::SeqCst);}}
            impl http_body::Body for PendingBody{type Data=axum::body::Bytes;type Error=std::io::Error;fn poll_frame(self:Pin<&mut Self>,_:&mut TaskContext<'_>)->Poll<Option<Result<http_body::Frame<Self::Data>,Self::Error>>>{self.polled.fetch_add(1,Ordering::SeqCst);Poll::Pending}}
            #[test]
            fn compression_drop_releases_polled_production_source_exactly_once(){tasks::block_on(async{
                let polled=Arc::new(AtomicUsize::new(0));let dropped=Arc::new(AtomicUsize::new(0));let mut source=Some(Body::new(PendingBody{polled:polled.clone(),dropped:dropped.clone()}));
                let endpoint=tower::service_fn(move|_:http::Request<Body>|{let body=source.take().unwrap();async move{let mut response=WireResponse::new(body);response.headers_mut().insert("content-type",HeaderValue::from_static("text/plain"));Ok::<_,Infallible>(response)}});
                let policy=NativeMiddleware::compress();let response=policy.apply(http::Request::builder().header("accept-encoding","gzip").body(Body::empty()).unwrap(),endpoint).await;assert_eq!(polled.load(Ordering::SeqCst),0);
                {let mut body=response.into_body();let mut frame=std::pin::pin!(body.frame());assert!(matches!(frame.as_mut().poll(&mut TaskContext::from_waker(Waker::noop())),Poll::Pending));}
                assert!(polled.load(Ordering::SeqCst)>0);assert_eq!(dropped.load(Ordering::SeqCst),1);
            });}
            #[test]
            fn preflight_short_circuit_retains_the_context_body_owner_until_scope_exit(){tasks::block_on(async{
                let polled=Arc::new(AtomicUsize::new(0));let dropped=Arc::new(AtomicUsize::new(0));let request=Request{inner:http::Request::builder().method("OPTIONS").uri("/private").header("origin","https://allowed.example").header("access-control-request-method","GET").body(ExclusiveBody::new(Body::new(PendingBody{polled:polled.clone(),dropped:dropped.clone()}))).unwrap(),params:BTreeMap::new(),limit:None};
                let mut context=super::super::Context::new(request,Arc::new(()));context.header("x-before","kept").unwrap();context.header("set-cookie","session=kept; HttpOnly").unwrap();let layers:Arc<Vec<ContextLayer<()>>>=Arc::new(Vec::new());let next=ContextNext{layers,index:0};NativeMiddleware::cors_default().context(&mut context,next).await;
                assert!(context.finalized);assert_eq!(context.field2.status(),200);assert_eq!(context.field2.inner.headers()["x-before"],"kept");assert_eq!(context.field2.inner.headers()["set-cookie"],"session=kept; HttpOnly");assert_eq!(polled.load(Ordering::SeqCst),0);assert_eq!(dropped.load(Ordering::SeqCst),0);drop(context);assert_eq!(dropped.load(Ordering::SeqCst),1);
            });}
            #[test]
            fn compressed_production_stream_preserves_source_errors(){tasks::block_on(async{
                struct FailingBody(u8);impl http_body::Body for FailingBody{type Data=axum::body::Bytes;type Error=std::io::Error;fn poll_frame(mut self:Pin<&mut Self>,_:&mut TaskContext<'_>)->Poll<Option<Result<http_body::Frame<Self::Data>,Self::Error>>>{let stage=self.0;self.0+=1;Poll::Ready(match stage{0=>Some(Ok(http_body::Frame::data(axum::body::Bytes::from_static(b"source prefix")))),1=>Some(Err(std::io::Error::other("source failure"))),_=>None})}}
                let endpoint=tower::service_fn(|_:http::Request<Body>|async{Ok::<_,Infallible>(WireResponse::new(Body::new(FailingBody(0))))});
                let policy=NativeMiddleware::compress();let response=policy.apply(http::Request::builder().header("accept-encoding","gzip").body(Body::empty()).unwrap(),endpoint).await;let mut body=Stream{body:response.into_body(),remaining:None};loop{match body.next().await{Ok(Some(_))=>{},Err(error)=>{assert_eq!(error.code,"body");break;},Ok(None)=>panic!("compression erased the source error")}}
            });}
        }
        #[cfg(feature="logging")]
        struct AccessLogGuard{logger:super::super::tsrust_native_logging::Logger,id:u64,method:Method,route:Option<RouteTemplate>,started:std::time::Instant,finished:bool}
        #[cfg(feature="logging")]
        impl AccessLogGuard{
            fn complete(&mut self,status:u16){self.finished=true;if self.logger.enabled("info"){self.logger.emit(||tracing::info!(message="request",request_id=self.id,method=self.method.as_str(),route=self.route.as_ref().map(|route|route.0.as_ref()).unwrap_or(""),status=i64::from(status),elapsed_ms=self.started.elapsed().as_secs_f64()*1000.0));}}
        }
        #[cfg(feature="logging")]
        impl Drop for AccessLogGuard{fn drop(&mut self){if !self.finished&&self.logger.enabled("info"){self.logger.emit(||tracing::info!(message=if std::thread::panicking(){"request_failed"}else{"request_cancelled"},request_id=self.id,method=self.method.as_str(),route=self.route.as_ref().map(|route|route.0.as_ref()).unwrap_or(""),elapsed_ms=self.started.elapsed().as_secs_f64()*1000.0));}}}
        impl App {pub fn use_native(&mut self,middleware:NativeMiddleware){let middleware=Arc::new(middleware);self.use_middleware(move|request,next|{let middleware=middleware.clone();async move{middleware.app(request,next).await}});}}
    }
    include!("runtime_realtime.rs");
    include!("runtime_websocket.rs");
}
