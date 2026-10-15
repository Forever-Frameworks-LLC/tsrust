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
  server.shutdown().await;assert!(tokio::net::TcpStream::connect(address).await.is_err());
 });
}
#[test]
fn validation_and_streaming_limits_do_not_silently_truncate(){
 tsrust_native_futures::block_on(async{
  let mut app=App::new();app.route("GET","/x/:id",|_|async{Response::text("ok".into(),200).unwrap()}).unwrap();
  for (method,path) in [("HEAD","/x/:id"),("POST","/x/:other"),("GET","/x/:id/:id"),("GET","/bad//path"),("GET","/x/*all")]{assert!(app.route(method,path,|_|async{failed()}).is_err());}
  assert!(Response::text("bad".into(),199).is_err());let mut request=Request::new("POST","http://localhost/","body".into()).unwrap();assert!(request.set_header("x-test","bad\r\nInjected: true").is_err());assert!(request.set_header("content-length","100").is_err());
  assert_eq!(request.method(),"POST");assert_eq!(request.url(),"http://localhost/");assert_eq!(request.text().await.unwrap(),"body");
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
   assert!(tokio::net::TcpStream::connect(address).await.is_err());
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
