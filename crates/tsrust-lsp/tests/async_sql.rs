use serde_json::json;
use tsrust_lsp::{project_diagnostics::source_id,project_intelligence::ProjectIndex,workspace::Workspace};
#[test]
fn sql_methods_and_inferred_rows_complete(){
 for (before,receiver,members) in [
 ("import {Pool} from 'std:sql';function inspect(x:Pool){","x.",vec!["query","execute","begin","close"]),
 ("import {Transaction} from 'std:sql';function inspect(x:Transaction){","x.",vec!["query","execute","commit","rollback"]),
 ("import {connect} from 'std:sql';async function main():Task<void>{const db=await connect('sqlite::memory:',1);const rows=await db.query('SELECT 1 AS id');","rows[0].",vec!["int","number","boolean","text","blob","isNull"])
 ]{
 let source=format!("{before}{receiver};}}");let offset=before.len()+receiver.len();let uri="file:///async-sql.tsr";let id=source_id(uri).unwrap();let mut workspace=Workspace::default();workspace.open(id.clone(),1,source);let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&id.0).unwrap();let result=ProjectIndex::new(&graph).request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
 for member in members{assert!(result.as_array().unwrap().iter().any(|entry|entry["label"]==member),"{receiver}: {result}");}
 }
}
#[test]
fn sql_catalog_documents_defaults_and_lifecycle(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let connect=&catalog["std:sql"][0];assert!(connect["signature"].as_str().unwrap().contains("Task<Pool>"));assert!(connect["documentation"].as_str().unwrap().contains("30000"));
 for family in ["SqlPool","SqlTransaction","SqlRow"]{for member in catalog[family].as_array().unwrap(){assert!(member["documentation"].as_str().unwrap().contains("Ownership:"));assert!(!member["examples"].as_array().unwrap().is_empty());}}
}

#[test]
fn connect_hover_and_signature_show_lazy_task_and_optional_defaults(){
 let source="import {connect as open} from 'std:sql';async function main():Task<void>{const db=await open('sqlite::memory:');await db.close();}";
 let index=tsrust_lsp::intelligence::SourceIndex::new(source);
 for (method,offset) in [("textDocument/hover",source.find("await open").unwrap()+7),("textDocument/signatureHelp",source.find("open('").unwrap()+5)] {
  let response=index.request(method,&json!({"position":{"line":0,"character":offset}}),"file:///sql.tsr").unwrap().to_string();
  assert!(response.contains("Task<Pool>"),"{response}");assert!(response.contains("acquireTimeout?"),"{response}");assert!(response.contains("30000"),"{response}");
 }
}
