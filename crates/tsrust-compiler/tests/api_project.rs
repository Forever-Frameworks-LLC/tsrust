mod support;
use std::{fs,path::Path};
use tsrust_compiler::{project::{FileSourceProvider,SourceId,load_project,check_graph},project_session::ProjectSession};
fn setup(root:&Path)->String{
 fs::create_dir_all(root.join("app/api")).unwrap();fs::create_dir_all(root.join("app/lib")).unwrap();
 for (path,text) in [
  ("app/main.tsr",include_str!("../templates/application/app/main.tsr.template")),
  ("app/api/operations.tsr",include_str!("../templates/application/app/api/operations.tsr.template")),
  ("app/api/context.tsr",include_str!("../templates/application/app/api/context.tsr.template")),
  ("app/api/access.tsr",include_str!("../templates/application/app/api/access.tsr.template")),
  ("app/lib/greeting.tsr",include_str!("../templates/application/app/lib/greeting.tsr.template")),
 ]{fs::write(root.join(path),support::format_round_trip(text)).unwrap();}
 let config=serde_json::json!({"schemaVersion":1,"name":"api-project","build":{"entry":"app/main.tsr","output":"target/service"},"api":{
  "entry":"app/api/operations.tsr","package":{"name":"@app/api","version":"0.1.0"},
  "server":{"context":{"module":"app/api/context.tsr","export":"AppContext"},"authorize":{"module":"app/api/access.tsr","export":"authorize"}},
  "operations":[{"name":"greet","module":"app/api/operations.tsr","export":"greet","kind":"mutation"}]
 }});
 fs::write(root.join("tsrust.config.json"),serde_json::to_vec_pretty(&config).unwrap()).unwrap();
 fs::canonicalize(root.join("app/main.tsr")).unwrap().to_str().unwrap().into()
}
#[test]
fn server_dispatch_is_checked_in_memory_without_client_outputs_and_refreshes_after_edits(){
 let dir=support::TempDirectory::new();let entry=setup(&dir.0);let mut session=ProjectSession::default();
 let first=session.check(&FileSourceProvider,&entry).unwrap();
 let virtual_id=SourceId(fs::canonicalize(&dir.0).unwrap().join("target/tsrust-api/server.tsr").to_string_lossy().into_owned());
 let original=first.graph.modules[&virtual_id].source.text.clone();
 assert!(original.contains("export async function dispatch"));assert!(original.contains("export async function describe"));
 assert!(!dir.0.join("target").exists());assert!(!dir.0.join("generated").exists());
 let path=dir.0.join("app/api/operations.tsr");
 fs::write(&path,support::format_round_trip("export function greet(name: string): int { return name.length; }" )).unwrap();
 let changed=session.check(&FileSourceProvider,&entry).unwrap();
 assert_ne!(changed.graph.modules[&virtual_id].source.text,original);
 assert!(!dir.0.join("target").exists());
 let config_path=dir.0.join("tsrust.config.json");let mut config:serde_json::Value=serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
 config["api"]["server"]["maxRequestBytes"]=0.into();fs::write(&config_path,config.to_string()).unwrap();
 let rejected=session.check(&FileSourceProvider,&entry).err().unwrap();assert_eq!(rejected.code,"api-config");assert!(rejected.message.contains("byte limits"));
}
#[test]
fn operations_cannot_depend_on_dispatch_directly_or_indirectly(){
 let dir=support::TempDirectory::new();let entry=setup(&dir.0);
 let operations=dir.0.join("app/api/operations.tsr");let original=fs::read_to_string(&operations).unwrap();
 for import in ["import {dispatch} from 'api:server';","import {helper} from './bridge.tsr';"]{
  fs::write(dir.0.join("app/api/bridge.tsr"),support::format_round_trip("import {dispatch} from 'api:server'; export function helper(): int { return 1; }")).unwrap();
  fs::write(&operations,support::format_round_trip(&format!("{import}\n{original}"))).unwrap();
  let rejected=load_project(&FileSourceProvider,&entry).err().unwrap();assert_eq!(rejected.code,"api-cycle");
  assert!(!dir.0.join("target").exists());
 }
 fs::write(&operations,original).unwrap();check_graph(load_project(&FileSourceProvider,&entry).unwrap()).unwrap();
}

#[test]
fn unreadable_nearest_configuration_never_selects_a_parent_api_policy(){
 use tsrust_compiler::project::SourceProvider;
 struct Denied{config:SourceId}
 impl SourceProvider for Denied{
  fn resolve(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{FileSourceProvider.resolve(importer,path)}
  fn read(&self,id:&SourceId)->Result<String,String>{if id==&self.config{Err("Fixture configuration read denied".into())}else{FileSourceProvider.read(id)}}
 }
 let dir=support::TempDirectory::new();let child=dir.0.join("child");let entry=setup(&child);
 fs::write(dir.0.join("tsrust.config.json"),"{\"api\":{}}").unwrap();
 let config=SourceId(fs::canonicalize(child.join("tsrust.config.json")).unwrap().to_string_lossy().into_owned());
 let provider=Denied{config:config.clone()};let rejected=load_project(&provider,&entry).err().unwrap();
 assert_eq!(rejected.code,"api-config");assert_eq!(rejected.location.source,config);
 assert!(rejected.message.contains("configuration read denied"));assert!(!child.join("target").exists());
}

#[test]
fn wire_facts_and_executable_operations_share_the_same_source_and_reject_mixed_snapshots(){
 use std::cell::Cell;
 use tsrust_compiler::project::SourceProvider;
 struct Changing{operation:SourceId,reads:Cell<usize>}
 impl SourceProvider for Changing{
  fn resolve(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{FileSourceProvider.resolve(importer,path)}
  fn read(&self,id:&SourceId)->Result<String,String>{
   if id==&self.operation{
    let count=self.reads.get();self.reads.set(count+1);
    if count>0{return Ok(support::format_round_trip("export function greet(name: string): int { return name.length; }"));}
   }
   FileSourceProvider.read(id)
  }
 }
 let dir=support::TempDirectory::new();let entry=setup(&dir.0);
 let operation=SourceId(fs::canonicalize(dir.0.join("app/api/operations.tsr")).unwrap().to_string_lossy().into_owned());
 let original=fs::read_to_string(&operation.0).unwrap();
 let provider=Changing{operation:operation.clone(),reads:Cell::new(0)};
 let checked=check_graph(load_project(&provider,&entry).unwrap()).unwrap();
 assert_eq!(provider.reads.get(),1,"Manifest derivation and server lowering must not reread operations");
 assert_eq!(checked.graph.modules[&operation].source.text,original);
 let main=fs::read_to_string(&entry).unwrap();
 fs::write(&entry,support::format_round_trip(&format!("import {{greet}} from './api/operations.tsr';\n{main}"))).unwrap();
 let provider=Changing{operation,reads:Cell::new(0)};
 let rejected=load_project(&provider,&entry).err().unwrap();assert_eq!(rejected.code,"api-input-changed");
 assert!(!dir.0.join("target").exists());
}
