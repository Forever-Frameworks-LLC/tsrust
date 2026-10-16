use tsrust_native_http::*;
#[test]
fn loopback_routes_middleware_bodies_and_shutdown(){
 tsrust_native_futures::block_on(async{
  let mut app=App::new();
  app.use_middleware(|mut request,next|async move{request.set_header("x-before","yes").unwrap();let mut response=next.run(request).await;response.set_header("x-after","yes").unwrap();response});
  app.route("GET","/items/:id",|request|async move{assert_eq!(request.header("x-before").unwrap().as_deref(),Some("yes"));Response::text(request.param("id").unwrap(),200).unwrap()}).unwrap();
  app.route("POST","/echo",|request|async move{Response::stream(request.body(),201).unwrap()}).unwrap();
  app.fallback(|_|async{Response::text("Not Found".into(),404).unwrap()});
  let server=app.listen("127.0.0.1:0").await.unwrap();let address=server.address();let client=Client::new().unwrap();
  let response=client.send(Request::new("GET",&format!("http://{address}/items/42"),String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),200);assert_eq!(response.header("x-after").unwrap().as_deref(),Some("yes"));assert_eq!(response.text_body().await.unwrap(),"42");
  let response=client.send(Request::new("POST",&format!("http://{address}/echo"),"héllo".into()).unwrap()).await.unwrap();assert_eq!(response.status(),201);assert_eq!(response.text_body().await.unwrap(),"héllo");
  for (method,path,status) in [("GET","/missing",404),("POST","/items/42",405)]{let response=client.send(Request::new(method,&format!("http://{address}{path}"),String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),status);assert_eq!(response.header("x-after").unwrap().as_deref(),Some("yes"));}
  server.shutdown().await;assert!(tokio::net::TcpListener::bind(address).await.is_ok(),"shutdown must release the listening socket");
 });
}
#[test]
fn validation_and_streaming_limits_do_not_silently_truncate(){
 tsrust_native_futures::block_on(async{
  let mut app=App::new();app.route("GET","/x/:id",|_|async{Response::text("ok".into(),200).unwrap()}).unwrap();
  for (method,path) in [("HEAD","/x/:id"),("POST","/x/:other"),("GET","/x/:id/:id"),("GET","/bad//path"),("GET","/x/*all")]{assert!(app.route(method,path,|_|async{failed()}).is_err());}
  assert!(Response::text("bad".into(),199).is_err());let mut request=Request::new("POST","http://localhost/","body".into()).unwrap();assert!(request.set_header("x-test","bad\r\nInjected: true").is_err());assert!(request.set_header("content-length","100").is_err());
  assert_eq!(request.method(),"POST");assert_eq!(request.url(),"http://localhost/");assert_eq!(request.text().await.unwrap(),"body");
  let mut body=Request::new("POST","http://localhost/","abc".into()).unwrap().body();assert_eq!(body.next().await.unwrap(),Some(b"abc".to_vec()));assert_eq!(body.next().await.unwrap(),None);
  let request=Request::new("POST","http://localhost/","x".repeat(1048577)).unwrap();assert_eq!(request.text().await.unwrap_err().code,"limit");
  let request=Request::new("POST","http://localhost/","x".repeat(1048577)).unwrap();let response=Response::stream(request.body(),200).unwrap();assert!(response.text_body().await.is_err());
 });
}
#[test]
fn request_failure_and_server_drop_drain_nested_tasks(){
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 let drops=Arc::new(AtomicUsize::new(0));let seen=drops.clone();
 tsrust_native_futures::block_on(async move{
  let mut app=App::new();app.route("GET","/panic",move |_|{let guard=Guard(seen.clone());async move{let _child=tsrust_native_futures::start(async move{let _guard=guard;std::future::pending::<()>().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});panic!("private failure")}}).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();let response=client.send(Request::new("GET",&format!("http://{}/panic",server.address()),String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),500);assert_eq!(response.text_body().await.unwrap(),"Internal Server Error");drop(server);
 });
 assert_eq!(drops.load(Ordering::SeqCst),1);
}

