mod support;
use std::fs;
use tsrust_compiler::{api::RouterContract,project::{FileSourceProvider,load_project,check_graph}};

#[test]
fn contracts_follow_checked_routes_without_api_configuration(){
    let dir=support::TempDirectory::new();
    let source=support::format_round_trip("import {Router,text,Context,Response} from 'std:http';type State={greeting:string;};async function named(c:Mutable<Context<State>>):Task<Response>{return c.json({ok:true});}async function main():Task<void>{const app=new Router<State>({greeting:'hello'});app.use(async(c,next)=>{c.header('x-before','yes');await next();});app.get('/',c=>{return c.json({greeting:clone(c.env.greeting),nested:{ready:true},encoded:JSON.stringify({x:1})});});app.get('/text',c=>{return c.text('hello');});app.get('/opaque',c=>{const discarded=c.json({ignored:true});return text('raw');});app.get('/named',named);const server=await app.listen('127.0.0.1:0');await server.shutdown();}");
    let entry=dir.0.join("main.tsr");fs::write(&entry,source).unwrap();
    let project=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();
    let contract=RouterContract::from_project(&project).unwrap();
    assert_eq!(contract.routes.len(),4,"{contract:?}");
    let root=contract.routes.iter().find(|route|route.path=="/").unwrap();assert_eq!(root.method,"GET");
    let schema=serde_json::to_value(root.json.as_ref().expect("inferred JSON response")).unwrap();
    assert_eq!(schema["properties"]["greeting"]["type"],"string");assert_eq!(schema["properties"]["encoded"]["type"],"string");assert_eq!(schema["properties"]["nested"]["properties"]["ready"]["type"],"boolean");
    assert!(contract.routes.iter().find(|route|route.path=="/text").unwrap().json.is_none());
    assert!(contract.routes.iter().find(|route|route.path=="/opaque").unwrap().json.is_none(),"discarded JSON is not a return contract");
    assert!(contract.routes.iter().find(|route|route.path=="/named").unwrap().json.is_some(),"named async response: {contract:?}");
    let files=contract.typescript_package().unwrap();assert!(files["index.d.ts"].contains("greeting"));
    assert_eq!(RouterContract::parse(&files["http-api.json"]).unwrap(),contract);
    for (name,text) in files{fs::write(dir.0.join(name),text).unwrap();}
    let script=r#"import {createClient,HttpError} from './index.js';import assert from 'node:assert/strict';let call;const client=createClient('http://example.test/prefix',{fetch:async(url,options)=>{call={url:String(url),options};return Response.json({greeting:'hello',nested:{ready:true},encoded:'encoded'});}});assert.deepEqual(await client.get('/',{query:{tag:['a','b']},headers:{'x-custom':'value'}}),{greeting:'hello',nested:{ready:true},encoded:'encoded'});assert.equal(call.url,'http://example.test/prefix/?tag=a&tag=b');assert.equal(call.options.method,'GET');assert.equal(call.options.headers.get('x-custom'),'value');await assert.rejects(()=>createClient('http://example.test',{fetch:async()=>Response.json({greeting:7})}).get('/'));await assert.rejects(()=>createClient('http://example.test',{fetch:async()=>new Response('private',{status:401})}).get('/'),error=>error instanceof HttpError&&error.response.status===401);const raw=await client.get('/text');assert.ok(raw instanceof Response);"#;
    fs::write(dir.0.join("check.mjs"),script).unwrap();support::success(&support::run(std::process::Command::new("node").current_dir(&dir.0).arg("check.mjs"),30));
}

