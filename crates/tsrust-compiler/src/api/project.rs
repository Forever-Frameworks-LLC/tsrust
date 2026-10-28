//! Read-only compiler dispatch. Client generation is never a server prerequisite.
use std::path::Path;
use crate::{Span,project::{SourceId,SourceProvider,SourceFile,ProjectDiagnostic,SourceLocation}};
use super::{configuration::{Configuration,relative},CheckedApi,ApiHistory,Manifest,NativeImport,NativeServer};
pub(crate) const IMPORT:&str="api:server";
pub(crate) struct ServerSource {
 pub source:SourceFile,
 pub operations:crate::project::ProjectGraph,
}

fn error(id:&SourceId,message:impl Into<String>)->ProjectDiagnostic{
 ProjectDiagnostic{code:"api-config",message:message.into(),location:SourceLocation{source:id.clone(),span:Span::default()},related:vec![],fixes:Box::new([])}
}
fn identity(config:&SourceId)->SourceId {
 SourceId(Path::new(&config.0).parent().expect("configuration parent").join("target/tsrust-api/server.tsr").to_string_lossy().into_owned())
}
/// This path is an in-memory compiler source identity, never an emitted file.
pub(crate) fn config_for_virtual(id:&SourceId)->Option<SourceId>{
 let path=Path::new(&id.0);let root=path.parent()?.parent()?.parent()?;
 (path==root.join("target/tsrust-api/server.tsr")).then(||SourceId(root.join("tsrust.config.json").to_string_lossy().into_owned()))
}
pub(crate) fn discover(provider:&impl SourceProvider,importer:&SourceId)->Result<SourceId,ProjectDiagnostic>{
 for root in Path::new(&importer.0).parent().ok_or_else(||error(importer,"API imports require an application source"))?.ancestors(){
  let id=SourceId(root.join("tsrust.config.json").to_string_lossy().into_owned());
  match provider.read(&id){
   Ok(_)=>return Ok(id),
   Err(message)=>match std::fs::symlink_metadata(&id.0){
    Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
    // A failed read of an existing config is not permission to select a
    // different project's authorization policy. Unsaved provider configs
    // remain supported by the successful-read branch above.
    _=>return Err(error(&id,message)),
   }
  }
 }
 Err(error(importer,"api:server requires api settings in tsrust.config.json"))
}
pub(crate) fn source(provider:&impl SourceProvider,config_id:&SourceId)->Result<ServerSource,ProjectDiagnostic>{
 let text=provider.read(config_id).map_err(|e|error(config_id,e))?;
 let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|error(config_id,e.to_string()))?;
 let config=Configuration::inline(value.get("api").ok_or_else(||error(config_id,"api:server requires api settings"))?).map_err(|e|error(config_id,e))?;
 let root=Path::new(&config_id.0).parent().expect("configuration parent");
 let resolve=|path:&str|->Result<SourceId,ProjectDiagnostic>{
  let relative=relative(path).map_err(|e|error(config_id,e))?;
  provider.resolve(None,&root.join(relative).to_string_lossy()).map_err(|e|error(config_id,e))
 };
 let entry=resolve(&config.entry)?;
 // API functions must not depend on their own dispatcher, including indirectly.
 // The separate bootstrap forbids this import before invoking another bootstrap.
 let graph=crate::project::load_api_operations(provider,&entry.0)?;
 let checked=crate::project::check_graph(graph)?;
 let mut selections=config.operations;
 for selection in &mut selections {selection.module=resolve(&selection.module)?.0;}
 let api=CheckedApi::from_project(&checked,&selections).map_err(|e|error(config_id,e))?;
 let server=config.server.ok_or_else(||error(config_id,"api:server requires server.context and server.authorize"))?;
 let mut history=ApiHistory::new(&config.package.name);
 for path in config.compatible.iter().chain(&server.compatible){
  let id=resolve(path)?;let text=provider.read(&id).map_err(|e|error(&id,e))?;
  let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|error(&id,e.to_string()))?;
  if value.get("releases").is_some(){
   let retained:ApiHistory=serde_json::from_value(value).map_err(|e|error(&id,e.to_string()))?;
   history.merge(&retained).map_err(|e|error(&id,e))?;
  }else{
   let retained:Manifest=serde_json::from_value(value).map_err(|e|error(&id,e.to_string()))?;
   history.retain(1,retained).map_err(|e|error(&id,e))?;
  }
 }
 history=history.advance(&config.package.name,&config.package.version,config.api_version,&api.manifest).map_err(|e|error(config_id,e))?;
 let context=NativeImport{module:resolve(&server.context.module)?.0,export:server.context.export};
 let authorize=NativeImport{module:resolve(&server.authorize.module)?.0,export:server.authorize.export};
 let id=identity(config_id);
 let options=NativeServer{context,authorize,previous:vec![],max_request_bytes:server.max_request_bytes,max_response_bytes:server.max_response_bytes};
 let text=api.native_source_versioned(Path::new(&id.0),&options,&history).map_err(|e|error(config_id,e))?;
 Ok(ServerSource{source:SourceFile{id,text},operations:checked.graph})
}