#[test]
fn disconnect_and_shutdown_release_inflight_request_children(){
 use std::{io::Write,sync::{Arc,atomic::{AtomicUsize,Ordering}}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 for disconnect in [true,false]{
  let drops=Arc::new(AtomicUsize::new(0));let observed=drops.clone();
  tsrust_native_futures::block_on(async move{
   let started=Arc::new(tokio::sync::Notify::new());let signal=started.clone();
   let observed_handler=observed.clone();let mut app=App::new();app.route("GET","/pending",move |_|{let guard=Guard(observed_handler.clone());let signal=signal.clone();async move{
    let _child=tsrust_native_futures::start(async move{let _guard=guard;signal.notify_one();std::future::pending::<()>().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});
    std::future::pending::<Response>().await
   }}).unwrap();
   let server=app.listen("127.0.0.1:0").await.unwrap();let address=server.address();
   let mut socket=std::net::TcpStream::connect(&address).unwrap();socket.write_all(b"GET /pending HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
   tokio::time::timeout(std::time::Duration::from_secs(2),started.notified()).await.unwrap();
   if disconnect{drop(socket);tokio::time::timeout(std::time::Duration::from_secs(2),async{while observed.load(Ordering::SeqCst)==0{tokio::task::yield_now().await;}}).await.unwrap();server.shutdown().await;}
   else{tokio::time::timeout(std::time::Duration::from_secs(7),server.shutdown()).await.unwrap();drop(socket);}
   assert!(tokio::net::TcpListener::bind(address).await.is_ok(),"shutdown must release the listening socket");
  });
  assert_eq!(drops.load(Ordering::SeqCst),1);
 }
}
#[test]
fn middleware_order_short_circuit_and_encoded_parameters(){
 use std::sync::{Arc,Mutex};
 tsrust_native_futures::block_on(async{
  let trace=Arc::new(Mutex::new(Vec::new()));let mut app=App::new();
  for name in ["outer","inner"]{let trace=trace.clone();app.use_middleware(move|request,next|{let trace=trace.clone();async move{
   trace.lock().unwrap().push(format!("{name}:before"));
   let response=if name=="inner"&&request.header("x-stop").unwrap().is_some(){Response::text("Denied".into(),403).unwrap()}else{next.run(request).await};
   trace.lock().unwrap().push(format!("{name}:after"));response
  }});}
  app.route("GET","/:id",|request|async move{Response::text(request.param("id").unwrap(),200).unwrap()}).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();
  for stop in [false,true]{
   let mut request=Request::new("GET",&format!("http://{}/hello%20world",server.address()),String::new()).unwrap();if stop{request.set_header("x-stop","yes").unwrap();}
   let response=client.send(request).await.unwrap();assert_eq!(response.status(),if stop{403}else{200});assert_eq!(response.text_body().await.unwrap(),if stop{"Denied"}else{"hello world"});
   assert_eq!(std::mem::take(&mut *trace.lock().unwrap()),["outer:before","inner:before","inner:after","outer:after"]);
  }
  let response=client.send(Request::new("GET",&format!("http://{}/%FF",server.address()),String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),400);
  server.shutdown().await;
 });
}
#[test]
fn cancelling_server_wait_drains_its_listener(){
 tsrust_native_futures::block_on(async{
  let server=App::new().listen("127.0.0.1:0").await.unwrap();let address=server.address();
  let job=tsrust_native_futures::start(async move{server.wait().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});
  job.cancel();assert!(matches!(job.join().await,Err(tsrust_native_futures::Failure::Cancelled)));
  assert!(tokio::net::TcpListener::bind(address).await.is_ok(),"cancelled wait must finish listener cleanup");
 });
}

const TLS_CA:&str=include_str!("../../../tests/fixtures/tls/ca.pem");
const TLS_CERT:&str=include_str!("../../../tests/fixtures/tls/server.pem");
const TLS_KEY:&str=include_str!("../../../tests/fixtures/tls/server.key");
#[test]
fn tls_verifies_trust_hostname_and_validity(){
 tsrust_native_futures::block_on(async{
  for (cert,accepted) in [(TLS_CERT,true),(include_str!("../../../tests/fixtures/tls/wrong-host.pem"),false),(include_str!("../../../tests/fixtures/tls/expired.pem"),false),(include_str!("../../../tests/fixtures/tls/future.pem"),false)]{
   let calls=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));let observed=calls.clone();
   let mut app=App::new();app.route("POST","/echo",move|request|{let calls=observed.clone();async move{calls.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Response::stream(request.body(),201).unwrap()}}).unwrap();
   let server=app.listen_tls("127.0.0.1:0",cert,TLS_KEY).await.unwrap();let address=server.address();let url=format!("https://{address}/echo");
   let untrusted=Client::new().unwrap().send(Request::new("POST",&url,"hidden".into()).unwrap()).await;
   assert!(untrusted.is_err(),"default roots must not trust our private CA");
   let client=Client::with_ca(TLS_CA).unwrap();
   for _ in 0..3{
    let response=client.send(Request::new("POST",&url,"héllo".into()).unwrap()).await;
    if accepted{let response=response.unwrap();assert_eq!(response.status(),201);assert_eq!(response.text_body().await.unwrap(),"héllo");}else{assert!(response.is_err(),"invalid certificate accepted");}
   }
   assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst),if accepted{3}else{0});
   server.shutdown().await;assert!(tokio::net::TcpListener::bind(address).await.is_ok());
  }
 });
}
#[test]
fn tls_rejects_bad_configuration_and_drains_stalled_handshakes(){
 tsrust_native_futures::block_on(async{
  for (cert,key) in [("",TLS_KEY),("not PEM",TLS_KEY),(TLS_CERT,"bad key"),(TLS_CERT,include_str!("../../../tests/fixtures/tls/mismatch.key"))]{
   let result=App::new().listen_tls("127.0.0.1:0",cert,key).await;assert!(matches!(result,Err(Error{code:"tls",..})));
  }
  assert!(Client::with_ca("").is_err());assert!(Client::with_ca("not PEM").is_err());assert!(Client::with_ca("-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----").is_err());
  let server=App::new().listen_tls("127.0.0.1:0",TLS_CERT,TLS_KEY).await.unwrap();let address=server.address();
  let mut sockets=Vec::new();for _ in 0..16{sockets.push(tokio::net::TcpStream::connect(&address).await.unwrap());}
  tokio::time::sleep(std::time::Duration::from_millis(30)).await;
  tokio::time::timeout(std::time::Duration::from_secs(2),server.shutdown()).await.expect("shutdown must interrupt TLS handshakes");
  assert!(tokio::net::TcpListener::bind(&address).await.is_ok());
  use tokio::io::AsyncReadExt;
  for mut socket in sockets{let mut byte=[0];let read=tokio::time::timeout(std::time::Duration::from_secs(1),socket.read(&mut byte)).await.unwrap();assert!(matches!(read,Ok(0)|Err(_)));}
 });
}
#[test]
fn tls_negotiates_http_versions_and_cancellation_drains_children(){
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 let drops=Arc::new(AtomicUsize::new(0));let observed=drops.clone();
 tsrust_native_futures::block_on(async move{
  let started=Arc::new(tokio::sync::Notify::new());let signal=started.clone();
  let mut app=App::new();app.route("GET","/",|_|async{Response::text("ok".into(),200).unwrap()}).unwrap();
  app.route("GET","/pending",move |_|{let guard=Guard(observed.clone());let signal=signal.clone();async move{
   let _child=tsrust_native_futures::start(async move{let _guard=guard;signal.notify_one();std::future::pending::<()>().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});
   std::future::pending::<Response>().await
  }}).unwrap();
  let server=app.listen_tls("127.0.0.1:0",TLS_CERT,TLS_KEY).await.unwrap();let address=server.address();let url=format!("https://{address}");
  for http1 in [true,false]{
   let builder=reqwest::Client::builder().no_proxy().tls_certs_only([reqwest::Certificate::from_pem(TLS_CA.as_bytes()).unwrap()]);
   let client=if http1{builder.http1_only()}else{builder.http2_prior_knowledge()}.build().unwrap();
   let response=client.get(&url).send().await.unwrap();assert_eq!(response.version(),if http1{reqwest::Version::HTTP_11}else{reqwest::Version::HTTP_2});assert_eq!(response.text().await.unwrap(),"ok");
  }
  let job=tsrust_native_futures::start(async move{let client=Client::with_ca(TLS_CA).unwrap();let _=client.send(Request::new("GET",&format!("{url}/pending"),String::new()).unwrap()).await;Ok::<_,tsrust_native_futures::Failure<()>>(())});
  tokio::time::timeout(std::time::Duration::from_secs(2),started.notified()).await.unwrap();
  job.cancel();assert!(matches!(job.join().await,Err(tsrust_native_futures::Failure::Cancelled)));
  let wait=tsrust_native_futures::start(async move{server.wait().await;Ok::<_,tsrust_native_futures::Failure<()>>(())});
  wait.cancel();assert!(matches!(wait.join().await,Err(tsrust_native_futures::Failure::Cancelled)));
  assert!(tokio::net::TcpListener::bind(address).await.is_ok());
 });
 assert_eq!(drops.load(Ordering::SeqCst),1);
}

