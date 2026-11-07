use super::tsrust_native_http::*;
use super::tsrust_native_futures as tasks;
use std::time::Duration;
fn options(timeout:Option<i64>,connect:Option<i64>,follow:Option<bool>,maximum:Option<i64>,proxy:Option<&str>,none:Option<bool>)->Result<Client,Error>{Client::with_transport_options(None,None,timeout,connect,follow,maximum,proxy,none)}
#[test]
fn client_transport_policy_ranges_conflicts_and_redaction(){
 for value in [-1,i64::MIN]{assert_eq!(options(Some(value),None,None,None,None,None).err().unwrap().code,"options");assert_eq!(options(None,Some(value),None,None,None,None).err().unwrap().code,"options");assert_eq!(options(None,None,None,Some(value),None,None).err().unwrap().code,"options");}
 assert!(options(Some(0),Some(0),Some(true),Some(0),None,Some(false)).is_ok());
 assert_eq!(options(None,None,Some(false),Some(1),None,None).err().unwrap().code,"options");
 assert_eq!(options(None,None,None,None,Some("http://synthetic-user:synthetic-password@127.0.0.1"),Some(true)).err().unwrap().code,"options");
 let invalid=options(None,None,None,None,Some("http://synthetic-user:synthetic-password@[invalid"),None).err().unwrap();assert_eq!(invalid.code,"options");assert!(!invalid.message.contains("synthetic"));
}
#[test]
fn binary_stream_channel_uploads_keep_exact_payload_order_and_framing(){tasks::block_on(async{
 let mut app=App::new();app.route("POST","/upload/:kind",|request|async move{
  let kind=request.param("kind").unwrap();let length=request.header("content-length").unwrap();let transfer=request.header("transfer-encoding").unwrap();
  if kind=="bytes"{assert_eq!(length.as_deref(),Some("65537"));assert!(transfer.is_none());}else{assert!(length.is_none());assert_eq!(transfer.as_deref(),Some("chunked"));}
  let mut body=request.body();let mut offset=0;while let Some(chunk)=body.next().await.unwrap(){for byte in chunk{assert_eq!(byte,(offset%251)as u8);offset+=1;}}assert_eq!(offset,65537);
  Response::text("exact binary".into(),200).unwrap()
 }).unwrap();let server=app.listen("127.0.0.1:0").await.unwrap();let client=options(None,None,None,None,None,Some(true)).unwrap();
 for mode in ["bytes","stream","channel"]{let bytes=(0..65537).map(|i|(i%251)as u8).collect::<Vec<_>>();let url=format!("http://{}/upload/{mode}",server.address());
  let input=if mode=="bytes"{Request::from_bytes("POST",&url,bytes).unwrap()}else{let channel=tasks::Channel::new(2).unwrap();let sender=channel.sender();let mut bytes=bytes;let tail=bytes.split_off(32768);assert!(sender.try_send(bytes).is_ok());assert!(sender.try_send(tail).is_ok());drop(sender);if mode=="stream"{Request::from_stream("POST",&url,Response::stream_channel(channel,200,|chunk|chunk).unwrap().body()).unwrap()}else{Request::from_channel("POST",&url,channel,|chunk|chunk).unwrap()}};
  let response=tokio::time::timeout(Duration::from_secs(5),client.send(input)).await.unwrap().unwrap();assert_eq!(response.status(),200);assert_eq!(response.text_body().await.unwrap(),"exact binary");
 }server.shutdown().await;
});}
#[test]
fn channel_upload_disposal_wakes_stalled_sender_and_retains_unsent_owner(){tasks::block_on(async{
 use std::{future::Future,task::{Context,Poll,Waker}};
 let channel=tasks::Channel::new(1).unwrap();let sender=channel.sender();assert!(sender.try_send(vec![1u8]).is_ok());
 let unsent=vec![2u8,3];let pointer=unsent.as_ptr();let mut pending=Box::pin(sender.send(unsent));assert!(matches!(pending.as_mut().poll(&mut Context::from_waker(Waker::noop())),Poll::Pending));
 let input=Request::from_channel("POST","http://127.0.0.1:1/",channel,|chunk|chunk).unwrap();let client=Client::new().unwrap();let unpolled=client.send(input);drop(unpolled);
 let error=tokio::time::timeout(Duration::from_secs(5),pending).await.unwrap().err().expect("receiver disposal closes pending send");assert_eq!(error.code,"closed");assert_eq!(error.value,vec![2,3]);assert_eq!(error.value.as_ptr(),pointer);
});}
#[test]
fn explicit_redirect_policy_and_cross_origin_credentials_are_observed(){tasks::block_on(async{
 let mut target=App::new();target.route("POST","/final",|request|async move{assert!(request.header("authorization").unwrap().is_none());assert!(request.header("cookie").unwrap().is_none());assert_eq!(request.text().await.unwrap(),"replay é");Response::text("replayed".into(),200).unwrap()}).unwrap();let target=target.listen("127.0.0.1:0").await.unwrap();let destination=format!("http://{}/final",target.address());
 let mut origin=App::new();origin.route("POST","/redirect",move |_|{let destination=destination.clone();async move{let mut response=Response::text(String::new(),307).unwrap();response.set_header("location",&destination).unwrap();response}}).unwrap();let origin=origin.listen("127.0.0.1:0").await.unwrap();
 for(follow,limit,status)in[(Some(false),None,307),(Some(true),Some(2),200)]{let client=options(None,None,follow,limit,None,Some(true)).unwrap();let mut request=Request::from_bytes("POST",&format!("http://{}/redirect",origin.address()),"replay é".as_bytes().to_vec()).unwrap();request.set_header("authorization","Bearer synthetic").unwrap();request.set_header("cookie","synthetic=1").unwrap();let response=client.send(request).await.unwrap();assert_eq!(response.status(),status);if status==200{assert_eq!(response.text_body().await.unwrap(),"replayed");}}
 let client=options(None,None,None,Some(0),None,Some(true)).unwrap();let failure=client.send(Request::new("POST",&format!("http://{}/redirect?synthetic-secret",origin.address()),String::new()).unwrap()).await.err().unwrap();assert_eq!(failure.code,"request");assert!(!failure.message.contains("synthetic-secret"));
 origin.shutdown().await;target.shutdown().await;
});}
#[test]
fn explicit_proxy_auth_and_request_failure_redact_url_credentials(){tasks::block_on(async{
 let mut proxy=App::new();proxy.route("GET","/probe",|request|async move{assert_eq!(request.header("proxy-authorization").unwrap().as_deref(),Some("Basic dXNlcjpwYXNz"));Response::text("proxy used".into(),200).unwrap()}).unwrap();let proxy=proxy.listen("127.0.0.1:0").await.unwrap();
 let address=format!("http://user:pass@{}",proxy.address());let client=options(None,None,None,None,Some(&address),None).unwrap();let response=client.send(Request::new("GET","http://not-resolved.invalid/probe",String::new()).unwrap()).await.unwrap();assert_eq!(response.text_body().await.unwrap(),"proxy used");
 let error=client.send(Request::new("GET","http://user:pass@not-resolved.invalid/probe?synthetic-secret",String::new()).unwrap()).await.err().unwrap();assert_eq!(error.code,"url");assert!(!error.message.contains("user")&&!error.message.contains("pass")&&!error.message.contains("secret"));proxy.shutdown().await;
});}
