mod support;
#[test]
fn retained_readonly_configuration_survives_concurrent_requests(){
 let directory=support::TempDirectory::new();let source=include_str!("../../../examples/http-state/main.tsr").replace("  while (replies.length > 0)","  for (const reply of replies) { strictEqual(unwrap(clone(reply)), 'Shared, checked state'); }\n  while (replies.length > 0)");let source=support::format_round_trip(&source);
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&output);assert_eq!(output.stdout,b"HTTP retained state accepted\n");
}
#[test]
fn retained_handlers_reject_mutation_consumption_and_local_resources(){
 for source in [
 "import {App,Request,Response,text} from 'std:http';async function reply(input:Request):Task<Response>{return text('ok');}function routes(){let count=0;const app=new App();app.get('/',(input:Request)=>{count+=1;return reply(input);});return app;}function main(){routes();}",
 "import {App,Request,Response,text} from 'std:http';async function reply(value:string,input:Request):Task<Response>{return text(value);}function routes(value:string){const app=new App();app.get('/',(input:Request)=>reply(value,input));return app;}function main(){routes('once');}",
 "import {App,Request,Response,text} from 'std:http';import {LruCache} from 'std:cache';async function reply(input:Request):Task<Response>{return text('ok');}function routes(){const cache=new LruCache<int,int>(2);const app=new App();app.get('/',(input:Request)=>{cache.get(1);return reply(input);});return app;}function main(){routes();}",
 "import {App,Request,Response,text} from 'std:http';import {TrackedResource,resourceId} from 'std:testing';async function reply(input:Request):Task<Response>{return text('ok');}function routes(resource:TrackedResource){const app=new App();app.get('/',(input:Request)=>{resourceId(resource);return reply(input);});return app;}function main(){}",
 ]{
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains("error[http-state]"),"{error}");
 }
}
#[test]
fn retained_sql_pool_example_checks_without_a_database(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/http-state/sql.tsr"));support::success(&support::run(support::cli().arg("check").arg(directory.source(&source)),30));
}
#[test]
#[ignore="opt-in database acceptance: node tools/check-async-sql.mjs --docker"]
fn retained_sql_pool_serves_concurrent_requests_for_all_drivers(){
 let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").expect("set PostgreSQL URL");let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").expect("set MySQL URL");let resume=std::env::var("TSRUST_TEST_SQL_RESUME").unwrap_or_default();
 for(driver,url)in [("sqlite","sqlite::memory:"),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
  if (resume=="postgres"&&driver=="sqlite")||(resume=="mysql"&&driver!="mysql"){continue;}
  let source=include_str!("../../../examples/http-state/sql.tsr").replace("'sqlite::memory:'",&serde_json::to_string(url).unwrap());let source=support::format_round_trip(&source);let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(directory.source(&source)),300);support::success(&output);assert_eq!(output.stdout,b"HTTP retained SQL pool accepted\n","{driver}");eprintln!("HTTP retained {driver} pool accepted");
 }
}
#[test]
fn retained_state_permission_does_not_allow_invocation_local_or_ordinary_returns(){
 for source in [
 "import {App,Request,Response,text} from 'std:http';type Config={text:string;};async function reply(config:ReadOnly<Config>,input:Request):Task<Response>{return text(clone(config.text));}function main(){const app=new App();app.get('/',(input:Request)=>{const local:Config={text:'local'};return reply(local,input);});}",
 "import {Request,Response,text,request} from 'std:http';type Config={text:string;};async function reply(config:ReadOnly<Config>,input:Request):Task<Response>{return text(clone(config.text));}function invalid(config:Owned<Config>):Task<Response>{return reply(config,request('GET','http://localhost/',''));}function main(){invalid({text:'local'});}",
 ]{let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success(),"accepted {source}");assert!(String::from_utf8_lossy(&output.stderr).contains("error[callback-escape]"),"{}",String::from_utf8_lossy(&output.stderr));}
}