#[test]
fn retained_callback_state_outlives_requests_and_drops_once_after_shutdown(){
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 struct State{drops:Arc<AtomicUsize>,entered:Arc<tokio::sync::Notify>,pending:bool}
 impl State{async fn call(&self,_request:Request)->Response{self.entered.notify_one();if self.pending{std::future::pending::<()>().await;}Response::text("retained".into(),200).unwrap()}}
 impl Drop for State{fn drop(&mut self){self.drops.fetch_add(1,Ordering::SeqCst);}}
 for pending in [false,true]{
  let drops=Arc::new(AtomicUsize::new(0));let observed=drops.clone();
  tsrust_native_futures::block_on(async move{
   let entered=Arc::new(tokio::sync::Notify::new());let callback=Arc::new(State{drops:observed.clone(),entered:entered.clone(),pending});
   let mut app=App::new();app.route("GET","/",move|request|{let callback=Arc::clone(&callback);async move{callback.call(request).await}}).unwrap();
   let server=app.listen("127.0.0.1:0").await.unwrap();let url=format!("http://{}/",server.address());
   let job=tsrust_native_futures::start(async move{let response=Client::new().unwrap().send(Request::new("GET",&url,String::new()).unwrap()).await;Ok::<_,tsrust_native_futures::Failure<()>>(response.is_ok())});
   tokio::time::timeout(std::time::Duration::from_secs(2),entered.notified()).await.unwrap();assert_eq!(observed.load(Ordering::SeqCst),0);
   if pending{job.cancel();assert!(matches!(job.join().await,Err(tsrust_native_futures::Failure::Cancelled)));}else{assert!(matches!(job.join().await,Ok(true)));}
   assert_eq!(observed.load(Ordering::SeqCst),0);server.shutdown().await;
  });
  assert_eq!(drops.load(Ordering::SeqCst),1);
 }
}

