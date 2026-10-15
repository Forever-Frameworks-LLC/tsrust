//! Explicit Cargo inputs for handwritten Rust binding modules.
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,path::{Path,PathBuf}};
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Bindings {pub schema_version:u32,pub source:String,pub dependencies:BTreeMap<String,Dependency>,#[serde(default,skip_serializing_if="Option::is_none")]pub lockfile:Option<String>}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub struct Dependency {pub version:String,#[serde(default)]pub features:Vec<String>,#[serde(default="enabled")]pub default_features:bool}
fn enabled()->bool{true}
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Package {pub manifest:String,pub lock:String,pub vendor:PathBuf,pub crates:Vec<String>}
impl Bindings {
 pub fn parse(text:&str)->Result<Self,String>{
  let value:Self=serde_json::from_str(text).map_err(|error|error.to_string())?;
  if value.schema_version!=1 || !value.source.ends_with(".rs"){return Err("Bindings require schemaVersion 1 and a relative .rs source.".into());}
  if Path::new(&value.source).is_absolute(){return Err("Binding source must be relative.".into());}
  for (name,dependency) in &value.dependencies {
   if name.is_empty() || !name.bytes().all(|byte|byte.is_ascii_alphanumeric() || b"_-".contains(&byte)) || syn::parse_str::<syn::Ident>(&name.replace('-',"_")).is_err() || matches!(name.as_str(),"rayon"|"serde"|"serde_json"|"tokio"|"axum"|"reqwest"|"http-body"|"http-body-util"|"hyper-util"|"rusqlite"|"libsqlite3-sys"|"tokio-rustls"|"tsrust_runtime") {return Err(format!("Invalid or reserved dependency name {name}."));}
   if !dependency.version.starts_with('=') || dependency.version.len()<2 || !dependency.version[1..].bytes().all(|byte|byte.is_ascii_alphanumeric() || b".-+".contains(&byte)) || {let core=dependency.version[1..].split(['-','+']).next().unwrap();let parts=core.split('.').collect::<Vec<_>>();parts.len()!=3 || parts.iter().any(|part|part.is_empty() || !part.bytes().all(|byte|byte.is_ascii_digit()))}{return Err(format!("Dependency {name} requires an exact version, such as =1.2.3."));}
   if dependency.features.iter().any(|feature|feature.is_empty() || !feature.bytes().all(|byte|byte.is_ascii_alphanumeric() || b"_-/".contains(&byte))){return Err(format!("Invalid feature for {name}."));}
  }Ok(value)
 }
 pub fn manifest(&self)->String {
  let dependencies=self.dependencies.iter().map(|(name,dependency)|format!("{name} = {{ version = {:?}, features = [{}], default-features = {} }}\n",dependency.version,dependency.features.iter().map(|feature|format!("{feature:?}")).collect::<Vec<_>>().join(","),dependency.default_features)).collect::<String>();
  include_str!("../../native-dependencies/Cargo.toml").replace("\n[features]",&format!("\n{dependencies}\n[features]"))
 }
 pub fn lock_path(path:&Path)->PathBuf{path.with_extension("lock")}
 pub fn package(&self,path:&Path,lock:String)->Package {
  use std::hash::{Hash,Hasher};let manifest=self.manifest();let mut hash=std::collections::hash_map::DefaultHasher::new();manifest.hash(&mut hash);lock.hash(&mut hash);
  Package{manifest,lock,vendor:path.parent().unwrap().join("target/tsrust-native/bindings").join(format!("{:016x}",hash.finish())).join("vendor"),crates:self.dependencies.keys().map(|name|name.replace('-',"_")).collect()}
 }
}
pub const MARKER:&str="// tsrust-binding-package ";
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn binding_dependencies_are_explicit_exact_and_closed(){
  let value=Bindings::parse(r#"{"schemaVersion":1,"source":"./adapter.rs","dependencies":{"regex":{"version":"=1.12.2","features":["unicode"],"defaultFeatures":false}}}"#).unwrap();
  assert!(value.manifest().contains("regex = { version = \"=1.12.2\", features = [\"unicode\"], default-features = false }"));
  for version in ["=1","=1.2","=","=1.2.x"]{let mut invalid=value.clone();invalid.dependencies.get_mut("regex").unwrap().version=version.into();assert!(Bindings::parse(&serde_json::to_string(&invalid).unwrap()).is_err());}
  for text in [r#"{"schemaVersion":2,"source":"./a.rs","dependencies":{}}"#,r#"{"schemaVersion":1,"source":"./a.rs","dependencies":{"regex":{"version":"1"}}}"#,r#"{"schemaVersion":1,"source":"./a.rs","dependencies":{"regex":{"version":"=1","path":"../escape"}}}"#]{assert!(Bindings::parse(text).is_err());}
 }
}
