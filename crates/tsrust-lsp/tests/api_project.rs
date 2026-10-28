use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_diagnostics::convert_all};
#[test]
fn unsaved_api_operations_and_config_regenerate_dispatch_and_editor_diagnostics(){
 struct Directory(std::path::PathBuf);impl Drop for Directory{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
 let path=std::env::temp_dir().join(format!("tsrust-api-editor-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
 std::fs::create_dir(&path).unwrap();let dir=Directory(std::fs::canonicalize(path).unwrap());
 std::fs::create_dir_all(dir.0.join("app")).unwrap();
 let main="import {dispatch,describe} from 'api:server'; function main() {}";
 let operation="export function greet(name: string): string { return name; }";
 let context="import {Context} from 'std:http'; export type AppState = {name: string;}; export type AppContext = Context<AppState>; export async function authorize(c: Mutable<AppContext>): Task<boolean> { return true; }";
 let formatted=|text:&str|{let output=tsrust_compiler::format(text).unwrap();assert_eq!(tsrust_compiler::format(&output).unwrap(),output);output};
 for (file,text) in [("main.tsr",main),("operations.tsr",operation),("access.tsr",context)]{std::fs::write(dir.0.join("app").join(file),formatted(text)).unwrap();}
 let config=serde_json::json!({"schemaVersion":1,"name":"api-editor","build":{"entry":"app/main.tsr","output":"target/service"},"api":{
 "entry":"app/operations.tsr","package":{"name":"@app/api","version":"0.1.0"},"server":{"context":{"module":"app/access.tsr","export":"AppContext"},"authorize":{"module":"app/access.tsr","export":"authorize"}},"operations":[{"name":"greet","module":"app/operations.tsr","export":"greet","kind":"mutation"}]}});
 std::fs::write(dir.0.join("tsrust.config.json"),config.to_string()).unwrap();
 let entry=SourceId(dir.0.join("app/main.tsr").to_string_lossy().into_owned());
 let operations=SourceId(dir.0.join("app/operations.tsr").to_string_lossy().into_owned());
 let config_id=SourceId(dir.0.join("tsrust.config.json").to_string_lossy().into_owned());
 let virtual_id=SourceId(dir.0.join("target/tsrust-api/server.tsr").to_string_lossy().into_owned());
 let mut workspace=Workspace::default();let first=workspace.snapshot();let checked=first.check_with_diagnostics(&entry).unwrap();
 assert!(convert_all(&first,&checked.warnings()).unwrap().values().all(Vec::is_empty));
 let before=first.sources()[&virtual_id].clone();workspace.register(entry.clone(),&first);
 workspace.open(operations.clone(),1,formatted("export function greet(name: string): int { return name.length; }"));
 assert!(!workspace.current(&first));assert!(workspace.affected(&operations).contains(&entry));
 let second=workspace.snapshot();let checked=second.check_with_diagnostics(&entry).unwrap();
 assert_ne!(*second.sources()[&virtual_id],*before);assert!(convert_all(&second,&checked.warnings()).unwrap().values().all(Vec::is_empty));
 let mut invalid=config.clone();invalid["api"]["server"]["maxRequestBytes"]=0.into();workspace.open(config_id,1,invalid.to_string());
 let third=workspace.snapshot();let errors=third.check_with_diagnostics(&entry).err().unwrap();assert!(errors.iter().any(|e|e.code=="api-config"));
 assert!(convert_all(&third,&errors).unwrap().values().any(|v|!v.is_empty()));
 assert_eq!(std::fs::read_to_string(&operations.0).unwrap(),formatted(operation));assert!(!dir.0.join("target").exists());
}
