mod support;
#[test]
fn loopback_routes_middleware_streaming_and_cleanup_execute(){
 let source=support::format_round_trip(include_str!("../../../examples/http/loopback.tsr"));let directory=support::TempDirectory::new();let path=directory.source(&source);
 let output=support::run(support::cli().arg("run").arg(path),300);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"200\n{\"id\":\"42\",\"message\":\"Hello\"}\nhello\n");
}
#[test]
fn body_and_next_are_single_owner_capabilities(){
 for source in [
 "import {Body} from 'std:http';async function read(body:Body):Task<void>{const pending=body.next();const other=body.text();await pending;await other;}function main(){}",
 "import {App,Request,Response,text} from 'std:http';async function handle(input:Request):Task<Response>{return text('ok');}async function main():Task<void>{const app=new App();const pending=app.listen('127.0.0.1:0');app.get('/',handle);const server=await pending;await server.shutdown();}",
 "import {Request} from 'std:http';async function read(request:Request):Task<void>{await request.text();await request.text();}function main(){}",
 "import {Request,Next,Response} from 'std:http';async function middleware(request:Request,next:Next):Task<Response>{const first=await next.run(request);return await next.run(request);}function main(){}",
 "import {Request} from 'std:http';function bad(request:Request):void{const other=clone(request);}function main(){}",
 "import {App,Request,Response,text} from 'std:http';function main(){const app=new App();app.get('/',(request:Request):Response=>text('wrong'));}",
 ] {let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(source)),30);assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("error[syntax]")&&!error.contains("error[import]"),"{error}");}
}
#[test]
fn validated_json_body_executes(){
 let source=support::format_round_trip(include_str!("../../../examples/http/validated-body.tsr"));let directory=support::TempDirectory::new();let path=directory.source(&source);
 let output=support::run(support::cli().arg("run").arg(path),300);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"201\n{\"count\":2,\"name\":\"Pencil\"}\n400\n");
}
#[test]
fn http_catalog_examples_check_and_execute(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let directory=support::TempDirectory::new();
 let mut verified=std::collections::HashSet::new();
 for(family,entries)in catalog.as_object().unwrap(){if family!="std:http"&&!family.starts_with("Http"){continue;}
  for entry in entries.as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
   let source=support::format_round_trip(example["source"].as_str().unwrap());
   if !verified.insert((source.clone(),example["output"].as_str().unwrap().to_owned())){continue;}
   let path=directory.source(&source);let output=support::run(support::cli().arg("run").arg(path).env("TSRUST_CACHE_DIR",directory.0.join("cache")),300);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap(),"{family}.{}",entry["name"]);
  }}
 }
}

#[test]
fn http_runtime_cache_survives_application_changes(){
 let directory=support::TempDirectory::new();let cache=directory.0.join("cache");
 for value in ["Hello","Welcome"]{
  let source=include_str!("../../../examples/http/loopback.tsr").replace("Hello",value);let path=directory.source(&source);
  let output=support::run(support::cli().arg("run").arg(path).env("TSRUST_CACHE_DIR",&cache),300);support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains(value));
  let artifacts=std::fs::read_dir(cache.join("runtime/v2")).unwrap().filter_map(Result::ok).filter(|entry|entry.file_type().unwrap().is_dir()).count();assert_eq!(artifacts,1,"HTTP support must fit the bounded artifact cache");
 }
}
#[test]
fn server_wait_cancellation_executes_and_standalone_example_checks(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let example=&catalog["HttpServer"].as_array().unwrap().iter().find(|entry|entry["name"]=="wait").unwrap()["examples"][0];
 let directory=support::TempDirectory::new();let source=support::format_round_trip(example["source"].as_str().unwrap());let output=support::run(support::cli().arg("run").arg(directory.source(&source)),300);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),example["output"].as_str().unwrap());
 let output=support::run(support::cli().arg("check").arg(directory.source(include_str!("../../../examples/http/server.tsr"))),30);support::success(&output);
}

