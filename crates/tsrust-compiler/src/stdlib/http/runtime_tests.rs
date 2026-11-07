use tsrust_native_http::*;
#[test]
fn ordinary_server_admits_more_than_legacy_concurrency_and_explicit_limit_rejects(){
 use std::{sync::{Arc,atomic::{AtomicUsize,Ordering}},time::Duration};
 struct Release(Arc<tokio::sync::Semaphore>);impl Drop for Release{fn drop(&mut self){self.0.add_permits(256);}}
 tsrust_native_futures::block_on(async{
  for configured in [false,true]{
   let entered=Arc::new(AtomicUsize::new(0));let changed=Arc::new(tokio::sync::Notify::new());let gate=Arc::new(tokio::sync::Semaphore::new(0));
   let mut app=App::new();if configured{app.configure([64,256,128,30000,5000,5000,0]).unwrap();}
   let observed=entered.clone();let signal=changed.clone();let blocked=gate.clone();
   app.route("GET","/",move |_|{let observed=observed.clone();let signal=signal.clone();let blocked=blocked.clone();async move{observed.fetch_add(1,Ordering::SeqCst);signal.notify_waiters();let permit=blocked.acquire().await.unwrap();permit.forget();Response::text("ok".into(),200).unwrap()}}).unwrap();
   let server=app.listen("127.0.0.1:0").await.unwrap();let release=Release(gate);let url=format!("http://{}/",server.address());let client=reqwest::Client::new();let mut requests=tokio::task::JoinSet::new();
   for _ in 0..128{let client=client.clone();let url=url.clone();requests.spawn(async move{let response=client.get(url).send().await.unwrap();assert_eq!(response.status().as_u16(),200);assert_eq!(response.text().await.unwrap(),"ok");});}
   tokio::time::timeout(Duration::from_secs(5),async{loop{let ready=changed.notified();if entered.load(Ordering::SeqCst)>=128{break;}ready.await;}}).await.expect("initial handlers did not enter");
   if configured{let response=tokio::time::timeout(Duration::from_secs(5),client.get(&url).send()).await.expect("explicit overload did not settle").unwrap();assert_eq!(response.status().as_u16(),503);assert_eq!(entered.load(Ordering::SeqCst),128);}
   else{let client=client.clone();let url=url.clone();requests.spawn(async move{let response=client.get(url).send().await.unwrap();assert_eq!(response.status().as_u16(),200);assert_eq!(response.text().await.unwrap(),"ok");});tokio::time::timeout(Duration::from_secs(5),async{loop{let ready=changed.notified();if entered.load(Ordering::SeqCst)>=129{break;}ready.await;}}).await.expect("ordinary backend still had the legacy128 cap");}
   drop(release);tokio::time::timeout(Duration::from_secs(5),async{while let Some(result)=requests.join_next().await{result.unwrap();}}).await.expect("released handlers did not settle");tokio::time::timeout(Duration::from_secs(5),server.shutdown()).await.expect("idle server did not drain");
  }
 });
}
#[test]
fn ordinary_server_body_reads_and_explicit_legacy_budgets_are_distinct(){
 tsrust_native_futures::block_on(async{
  for limit in [None,Some(1048576),Some(1048577)]{
   let mut app=App::new();if let Some(limit)=limit{app.configure([limit,8,8,5000,5000,5000,0]).unwrap();}
   app.route("POST","/echo",|request|async move{Response::stream(request.body(),200).unwrap()}).unwrap();
   let server=app.listen("127.0.0.1:0").await.unwrap();
   if limit==Some(1048576){
    // This policy rejects from Content-Length before consuming the body. Send
    // only headers so an in-flight upload cannot race the early 413 response.
    use tokio::io::{AsyncBufReadExt,AsyncWriteExt};
    let status=tokio::time::timeout(std::time::Duration::from_secs(5),async{
     let mut socket=tokio::net::TcpStream::connect(server.address()).await.unwrap();
     socket.write_all(b"POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1048577\r\nConnection: close\r\n\r\n").await.unwrap();
     let mut response=tokio::io::BufReader::new(socket);let mut status=String::new();response.read_line(&mut status).await.unwrap();status
    }).await.expect("header-only over-limit request did not receive its early response");
    assert!(status.starts_with("HTTP/1.1 413 "),"{status:?}");
   }else{
    let response=reqwest::Client::new().post(format!("http://{}/echo",server.address())).body("x".repeat(1048577)).send().await.unwrap();
    assert_eq!(response.status().as_u16(),200);let text=response.text().await.unwrap();assert_eq!(text.len(),1048577);assert!(text.bytes().all(|byte|byte==b'x'));
   }
   server.shutdown().await;
  }
 });
}
#[test]
fn explicit_http_bounds_validate_before_read_and_preserve_configured_limits(){
 tsrust_native_futures::block_on(async{
  for limit in [-1,i64::MIN]{assert_eq!(Client::with_options(None,Some(limit)).err().unwrap().code,"options");}
  #[cfg(target_pointer_width="32")]
  assert_eq!(Client::with_options(None,Some(i64::from(u32::MAX)+1)).err().unwrap().code,"options");
  let make_form=|body:String|{let mut input=Request::new("POST","http://localhost/",body).unwrap();input.set_header("content-type","application/x-www-form-urlencoded").unwrap();input};
  assert_eq!(make_form("a=b".into()).form(Some(-1)).await.err().unwrap().code,"options");
  assert!(make_form(String::new()).form(Some(0)).await.unwrap().is_empty());
  assert_eq!(make_form("a=b".into()).form(Some(0)).await.err().unwrap().code,"limit");
  let fields=(0..129).map(|i|format!("n={i}")).collect::<Vec<_>>().join("&");
  assert_eq!(make_form(fields.clone()).form(None).await.unwrap().len(),129);
  assert_eq!(make_form(fields.clone()).form(Some(128)).await.err().unwrap().code,"limit");
  let fields=make_form(fields).form(Some(129)).await.unwrap();assert_eq!(fields.len(),129);assert_eq!(fields[128].value,"128");
  let make_parts=||{let mut body=String::new();for i in 0..129{body.push_str(&format!("--x\r\nContent-Disposition: form-data; name=\"n\"\r\n\r\n{i}\r\n"));}body.push_str("--x--\r\n");let mut input=Request::new("POST","http://localhost/",body).unwrap();input.set_header("content-type","multipart/form-data; boundary=x").unwrap();input};
  assert_eq!(make_parts().multipart(None).await.unwrap().len(),129);
  assert_eq!(make_parts().multipart(Some(128)).await.err().unwrap().code,"limit");
  let parts=make_parts().multipart(Some(129)).await.unwrap();assert_eq!(parts.len(),129);assert_eq!(parts[128].data,b"128");
  let mut app=App::new();app.route("GET","/bytes",|_|async{Response::text("abcd".into(),200).unwrap()}).unwrap();app.route("GET","/empty",|_|async{Response::text(String::new(),200).unwrap()}).unwrap();
  app.route("GET","/above-old-limit",|_|async{Response::text("x".repeat(1048577),200).unwrap()}).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let base=format!("http://{}",server.address());
  for(limit,path,ok)in[(0,"empty",true),(0,"bytes",false),(3,"bytes",false),(4,"bytes",true)]{
   let client=Client::with_options(None,Some(limit)).unwrap();let response=client.send(Request::new("GET",&format!("{base}/{path}"),String::new()).unwrap()).await.unwrap();let result=response.text_body().await;
   if ok{assert_eq!(result.unwrap(),if path=="empty"{""}else{"abcd"});}else{assert_eq!(result.err().unwrap().code,"limit");}
  }
  for limit in [1048576,1048577]{
   let client=Client::with_options(None,Some(limit)).unwrap();let response=client.send(Request::new("GET",&format!("{base}/above-old-limit"),String::new()).unwrap()).await.unwrap();
   let result=response.text_body().await;
   if limit==1048577{let text=result.unwrap();assert_eq!(text.len(),1048577);assert!(text.bytes().all(|byte|byte==b'x'));}else{assert_eq!(result.err().unwrap().code,"limit");}
  }
  let response=Client::new().unwrap().send(Request::new("GET",&format!("{base}/above-old-limit"),String::new()).unwrap()).await.unwrap();
  assert_eq!(response.text_body().await.unwrap().len(),1048577);
  server.shutdown().await;
 });
}
#[test]
fn client_default_redirect_policy_follows_relative_locations_and_stops_loops(){
 tsrust_native_futures::block_on(async{
  let mut app=App::new();
  app.route("GET","/start",|_|async{let mut response=Response::text(String::new(),302).unwrap();response.set_header("location","/final").unwrap();response}).unwrap();
  app.route("GET","/final",|_|async{Response::text("followed".into(),200).unwrap()}).unwrap();
  app.route("GET","/loop",|_|async{let mut response=Response::text(String::new(),302).unwrap();response.set_header("location","/loop").unwrap();response}).unwrap();
  let observed=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  for path in ["/post-start","/post-final"]{let observed=observed.clone();app.route("POST",path,move |request|{let observed=observed.clone();async move{
   assert_eq!(request.method(),"POST");assert_eq!(request.header("content-length").unwrap().as_deref(),Some("17"));
   assert_eq!(request.header("transfer-encoding").unwrap(),None);
   let start=request.url().ends_with("/post-start");assert_eq!(request.text().await.unwrap(),"replayed é bytes");
   observed.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
   if start{let mut response=Response::text(String::new(),307).unwrap();response.set_header("location","/post-final").unwrap();response}else{Response::text("replayed".into(),200).unwrap()}
  }}).unwrap();}
  let server=app.listen("127.0.0.1:0").await.unwrap();let base=format!("http://{}",server.address());let client=Client::new().unwrap();
  let response=client.send(Request::new("POST",&format!("{base}/post-start"),"replayed é bytes".into()).unwrap()).await.unwrap();assert_eq!(response.status(),200);assert_eq!(response.text_body().await.unwrap(),"replayed");assert_eq!(observed.load(std::sync::atomic::Ordering::SeqCst),2);
  let response=client.send(Request::new("GET",&format!("{base}/start"),String::new()).unwrap()).await.unwrap();assert_eq!(response.status(),200);assert_eq!(response.text_body().await.unwrap(),"followed");
  let error=client.send(Request::new("GET",&format!("{base}/loop"),String::new()).unwrap()).await.err().expect("default backend redirect limit");assert_eq!(error.code,"request");
  server.shutdown().await;
 });
}
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
  let request=Request::new("POST","http://localhost/","x".repeat(1048577)).unwrap();assert_eq!(request.text().await.unwrap().len(),1048577);
  let request=Request::new("POST","http://localhost/","x".repeat(1048577)).unwrap();let response=Response::stream(request.body(),200).unwrap();assert_eq!(response.text_body().await.unwrap().len(),1048577);
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
   let observed_handler=observed.clone();let mut app=App::new();
   // This witness requires a bounded graceful shutdown; it is application policy.
   app.configure([1048576,128,128,30000,50,5000,0]).unwrap();
   app.route("GET","/pending",move |_|{let guard=Guard(observed_handler.clone());let signal=signal.clone();async move{
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
   assert!(matches!(Router::new().listen_tls("127.0.0.1:0",cert,key).await,Err(Error{code:"tls",..})));
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
  let fields=form.form(None).await.unwrap();assert_eq!(fields.len(),3);assert_eq!(fields[0].value,"one two");assert_eq!(fields[1].name,"a");assert_eq!(fields[1].value,"é");assert_eq!(fields[2].value,"");
  for input in ["a=%","a=%GG","a=%FF"]{let mut request=Request::new("POST","http://localhost/",input.into()).unwrap();request.set_header("content-type","application/x-www-form-urlencoded").unwrap();assert!(request.form(None).await.is_err());}
  let mut upload=Request::new("POST","http://localhost/","--x\r\nContent-Disposition: form-data; name=\"attachment\"; filename=\"../untrusted\"\r\nContent-Type: application/octet-stream\r\n\r\nabc\r\n--x--\r\n".into()).unwrap();upload.set_header("content-type","multipart/form-data; boundary=x").unwrap();
  let parts=upload.multipart(None).await.unwrap();assert_eq!(parts[0].name,"attachment");assert_eq!(parts[0].content_type.as_deref(),Some("application/octet-stream"));assert_eq!(parts[0].data,b"abc");assert_eq!(parts[0].file_name.as_deref(),Some("../untrusted"));
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
  assert!(sender.send(b"first".to_vec()).await.is_ok());
  let mut body=Response::stream_channel(channel,200,|bytes|bytes).unwrap().body();
  {let send=sender.send(b"second".to_vec());tokio::pin!(send);
  assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(matches!(std::future::Future::poll(send.as_mut(),cx),std::task::Poll::Pending))).await,"full channel must suspend the producer");
  assert_eq!(body.next().await.unwrap(),Some(b"first".to_vec()));
  assert!(send.await.is_ok());}drop(sender);
  assert_eq!(body.next().await.unwrap(),Some(b"second".to_vec()));
  assert_eq!(body.next().await.unwrap(),None,"the bridge must release the channel's internal sender");
  let channel=tsrust_native_futures::Channel::new(1).unwrap();let sender=channel.sender();
  let response=Response::stream_channel(channel,200,|bytes:Vec<u8>|bytes).unwrap();
  assert!(sender.send(vec![1]).await.is_ok());
  let blocked=sender.send(vec![2]);tokio::pin!(blocked);
  assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(matches!(std::future::Future::poll(blocked.as_mut(),cx),std::task::Poll::Pending))).await);
  drop(response);
  assert!(matches!(blocked.await,Err(tsrust_native_futures::SendError{code:"closed",value}) if value==vec![2]));
  let channel=tsrust_native_futures::Channel::new(1).unwrap();let sender=channel.sender();
  assert!(Response::stream_channel(channel,199,|bytes:Vec<u8>|bytes).is_err());
  assert!(matches!(sender.send(vec![1]).await,Err(tsrust_native_futures::SendError{code:"closed",..})));
 });
}

    #[test]
    fn static_configuration_is_atomic_and_streamed_routes_preserve_policy() {
        let root=std::env::temp_dir().join(format!("tsrust-http-static-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let directory=Cleanup(root);
        std::fs::write(directory.0.join("index.html"), "<h1>web</h1>").unwrap();
        std::fs::write(directory.0.join("file.txt"), "native file").unwrap();
        let root = directory.0.to_str().unwrap();
        let mut app = tsrust_native_http::Router::new();
        app.use_middleware(|context,next|Box::pin(async move{
            context.header("x-before","entry").unwrap();
            context.header("x-content-type-options","nosniff").unwrap();
            next.run(context).await;
            context.header("x-after","exit").unwrap();
            None
        }));
        app.route("GET","/raw",|_|Box::pin(async{Some(tsrust_native_http::Response::text("raw response".into(),200).unwrap())})).unwrap();
        for (prefix, fallback) in [("/", ""), ("/bad/", ""), ("/web", "../secret.html"), ("/web", "missing.html"), ("/web", "file.txt")] {
            assert_eq!(app.serve_static(prefix, root, fallback).unwrap_err().code, "static");
        }
        app.serve_static("/web", root, "index.html").unwrap();
        assert!(app.serve_static("/web/nested", root, "").is_err());
        assert!(app.serve_static("/web", root, "").is_err());
        app.serve_static("/second", root, "").unwrap();
        tsrust_native_futures::block_on(async {
            let server = app.listen("127.0.0.1:0").await.unwrap();
            let client = tsrust_native_http::Client::new().unwrap();
            for (path, accept, status, body) in [
                ("/web/file.txt", "*/*", 200, "native file"),
                ("/raw", "*/*", 200, "raw response"),
                ("/web/projects", "text/html", 200, "<h1>web</h1>"),
                ("/web/projects", "text/html;q=0", 404, "Not Found"),
                ("/second/projects", "text/html", 404, "Not Found"),
                ("/web/missing.js", "text/html", 404, "Not Found"),
                ("/api", "text/html", 404, "Not Found"),
            ] {
                let mut request = tsrust_native_http::Request::new("GET", &format!("http://{}{path}", server.address()), String::new()).unwrap();
                request.set_header("accept", accept).unwrap();
                let response = client.send(request).await.unwrap();
                assert_eq!(response.status(), status, "{path}");
                assert_eq!(response.header("x-before").unwrap().as_deref(),Some("entry"),"{path}");
                assert_eq!(response.header("x-after").unwrap().as_deref(),Some("exit"),"{path}");
                assert_eq!(response.header("x-content-type-options").unwrap().as_deref(),Some("nosniff"),"{path}");
                assert_eq!(response.text_body().await.unwrap(), body, "{path}");
            }
            server.shutdown().await;
        });
    }

#[test]
fn context_body_readers_consume_once_and_preserve_response_status(){
 tsrust_native_futures::block_on(async{
  let mut app=Router::new();assert!(app.configure([0,1,1,1,1,1,0]).is_err());
  app.configure([1048576,128,128,30000,5000,5000,0]).unwrap();
  app.route("POST","/text",|context|Box::pin(async move{
   context.status(202).unwrap();let value=context.request_text().await.unwrap();
   assert_eq!(context.request_text().await.unwrap_err().code,"body");Some(context.text(value,None).unwrap())
  })).unwrap();
  app.route("POST","/form",|context|Box::pin(async move{
   assert_eq!(context.request_form(Some(-1)).await.err().unwrap().code,"options");
   let fields=context.request_form(None).await.unwrap();assert_eq!(fields[0].name,"name");
   Some(context.text(fields[0].value.clone(),None).unwrap())
  })).unwrap();
  app.route("POST","/stream",|context|Box::pin(async move{
   let body=context.request_body().unwrap();
   assert_eq!(context.request_body().err().unwrap().code,"body");
   assert_eq!(context.request_text().await.unwrap_err().code,"body");
   assert_eq!(context.field0.method(),"POST");
   assert_eq!(context.field0.header("content-type").unwrap().as_deref(),Some("text/plain"));
   let value=body.text().await.unwrap();
   Some(context.text(value,None).unwrap())
  })).unwrap();
  app.route("POST","/multipart",|context|Box::pin(async move{
   assert_eq!(context.request_multipart(Some(-1)).await.err().unwrap().code,"options");
   let parts=context.request_multipart(None).await.unwrap();assert_eq!(parts[0].name,"name");
   Some(context.text(String::from_utf8(parts[0].data.clone()).unwrap(),None).unwrap())
  })).unwrap();
  let server=app.listen("127.0.0.1:0").await.unwrap();let client=Client::new().unwrap();
  for (path,body,content_type,status) in [("stream","hello","text/plain",200),("text","hello","text/plain",202),("form","name=hello","application/x-www-form-urlencoded",200),("multipart","--boundary\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nhello\r\n--boundary--\r\n","multipart/form-data; boundary=boundary",200)]{
   let mut request=Request::new("POST",&format!("http://{}/{path}",server.address()),body.into()).unwrap();request.set_header("content-type",content_type).unwrap();
   let response=client.send(request).await.unwrap();assert_eq!(response.status(),status);assert_eq!(response.text_body().await.unwrap(),"hello");
  }
  server.shutdown().await;
 });
}

#[test]
fn request_queries_preserve_repetition_empty_values_and_undecodable_bytes(){
 let request=Request::new("GET","http://localhost/?a=one+two&a=%C3%A9&empty=&bad=%FF",String::new()).unwrap();
 assert_eq!(request.query("a").as_deref(),Some("one two"));
 assert_eq!(request.queries("a"),Some(vec!["one two".into(),"é".into()]));
 assert_eq!(request.query("empty").as_deref(),Some(""));assert_eq!(request.query("missing"),None);
 assert_eq!(request.queries("missing"),None);assert_eq!(request.query("bad").as_deref(),Some("%FF"));
}
