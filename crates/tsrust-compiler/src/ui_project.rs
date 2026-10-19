//! UI authoring configuration and validation shared by the CLI and editor.
use std::path::{Component,Path,PathBuf};
use serde::{Deserialize,Serialize};
pub use tsrust_ui::{theme,style};
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Config {pub theme:PathBuf,pub components:PathBuf,pub lock:PathBuf}
impl Default for Config {fn default()->Self {Self{theme:"ui.theme.json".into(),components:"components/ui".into(),lock:"ui-components.lock.json".into()}}}
pub fn safe_relative(path:&Path)->Result<(),String> {
 if path.as_os_str().is_empty() || path.components().any(|part|!matches!(part,Component::Normal(_))) || path.to_str().is_none_or(|s|s.contains(['\\','\0'])) {return Err(format!("UI path must be a nonempty relative path without '.' or '..': {}",path.display()));}Ok(())
}
impl Config {
 pub fn parse(value:&serde_json::Value)->Result<Self,String> {
  let config:Self=serde_json::from_value(value.clone()).map_err(|e|format!("Invalid ui configuration: {e}"))?;
  for path in [&config.theme,&config.components,&config.lock] {safe_relative(path)?;}
  if config.theme.extension().is_none_or(|v|v!="json") || config.lock.extension().is_none_or(|v|v!="json") || config.theme==config.lock || config.theme.starts_with(&config.components) || config.lock.starts_with(&config.components) {return Err("UI theme and lock must be distinct JSON files outside the components directory".into());}Ok(config)
 }
}
/// Configuration reads use the same provider snapshot as imports, including unsaved edits.
pub fn configured_theme(path:&Path,mut read:impl FnMut(&Path)->Result<Option<String>,String>)->Result<bool,String> {
 for root in path.parent().unwrap_or(Path::new(".")).ancestors() {
  if let Some(text)=read(&root.join("tsrust.config.json"))? {
   let config:serde_json::Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
   return config.get("ui").map(Config::parse).transpose().map(|value|value.is_some_and(|ui|root.join(ui.theme)==path));
  }
 }
 Ok(path.file_name().is_some_and(|v|v=="ui.theme.json"))
}

/// Authoring references of a validated utility, independent of editor token tables.
pub fn utility_theme_paths(class:&str)->Vec<Vec<String>> {
 if style::validate(class).is_err(){return vec![];}
 let token=class.rsplit(':').next().unwrap();
 let color=if token=="ring" {Some("ring")}else{token.strip_prefix("bg-").or_else(||token.strip_prefix("text-")).or_else(||token.strip_prefix("border-"))};
 if let Some(color)=color.filter(|color|theme::COLORS.contains(color)) {return ["light","dark"].into_iter().map(|mode|vec![mode.into(),theme::key_name(color)]).collect();}
 let field=if token=="rounded" {Some(("radius","md"))}else if token=="rounded-sm" {Some(("radius","sm"))}else if token=="rounded-lg" {Some(("radius","lg"))}else if let Some(size)=token.strip_prefix("text-").filter(|size|matches!(*size,"base"|"xl")) {Some(("typography",size))}else if let Some((prefix,size))=token.split_once('-') && matches!(prefix,"p"|"px"|"py"|"gap") {match size {"2"=>Some(("spacing","sm")),"4"=>Some(("spacing","md")),"6"=>Some(("spacing","lg")),_=>None}}else{None};
 field.into_iter().map(|(group,key)|vec![group.into(),key.into()]).collect()
}

pub const RUNTIME_ABI:&str="ui-application-1";
pub const COMPILER_API:u64=1;
/// Installed compatibility pins apply during checks/builds, not only when adding source.
pub fn validate_lock(root:&Path,config:&Config,read:&mut impl FnMut(&Path)->Result<Option<String>,String>)->Result<(),String> {
 let path=root.join(&config.lock);let Some(text)=read(&path)? else{return Ok(());};
 let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|format!("{}: {e}",path.display()))?;
 if value.get("schemaVersion").and_then(serde_json::Value::as_u64)!=Some(1){return Err(format!("{}: unsupported UI lock schemaVersion",path.display()));}
 let entries=value.get("installed").and_then(serde_json::Value::as_array).ok_or("UI lock installed must be an array")?;
 for entry in entries {
  if entry.get("compilerApi").and_then(serde_json::Value::as_u64)!=Some(COMPILER_API) || entry.get("runtimeAbi").and_then(serde_json::Value::as_str)!=Some(RUNTIME_ABI) || entry.get("rust").and_then(serde_json::Value::as_str)!=Some(crate::RUST_BACKEND_VERSION) {return Err(format!("{}: installed UI source requires another compiler API, runtime ABI or Rust toolchain; use its recorded supported toolchain or migrate explicitly",path.display()));}
  if !entry.get("compilerSha256").and_then(serde_json::Value::as_str).is_some_and(|hash|hash.len()==64 && hash.bytes().all(|v|v.is_ascii_hexdigit())) {return Err(format!("{}: missing full compiler SHA-256 provenance",path.display()));}
 }
 Ok(())
}
