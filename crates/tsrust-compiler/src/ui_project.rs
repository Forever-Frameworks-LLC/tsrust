//! UI authoring configuration and validation shared by the CLI and editor.
use std::path::{Component,Path,PathBuf};
use serde::{Deserialize,Serialize};
pub use tsrust_ui::{theme,style};
#[derive(Clone,Debug,Default,Deserialize,Serialize,PartialEq,Eq)]
#[serde(default,deny_unknown_fields)]
pub struct Targets {pub native:bool,pub web:bool,pub embedded:bool}
impl Targets {
 pub fn validate(&self)->Result<(),String>{
  if self.embedded{return Err("ui.target.embedded is unsupported: no embedded renderer or hardware profile is implemented".into());}
  if !self.native && !self.web{return Err("ui.target must enable native or web".into());}Ok(())
 }
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Config {#[serde(default,skip_serializing_if="Option::is_none")] pub target:Option<Targets>,pub theme:serde_json::Value,pub components:PathBuf,pub lock:PathBuf}
impl Default for Config {fn default()->Self {Self{target:Some(Targets{native:true,..Targets::default()}),theme:serde_json::from_str(theme::DEFAULT_JSON).expect("bundled theme"),components:"src/components/ui".into(),lock:"ui-components.lock.json".into()}}}
pub fn safe_relative(path:&Path)->Result<(),String> {
 if path.as_os_str().is_empty() || path.components().any(|part|!matches!(part,Component::Normal(_))) || path.to_str().is_none_or(|s|s.contains(['\\','\0'])) {return Err(format!("UI path must be a nonempty relative path without '.' or '..': {}",path.display()));}Ok(())
}
impl Config {
 pub fn parse(value:&serde_json::Value)->Result<Self,String> {
  let config=Self::paths(value)?;
  if let Some(target)=&config.target{target.validate()?;}
  if config.external_theme().is_none(){theme::Theme::parse(&config.theme.to_string()).map_err(|e|format!("ui.theme: {e}"))?;}
  Ok(config)
 }
 pub(crate) fn paths(value:&serde_json::Value)->Result<Self,String> {
  let config:Self=serde_json::from_value(value.clone()).map_err(|e|format!("Invalid ui configuration: {e}"))?;
  for path in [&config.components,&config.lock] {safe_relative(path)?;}
  if config.lock.extension().is_none_or(|v|v!="json") || config.lock==Path::new("tsrust.config.json") || config.lock.starts_with(&config.components) || Path::new("tsrust.config.json").starts_with(&config.components) {return Err("UI lock must be a separate JSON file outside the components directory".into());}
  if let Some(path)=config.external_theme() {
   safe_relative(path)?;
   if path.extension().is_none_or(|v|v!="json") || path==config.lock || path==Path::new("tsrust.config.json") || path.starts_with(&config.components) {return Err("External UI theme must be a separate JSON file outside the components directory".into());}
  } else if !config.theme.is_object(){return Err("ui.theme must be a theme object or a legacy relative JSON path".into());}
  Ok(config)
 }
 pub fn external_theme(&self)->Option<&Path> {self.theme.as_str().map(Path::new)}
 pub fn theme_destination(&self)->&Path {self.external_theme().unwrap_or(Path::new("tsrust.config.json"))}
}
/// Theme data and its JSON field prefix, shared by validation and source navigation.
pub fn theme_document(path:&Path,text:&str)->Option<(serde_json::Value,Vec<String>)> {
 let value:serde_json::Value=serde_json::from_str(text).ok()?;
 if path.file_name().is_some_and(|v|v=="tsrust.config.json") {
  let theme=value.get("ui")?.get("theme")?;if !theme.is_object(){return None;}
  Some((theme.clone(),vec!["ui".into(),"theme".into()]))
 } else if value.get("light").is_some() && value.get("dark").is_some(){Some((value,vec![]))}else{None}
}
/// Configuration reads use the same provider snapshot as imports, including unsaved edits.
pub fn configured_theme(path:&Path,mut read:impl FnMut(&Path)->Result<Option<String>,String>)->Result<bool,String> {
 for root in path.parent().unwrap_or(Path::new(".")).ancestors() {
  if let Some(text)=read(&root.join("tsrust.config.json"))? {
   let config:serde_json::Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
   return config.get("ui").map(Config::parse).transpose().map(|value|value.is_some_and(|ui|root.join(ui.theme_destination())==path));
  }
 }
 Ok(path.file_name().is_some_and(|v|v=="ui.theme.json"))
}

/// Authoring references of a validated utility, independent of editor token tables.
pub fn utility_theme_paths(class:&str)->Vec<Vec<String>> {style::theme_paths(class)}

pub const RUNTIME_ABI:&str="ui-application-1";
pub const COMPILER_API:u64=11;
pub fn supports_compiler_api(value:u64)->bool {(1..=COMPILER_API).contains(&value)}
/// Installed compatibility pins apply during checks/builds, not only when adding source.
pub fn validate_lock(root:&Path,config:&Config,read:&mut impl FnMut(&Path)->Result<Option<String>,String>)->Result<(),String> {
 let path=root.join(&config.lock);let Some(text)=read(&path)? else{return Ok(());};
 let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|format!("{}: {e}",path.display()))?;
 if value.get("schemaVersion").and_then(serde_json::Value::as_u64)!=Some(1){return Err(format!("{}: unsupported UI lock schemaVersion",path.display()));}
 let entries=value.get("installed").and_then(serde_json::Value::as_array).ok_or("UI lock installed must be an array")?;
 for entry in entries {
  if !entry.get("compilerApi").and_then(serde_json::Value::as_u64).is_some_and(supports_compiler_api) || entry.get("runtimeAbi").and_then(serde_json::Value::as_str)!=Some(RUNTIME_ABI) || entry.get("rust").and_then(serde_json::Value::as_str)!=Some(crate::RUST_BACKEND_VERSION) {return Err(format!("{}: installed UI source requires another compiler API, runtime ABI or Rust toolchain; use its recorded supported toolchain or migrate explicitly",path.display()));}
  if !entry.get("compilerSha256").and_then(serde_json::Value::as_str).is_some_and(|hash|hash.len()==64 && hash.bytes().all(|v|v.is_ascii_hexdigit())) {return Err(format!("{}: missing full compiler SHA-256 provenance",path.display()));}
 }
 Ok(())
}

/// Header imports use the module parser, so comments and string contents cannot trigger setup.
pub fn source_imports(source:&str)->Result<Vec<String>,String> {
 let source=crate::project::SourceFile{id:crate::project::SourceId("ui-import-probe.tsr".into()),text:source.into()};
 crate::project::parse_module(source,false).map(|module|module.imports.into_iter().map(|import|import.path).collect()).map_err(|error|error.message)
}

/// Rewrite only parsed module specifiers; comments, application strings and JSX text stay intact.
pub fn rewrite_source_imports(source:&str,mut replacement:impl FnMut(&str)->Result<Option<String>,String>)->Result<String,String> {
 let input=crate::project::SourceFile{id:crate::project::SourceId("ui-import-rewrite.tsrx".into()),text:source.into()};
 let module=crate::project::parse_module(input,false).map_err(|error|error.message)?;
 let mut result=source.to_owned();
 for import in module.imports.into_iter().rev(){if let Some(path)=replacement(&import.path)? {
  result.replace_range(import.span.start..import.span.end,&serde_json::to_string(&path).map_err(|e|e.to_string())?);
 }}
 Ok(result)
}

/// Deployment checks consume the compiler's selected native features, not source-name guesses.
pub fn configured_targets(entry:&Path,mut read:impl FnMut(&Path)->Result<Option<String>,String>)->Result<Option<Targets>,String>{
 for root in entry.parent().unwrap_or(Path::new(".")).ancestors(){if let Some(text)=read(&root.join("tsrust.config.json"))?{
  let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|e.to_string())?;
  return value.get("ui").map(Config::parse).transpose().map(|config|config.and_then(|config|config.target));
 }}Ok(None)
}
pub fn require_target(entry:&Path,target:&str,read:impl FnMut(&Path)->Result<Option<String>,String>)->Result<(),String>{
 let selected=configured_targets(entry,read)?.ok_or("UI deployment needs an explicit ui.target. Run tsrust ui add tokens@1.0.0 with --target native or --target web")?;
 if match target{"native"=>selected.native,"web"=>selected.web,_=>false}{return Ok(());}
 Err(format!("UI {target} target is disabled in tsrust.config.json ui.target. Enable it explicitly to use this backend. Use tsrust dev --target web for browser development or --target native for a desktop window."))
}