#[test]
fn consecutive_chunk_reads_release_the_receiver_borrow(){
 let source="import {App,Client,request,text} from 'std:http';import {length} from 'std:bytes';async function main():Task<void>{const body=text('abc').body();body.next();(body.next(),body.next());const first=await body.next();if(first===undefined){throw 'Missing chunk';}console.log(length(first));const second=await body.next();console.log(second===undefined);console.log(await text('done').text());const app=new App();const server=await app.listen('127.0.0.1:0');const client=new Client();console.log((await client.send(request('GET','http://'+server.address()+'/', ''))).status());await server.shutdown();}";
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(source))),300);support::success(&output);assert_eq!(output.stdout,b"3\ntrue\ndone\n404\n");
}
#[test]
fn https_example_executes_with_verified_private_ca(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/http/https.tsr"));
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)).current_dir(root),300);support::success(&output);assert_eq!(output.stdout,b"200\nHello over HTTPS\n");
}
#[test]
fn tls_options_preserve_checked_types_and_loans(){
 for source in [
 "import {Client} from 'std:http';function main(){new Client({ca:3});}",
 "import {App} from 'std:http';async function main():Task<void>{const app=new App();await app.listen('127.0.0.1:0',{cert:'pem'});}",
 "import {App,TlsOptions} from 'std:http';async function main():Task<void>{const app=new App();const tls:TlsOptions={cert:'pem',key:'key'};const pending=app.listen('127.0.0.1:0',tls);tls.key='changed';await pending;}",
 ]{let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(source)),30);assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("error[syntax]")&&!error.contains("error[import]"),"{error}");}
}

#[test]
fn tls_configuration_errors_are_typed(){
 let source="import {App,Client,TlsOptions} from 'std:http';import {strictEqual} from 'std:testing';async function main():Task<void>{const tls:TlsOptions={key:'bad key',cert:'bad cert'};let failed=false;try{const app=new App();await app.listen('127.0.0.1:0',tls);}catch(error){strictEqual(error.code,'tls');failed=true;}strictEqual(failed,true);failed=false;try{new Client({ca:''});}catch(error){strictEqual(error.code,'tls');failed=true;}strictEqual(failed,true);}";
 let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(source))),300);support::success(&output);
}
#[test]
#[ignore = "opt-in SQL/TLS integration: node tools/check-async-sql.mjs --docker"]
fn sql_and_https_features_share_one_generated_program_without_provider_ambiguity(){
 let directory=support::TempDirectory::new();
 let source=format!("import {{connect}} from 'std:sql';import {{strictEqual}} from 'std:testing';{}",include_str!("../../../examples/http/https.tsr").replace("async function main(): Task<void> {","async function main(): Task<void> {const db=await connect('sqlite::memory:',1);const rows=await db.query('SELECT 42 AS value');strictEqual(unwrap(rows[0].int('value')),42);await db.close();"));
 assert!(source.contains("await db.query('SELECT 42 AS value')"),"HTTPS example entry point changed");
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let output=support::run(support::cli().arg("run").arg(directory.source(&support::format_round_trip(&source))).current_dir(root),600);support::success(&output);assert_eq!(output.stdout,b"200\nHello over HTTPS\n");
}

#[test]
fn producer_stream_executes_and_rejects_wrong_payload_or_reuse(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http/producer-stream.tsr"));
 support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)),300));
 for source in [
  "import {Channel} from 'std:futures';import {Bytes,fromUtf8} from 'std:bytes';async function main():Task<void>{const chunks=new Channel<Bytes>(1);await chunks.send(fromUtf8('invalid'));}",
  "import {streamChannel} from 'std:http';import {Channel} from 'std:futures';function main(){const chunks=new Channel<int>(1);streamChannel(chunks);}",
  "import {streamChannel} from 'std:http';import {Channel} from 'std:futures';import {Bytes} from 'std:bytes';function main(){const chunks=new Channel<Bytes>(1);streamChannel(chunks);chunks.close();}",
 ]{let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success());assert!(!String::from_utf8_lossy(&output.stderr).contains("error[import]"));}
}
