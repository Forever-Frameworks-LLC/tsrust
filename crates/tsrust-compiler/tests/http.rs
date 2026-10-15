mod support;
#[test]
fn loopback_routes_middleware_streaming_and_cleanup_execute(){
 let source=support::format_round_trip(include_str!("../../../examples/http/loopback.tsr"));let directory=support::TempDirectory::new();let path=directory.source(&source);
 let output=support::run(support::cli().arg("run").arg(path),300);support::success(&output);assert_eq!(String::from_utf8_lossy(&output.stdout),"200\n{\"id\":\"42\",\"message\":\"Hello\"}\nhello\n");
}
#[test]
fn body_and_next_are_single_owner_capabilities(){
 for source in [
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
 for(family,entries)in catalog.as_object().unwrap(){if family!="std:http"&&!family.starts_with("Http"){continue;}
  for entry in entries.as_array().unwrap(){for example in entry["examples"].as_array().unwrap(){
   let source=support::format_round_trip(example["source"].as_str().unwrap());
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
