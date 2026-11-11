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
// Runtime-owned pins cannot be redeclared or replaced by a binding package.
pub fn reserved(name:&str)->bool {
 [
        "arboard",
        "color",
        "url",
        "regex",
        "icu_provider","icu_collator","icu_decimal","icu_locale","icu_segmenter","fixed_decimal",
        "percent-encoding",
        "winit",
        "vello",
        "kurbo",
        "svgtypes",
        "taffy",
        "parley",
        "pollster",
        "accesskit",
        "accesskit_winit",
        "futures-core",
        "futures-util","tokio-tungstenite","webpki-roots","pin-project-lite","tracing","tracing-subscriber",
        "tokio-util",
        "flate2",
        "nix",
        "base64",
        "tempfile",
        "memchr",
        "hostname",
        "sha2","subtle","hmac","hkdf","argon2","zeroize","rand","uuid","getrandom","chrono-tz","chrono","bigdecimal","sqlx","rusqlite","axum","tower-http","matchit","tokio-rustls","reqwest","form_urlencoded","tower","async-compression","compression-codecs","compression-core","http-body","http-body-util","hyper-util","tokio","rayon","serde","serde_json","libsqlite3-sys","tsrust_runtime",
    ].contains(&name)
}
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Package {pub manifest:String,pub lock:String,pub crates:Vec<String>}
impl Bindings {
 pub fn parse(text:&str)->Result<Self,String>{
  let value:Self=serde_json::from_str(text).map_err(|error|error.to_string())?;
  if !matches!(value.schema_version,1|2) || !value.source.ends_with(".rs"){return Err("Bindings require schemaVersion 1 (legacy) or 2 (backend) and a relative .rs source.".into());}
  if Path::new(&value.source).is_absolute(){return Err("Binding source must be relative.".into());}
  for (name,dependency) in &value.dependencies {
   if name.is_empty() || !name.bytes().all(|byte|byte.is_ascii_alphanumeric() || b"_-".contains(&byte)) || syn::parse_str::<syn::Ident>(&name.replace('-',"_")).is_err() || (reserved(name) && name!="regex") {return Err(format!("Invalid or reserved dependency name {name}."));}
   if !dependency.version.starts_with('=') || dependency.version.len()<2 || !dependency.version[1..].bytes().all(|byte|byte.is_ascii_alphanumeric() || b".-+".contains(&byte)) || {let core=dependency.version[1..].split(['-','+']).next().unwrap();let parts=core.split('.').collect::<Vec<_>>();parts.len()!=3 || parts.iter().any(|part|part.is_empty() || !part.bytes().all(|byte|byte.is_ascii_digit()))}{return Err(format!("Dependency {name} requires an exact version, such as =1.2.3."));}
   if name=="regex" && (dependency.version!="=1.13.1" || !dependency.default_features || !dependency.features.is_empty()) {return Err("Regex bindings must reuse the bundled =1.13.1 default-feature backend without extra features.".into());}
   if dependency.features.iter().any(|feature|feature.is_empty() || !feature.bytes().all(|byte|byte.is_ascii_alphanumeric() || b"_-/".contains(&byte))){return Err(format!("Invalid feature for {name}."));}
  }Ok(value)
 }
 pub fn manifest(&self)->String {
  let dependencies=self.dependencies.iter().filter(|(name,_)|name.as_str()!="regex").map(|(name,dependency)|format!("{name} = {{ version = {:?}, features = [{}], default-features = {} }}\n",dependency.version,dependency.features.iter().map(|feature|format!("{feature:?}")).collect::<Vec<_>>().join(","),dependency.default_features)).collect::<String>();
  // The schema fixes the runtime graph; never infer it from incidental lock contents.
  let base=if self.schema_version==1{include_str!("../../native-dependencies/Cargo.toml")}else{include_str!("../../backend-dependencies/Cargo.toml")};
  // A declared regex binding uses the admitted backend and enables it for Rust source.
  let base=if self.dependencies.contains_key("regex"){base.replace("regex = { version = \"=1.13.1\", optional = true }","regex = { version = \"=1.13.1\" }").replace("regex = [\"dep:regex\"]","regex = []")}else{base.to_owned()};
  base.replace("\n[features]",&format!("\n{dependencies}\n[features]"))
 }
 pub fn lock_path(path:&Path)->PathBuf{path.with_extension("lock")}
 pub fn package(&self,lock:String)->Package {
  Package{manifest:self.manifest(),lock,crates:self.dependencies.keys().map(|name|name.replace('-',"_")).collect()}
 }
}
pub const MARKER:&str="// tsrust-binding-package ";
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn every_bundled_dependency_pin_is_reserved(){
  for manifest in [include_str!("../../native-dependencies/Cargo.toml"),include_str!("../../backend-dependencies/Cargo.toml")] {
   let section=manifest.split_once("[dependencies]\n").unwrap().1.split_once("\n[features]").unwrap().0;
   for line in section.lines().filter(|line|!line.trim().is_empty()) {let name=line.split_once('=').unwrap().0.trim();assert!(reserved(name),"unprotected runtime dependency {name}");
    // Both schemas share the same runtime-owned names, even when a backend
    // feature is optional. Regex alone has its separately tested admitted reuse.
    if name!="regex" {for schema in [1,2] {
     let text=format!(r#"{{"schemaVersion":{schema},"source":"./adapter.rs","dependencies":{{"{name}":{{"version":"=0.0.0"}}}}}}"#);
     let error=Bindings::parse(&text).unwrap_err();assert!(error.contains("Invalid or reserved dependency name"),"{schema}: {name}: {error}");
    }}}
  }
  for schema in [1,2] {for name in ["nix","flate2"] {assert!(Bindings::parse(&format!(r#"{{"schemaVersion":{schema},"source":"./adapter.rs","dependencies":{{"{name}":{{"version":"=0.30.1"}}}}}}"#)).is_err());}}
 }

 #[test]fn binding_graphs_are_path_independent_and_preserve_legacy_locks(){
  let binding=Bindings::parse(r#"{"schemaVersion":2,"source":"./adapter.rs","dependencies":{}}"#).unwrap();
  assert!(!binding.manifest().contains("vello ="));
  let mut legacy_binding=binding.clone();legacy_binding.schema_version=1;
  let legacy=legacy_binding.package(include_str!("../../native-dependencies/Cargo.lock").into());
  // Schema 1 keeps the native dependency set and its recorded lock.
  assert!(legacy.manifest.contains("rusqlite =")&&!legacy.manifest.contains("vello ="));
  assert_eq!(legacy.lock,include_str!("../../native-dependencies/Cargo.lock"));
  assert!(!serde_json::to_string(&legacy).unwrap().contains("vendor"));
 }

 #[test]fn binding_dependencies_are_explicit_exact_and_closed(){
  let value=Bindings::parse(r#"{"schemaVersion":1,"source":"./adapter.rs","dependencies":{"regex":{"version":"=1.13.1","features":[],"defaultFeatures":true}}}"#).unwrap();
  let manifest=value.manifest().parse::<toml_edit::DocumentMut>().unwrap();
  assert_eq!(manifest["dependencies"]["regex"]["version"].as_str(),Some("=1.13.1"));
  assert!(manifest["dependencies"]["regex"].get("optional").is_none());
  for (version,defaults,features) in [("=1.12.2",true,vec![]),("=1.13.1",false,vec![]),("=1.13.1",true,vec!["unicode".into()])] {
   let mut invalid=value.clone();let dependency=invalid.dependencies.get_mut("regex").unwrap();dependency.version=version.into();dependency.default_features=defaults;dependency.features=features;
   assert!(Bindings::parse(&serde_json::to_string(&invalid).unwrap()).is_err());
  }
  for version in ["=1","=1.2","=","=1.2.x"]{let mut invalid=value.clone();invalid.dependencies.get_mut("regex").unwrap().version=version.into();assert!(Bindings::parse(&serde_json::to_string(&invalid).unwrap()).is_err());}
  for text in [r#"{"schemaVersion":3,"source":"./a.rs","dependencies":{}}"#,r#"{"schemaVersion":1,"source":"./a.rs","dependencies":{"regex":{"version":"1"}}}"#,r#"{"schemaVersion":1,"source":"./a.rs","dependencies":{"regex":{"version":"=1","path":"../escape"}}}"#]{assert!(Bindings::parse(text).is_err());}
 }
}
