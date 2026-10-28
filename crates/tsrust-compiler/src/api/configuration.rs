//! Application-owned API configuration, shared by compilation and SDK export.
use serde::Deserialize;
use std::path::{Component,Path};
use super::Selection;
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Configuration {
 pub schema_version:u32,pub entry:String,#[serde(default)] pub output:Option<String>,
 pub package:Package,pub operations:Vec<Selection>,
 #[serde(default="api_version")] pub api_version:u32,
 #[serde(default)] pub compatible:Vec<String>,#[serde(default)] pub server:Option<Server>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Server {
 #[serde(default)] pub output:Option<String>,pub context:Import,pub authorize:Import,
 #[serde(default)] pub compatible:Vec<String>,
 #[serde(default="byte_limit")] pub max_request_bytes:u32,
 #[serde(default="byte_limit")] pub max_response_bytes:u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {pub module:String,pub export:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {pub name:String,pub version:String}
fn api_version()->u32{1}
fn byte_limit()->u32{65536}
pub fn relative(value:&str)->Result<&Path,String>{
 let path=Path::new(value);
 if value.is_empty()||value.contains('\\')||path.components().any(|part|!matches!(part,Component::Normal(_))){return Err(format!("API configuration path {value:?} must be relative without '.', '..' or backslashes"));}
 Ok(path)
}
impl Configuration {
 pub fn inline(value:&serde_json::Value)->Result<Self,String>{
  let mut value=value.as_object().cloned().ok_or("api must be an object")?;
  value.entry("schemaVersion".to_owned()).or_insert(1.into());
  let config:Self=serde_json::from_value(serde_json::Value::Object(value)).map_err(|e|format!("Invalid api configuration: {e}"))?;
  config.validate()?;Ok(config)
 }
 pub fn validate(&self)->Result<(),String>{
  if self.schema_version!=1{return Err("API configuration schemaVersion must be 1".into());}
  if self.api_version==0{return Err("API compatibility versions start at 1".into());}
  relative(&self.entry)?;if let Some(path)=&self.output{relative(path)?;}
  for operation in &self.operations{relative(&operation.module)?;}
  for path in &self.compatible{relative(path)?;}
  if let Some(server)=&self.server{
   if let Some(path)=&server.output{relative(path)?;}
   relative(&server.context.module)?;relative(&server.authorize.module)?;
   for path in &server.compatible{relative(path)?;}
   for limit in [server.max_request_bytes,server.max_response_bytes]{if !(1..=16777216).contains(&limit){return Err("API byte limits must be from 1 to 16777216".into());}}
  }
  Ok(())
 }
}