#[test]
fn service_limits_forms_multipart_and_deadline_disposal(){
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 tsrust_native_futures::block_on(async{
  let mut form=Request::new("POST","http://localhost/","a=one+two&a=%C3%A9&empty=".into()).unwrap();form.set_header("content-type","application/x-www-form-urlencoded").unwrap();
  let fields=form.form().await.unwrap();assert_eq!(fields.len(),3);assert_eq!(fields[0].value,"one two");assert_eq!(fields[1].name,"a");assert_eq!(fields[1].value,"é");assert_eq!(fields[2].value,"");
  for input in ["a=%","a=%GG","a=%FF"]{let mut request=Request::new("POST","http://localhost/",input.into()).unwrap();request.set_header("content-type","application/x-www-form-urlencoded").unwrap();assert!(request.form().await.is_err());}
  let mut upload=Request::new("POST","http://localhost/","--x\r\nContent-Disposition: form-data; name=\"attachment\"; filename=\"../untrusted\"\r\nContent-Type: application/octet-stream\r\n\r\nabc\r\n--x--\r\n".into()).unwrap();upload.set_header("content-type","multipart/form-data; boundary=x").unwrap();
  let parts=upload.multipart().await.unwrap();assert_eq!(parts[0].name,"attachment");assert_eq!(parts[0].content_type.as_deref(),Some("application/octet-stream"));assert_eq!(parts[0].data,b"abc");assert_eq!(parts[0].file_name.as_deref(),Some("../untrusted"));
  let drops=Arc::new(AtomicUsize::new(0));let seen=drops.clone();let entered=Arc::new(tokio::sync::Notify::new());let ready=entered.clone();
  let mut app=App::new();assert!(app.configure([0,1,1,10,10,10,0]).is_err());app.configure([8,8,1,100,100,100,0]).unwrap();
  app.route("GET","/slow",move |_|{let guard=Guard(seen.clone());let ready=ready.clone();async move{let _guard=guard;ready.notify_one();std::future::pending::<Response>().await}}).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let base=format!("http://{}",server.address());let client=Client::new().unwrap();
  let url=format!("{base}/slow");let pending=tsrust_native_futures::start(async move{Ok::<_,tsrust_native_futures::Failure<()>>(Client::new().unwrap().send(Request::new("GET",&url,String::new()).unwrap()).await.unwrap().status())});
  entered.notified().await;
  let overloaded=client.send(Request::new("GET",&format!("{base}/slow"),String::new()).unwrap()).await.unwrap();assert_eq!(overloaded.status(),503);
  assert_eq!(pending.join().await.ok(),Some(504));server.shutdown().await;assert_eq!(drops.load(Ordering::SeqCst),1);
  let mut app=App::new();app.configure([8,8,2,1000,100,100,1]).unwrap();let server=app.listen("127.0.0.1:0").await.unwrap();let url=format!("http://{}/",server.address());
  assert_eq!(reqwest::Client::new().post(&url).body("123456789").send().await.unwrap().status().as_u16(),413);
  let response=client.send(Request::new("GET",&url,String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),429);assert_eq!(response.header("retry-after").unwrap().as_deref(),Some("1"));server.shutdown().await;
 });
}