#[test]
fn router_rust_client_compiles_and_sends_the_registered_wire_request(){
    use tsrust_compiler::api::{Route,Schema};use std::{collections::BTreeMap,process::Command};
    let contract=RouterContract{schema_version:1,routes:vec![Route{method:"POST".into(),path:"/echo/:id".into(),json:Some(Schema::Object{properties:BTreeMap::from([("count".into(),Schema::Integer)]),required:vec!["count".into()]})},Route{method:"GET".into(),path:"/raw".into(),json:None}]};
    let spec=contract.openapi().unwrap();let count=&spec["paths"]["/echo/{id}"]["post"]["responses"]["2XX"]["content"]["application/json"]["schema"]["properties"]["count"];assert_eq!(count["format"],"int64");assert!(count.get("maximum").is_none());
    let dir=support::TempDirectory::new();
    for (name,text) in contract.rust_package().unwrap(){let path=dir.0.join(name);fs::create_dir_all(path.parent().unwrap()).unwrap();fs::write(path,text).unwrap();}
    let manifest=dir.0.join("Cargo.toml");let text=fs::read_to_string(&manifest).unwrap();fs::write(&manifest,format!("{text}\n[dependencies.tokio]\nversion = \"=1.53.3\"\nfeatures = [\"rt\",\"time\",\"net\"]\n")).unwrap();
    fs::create_dir(dir.0.join("tests")).unwrap();
    fs::write(dir.0.join("tests/client.rs"),r###"
use tsrust_service_client::{Client,RequestOptions};
use std::{io::{Read,Write},net::TcpListener,time::Duration};
#[test]
fn client_uses_method_path_parameters_query_headers_and_json(){
 let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
 let server=std::thread::spawn(move||{for index in 0..2{let (mut stream,_)=listener.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();let mut bytes=Vec::new();let mut chunk=[0;2048];loop{let count=stream.read(&mut chunk).unwrap();assert!(count>0);bytes.extend_from_slice(&chunk[..count]);if let Some(end)=bytes.windows(4).position(|part|part==b"\r\n\r\n"){let head=String::from_utf8_lossy(&bytes[..end]);let length=head.lines().find_map(|line|line.to_ascii_lowercase().strip_prefix("content-length:").and_then(|value|value.trim().parse::<usize>().ok())).unwrap_or(0);if bytes.len()>=end+4+length{break;}}}let request=String::from_utf8(bytes).unwrap();if index==0{assert!(request.starts_with("POST /prefix/echo/a%2Fb?tag=x+y HTTP/1.1"),"{request}");assert!(request.to_ascii_lowercase().contains("x-auth: token"),"{request}");assert!(request.ends_with(r#"{"count":3}"#),"{request}");}else{assert!(request.starts_with("GET /prefix/raw HTTP/1.1"),"{request}");}let body=if index==0{r#"{"count":9007199254740993}"#}else{"raw"};write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();}});
 tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async{let client=Client::new(&format!("http://{address}/prefix")).unwrap();let mut options=RequestOptions::default();options.param.insert("id".into(),"a/b".into());options.query.push(("tag".into(),"x y".into()));options.headers.insert("x-auth","token".parse().unwrap());options.json=Some(serde_json::json!({"count":3}));assert_eq!(client.post_echo_id(options).await.unwrap().count,9007199254740993);assert_eq!(client.get_raw(RequestOptions::default()).await.unwrap().text().await.unwrap(),"raw");});server.join().unwrap();
}
"###).unwrap();
    support::success(&support::run(Command::new("cargo").current_dir(&dir.0).args(["test","--offline","--quiet","--test","client","--","--test-threads=3"]),180));
}

#[test]
fn saved_router_contracts_reject_invalid_shapes_and_ambiguous_routes(){
    for text in [
        r#"{"schemaVersion":2,"routes":[{"method":"GET","path":"/","json":null}]}"#,
        r#"{"schemaVersion":1,"routes":[{"method":"get","path":"/","json":null}]}"#,
        r#"{"schemaVersion":1,"routes":[{"method":"GET","path":"/?query","json":null}]}"#,
        r#"{"schemaVersion":1,"routes":[{"method":"GET","path":"/","json":null},{"method":"GET","path":"/","json":null}]}"#,
        r#"{"schemaVersion":1,"routes":[{"method":"GET","path":"/","json":{"type":"object","properties":{},"required":["missing"]}}]}"#,
    ]{assert!(RouterContract::parse(text).is_err(),"accepted {text}");}
}