#[test]
fn hono_context_middleware_unwinds_and_retains_closures_until_shutdown(){
 use std::sync::{Arc,Mutex,atomic::{AtomicUsize,Ordering}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 let drops=Arc::new(AtomicUsize::new(0));let observed=drops.clone();
 tsrust_native_futures::block_on(async move{
  let trace=Arc::new(Mutex::new(Vec::new()));let mut app=Router::new();
  for name in ["outer","inner"]{let trace=trace.clone();app.use_middleware(move|c,next|{let trace=trace.clone();Box::pin(async move{
   trace.lock().unwrap().push(format!("{name}:before"));
   c.header("x-entry",name).unwrap();next.run(c).await;c.header("x-exit",name).unwrap();
   trace.lock().unwrap().push(format!("{name}:after"));None
  })});}
  let guard=Guard(observed.clone());let handler_trace=trace.clone();
  app.route("GET","/early",move|c|{let _keep=&guard;let trace=handler_trace.clone();Box::pin(async move{trace.lock().unwrap().push("handler".into());Some(c.text("hello".into(),None).unwrap())})}).unwrap();
  app.use_middleware(|c,_|Box::pin(async move{Some(c.text("blocked".into(),Some(403)).unwrap())}));
  app.route("GET","/late",|_|Box::pin(async{panic!("short circuit must skip handler")})).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();
  for (path,status,body) in [("early",200,"hello"),("late",403,"blocked")]{
   trace.lock().unwrap().clear();let response=client.send(Request::new("GET",&format!("http://{}/{path}",server.address()),String::new()).unwrap()).await.unwrap();
   assert_eq!(response.status(),status);assert_eq!(response.header("x-exit").unwrap().as_deref(),Some("outer"));assert_eq!(response.text_body().await.unwrap(),body);
   let expected=if path=="early"{vec!["outer:before","inner:before","handler","inner:after","outer:after"]}else{vec!["outer:before","inner:before","inner:after","outer:after"]};
   assert_eq!(*trace.lock().unwrap(),expected);assert_eq!(observed.load(Ordering::SeqCst),0);
  }
  server.shutdown().await;
 });
 assert_eq!(drops.load(Ordering::SeqCst),1);
}

#[test]
fn hono_scoped_routes_keep_registration_order_and_method_boundaries(){
 use std::sync::Arc;
 tsrust_native_futures::block_on(async{
  let mut app=Router::new();
  app.use_many("/items/*",vec![Arc::new(|c,next|Box::pin(async move{c.header("x-scope","items").unwrap();next.run(c).await;None}))]).unwrap();
  app.route_many("GET","/items/:name",vec![
   Arc::new(|c,next|Box::pin(async move{let name=c.field0.param("name").unwrap();c.header("x-name",if name.is_ascii(){&name}else{"non-ascii"}).unwrap();next.run(c).await;c.header("x-unwind","yes").unwrap();None})),
   Arc::new(|c,_|Box::pin(async move{Some(c.text(if c.field0.param("name").is_some_and(|name|!name.is_ascii()){c.field0.param("name").unwrap()}else{"first".into()},None).unwrap())})),
  ]).unwrap();
  app.route("GET","/items/literal",|_|Box::pin(async{panic!("literal route cannot outrank registration order")})).unwrap();
  app.route("GET","/other",|c|Box::pin(async move{Some(c.text("other".into(),None).unwrap())})).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();let base=format!("http://{}",server.address());
  for (method,path,status,body,scope) in [("GET","/items/literal",200,"first",true),("GET","/items/a+b",200,"first",true),("GET","/items/%C3%A9",200,"é",true),("HEAD","/items/literal",200,"",true),("POST","/items/literal",404,"Not Found",true),("GET","/other",200,"other",false),("GET","/missing",404,"Not Found",false)]{
   let response=client.send(Request::new(method,&format!("{base}{path}"),String::new()).unwrap()).await.unwrap();
   assert_eq!(response.status(),status,"{method} {path}");assert_eq!(response.header("x-scope").unwrap().is_some(),scope);
   if method=="GET" && status==200 && scope{assert_eq!(response.header("x-name").unwrap().as_deref(),Some(if path.ends_with("%C3%A9"){"non-ascii"}else{path.rsplit('/').next().unwrap()}));}
   assert_eq!(response.text_body().await.unwrap(),body);
  }
  server.shutdown().await;
 });
}

#[test]
fn router_application_state_and_request_variables_have_distinct_lifetimes(){
 use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
 struct Guard(Arc<AtomicUsize>);impl Drop for Guard{fn drop(&mut self){self.0.fetch_add(1,Ordering::SeqCst);}}
 let application=Arc::new(AtomicUsize::new(0));let variables=Arc::new(AtomicUsize::new(0));
 tsrust_native_futures::block_on(async{
  let mut router=Router::with_state((Guard(application.clone()),variables.clone()));
  router.use_middleware(|c,next|Box::pin(async move{
   assert!(c.get::<Guard>("resource").is_none());
   c.set("resource",Guard(c.field1.1.clone()));c.set("resource",Guard(c.field1.1.clone()));
   next.run(c).await;None
  }));
  router.route("GET","/",|c|Box::pin(async move{assert!(c.get::<Guard>("resource").is_some());Some(c.text("ok".into(),None).unwrap())})).unwrap();
  let server=router.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();
  for _ in 0..2{let response=client.send(Request::new("GET",&format!("http://{}/",server.address()),String::new()).unwrap()).await.unwrap();assert_eq!(response.text_body().await.unwrap(),"ok");}
  assert_eq!(application.load(Ordering::SeqCst),0);server.shutdown().await;
 });
 assert_eq!(application.load(Ordering::SeqCst),1);assert_eq!(variables.load(Ordering::SeqCst),4);
}

#[test]
fn router_cookie_encoding_validation_and_after_next_headers(){
 tsrust_native_futures::block_on(async{
  let mut app=Router::new();
  app.use_middleware(|c,next|Box::pin(async move{
   assert_eq!(c.cookie("theme").as_deref(),Some("hé+"));
   assert_eq!(c.cookie("raw").as_deref(),Some("%FF"));
   assert_eq!(c.cookie("missing"),None);
   assert_eq!(c.field2.status(),404);
   assert!(c.set_cookie("__Host-session","x",CookieOptions::default()).is_err());
   assert!(c.set_cookie("x","x",CookieOptions{max_age:Some(-1),..Default::default()}).is_err());
   assert!(c.set_cookie("x","x",CookieOptions{path:Some("/;bad".into()),..Default::default()}).is_err());
   c.set_cookie("before","hé+",CookieOptions::default()).unwrap();
   next.run(c).await;
   assert_eq!(c.field2.status(),201);
   c.delete_cookie("__Host-session",CookieOptions{secure:Some(true),http_only:Some(true),same_site:Some("Strict".into()),..Default::default()}).unwrap();
   None
  }));
  app.route("GET","/",|c|Box::pin(async move{Some(c.text("ok".into(),Some(201)).unwrap())})).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();
  // Read wire headers independently: separate Set-Cookie headers must survive.
  let client=reqwest::Client::new();let response=client.get(format!("http://{}/",server.address())).header("cookie","theme=h%C3%A9+; theme=second; raw=%FF").send().await.unwrap();
  let cookies=response.headers().get_all("set-cookie").iter().map(|v|v.to_str().unwrap()).collect::<Vec<_>>();
  assert_eq!(cookies,["before=h%C3%A9%2B; Path=/","__Host-session=; Path=/; Max-Age=0; Secure; HttpOnly; SameSite=Strict"]);
  assert_eq!(response.text().await.unwrap(),"ok");server.shutdown().await;
 });
}

#[test]
fn router_metadata_is_shareable_without_sharing_body_access(){
 fn shareable<T:Send+Sync>(){}shareable::<Request>();shareable::<Response>();shareable::<Context<String>>();
 async fn observe(context:&Context<String>){tokio::task::yield_now().await;assert_eq!(context.field1.as_str(),"shared");assert_eq!(context.field0.method(),"GET");}
 tsrust_native_futures::block_on(async{
  let mut app=Router::with_state("shared".to_owned());
  app.route("GET","/",|c|Box::pin(async move{observe(c).await;Some(c.text("ok".into(),None).unwrap())})).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();
  let response=client.send(Request::new("GET",&format!("http://{}/",server.address()),String::new()).unwrap()).await.unwrap();
  assert_eq!(response.text_body().await.unwrap(),"ok");server.shutdown().await;
 });
}

#[test]
fn producer_channel_backpressure_end_and_disposal(){
 tsrust_native_futures::block_on(async{
  let channel=tsrust_native_futures::Channel::new(1).unwrap();
  let sender=channel.sender();
  assert!(sender.send::<()>(b"first".to_vec()).await.is_ok());
  let mut body=Response::stream_channel(channel,200,|bytes|bytes).unwrap().body();
  {let send=sender.send::<()>(b"second".to_vec());tokio::pin!(send);
  assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(matches!(std::future::Future::poll(send.as_mut(),cx),std::task::Poll::Pending))).await,"full channel must suspend the producer");
  assert_eq!(body.next().await.unwrap(),Some(b"first".to_vec()));
  assert!(send.await.is_ok());}drop(sender);
  assert_eq!(body.next().await.unwrap(),Some(b"second".to_vec()));
  assert_eq!(body.next().await.unwrap(),None,"the bridge must release the channel's internal sender");
  let channel=tsrust_native_futures::Channel::new(1).unwrap();let sender=channel.sender();
  let response=Response::stream_channel(channel,200,|bytes:Vec<u8>|bytes).unwrap();
  assert!(sender.send::<()>(vec![1]).await.is_ok());
  let blocked=sender.send::<()>(vec![2]);tokio::pin!(blocked);
  assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(matches!(std::future::Future::poll(blocked.as_mut(),cx),std::task::Poll::Pending))).await);
  drop(response);
  assert!(matches!(blocked.await,Err(tsrust_native_futures::Failure::Closed)));
  let channel=tsrust_native_futures::Channel::new(1).unwrap();let sender=channel.sender();
  assert!(Response::stream_channel(channel,199,|bytes:Vec<u8>|bytes).is_err());
  assert!(matches!(sender.send::<()>(vec![1]).await,Err(tsrust_native_futures::Failure::Closed)));
 });
}
