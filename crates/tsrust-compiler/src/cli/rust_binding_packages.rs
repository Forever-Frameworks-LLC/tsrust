//! Locked graph inputs shared by ordinary builds and explicit portable packs.
use tsrust_compiler::project::rust::dependencies::{Package,MARKER};
pub(crate) fn packages(source:&str)->Result<Vec<Package>,String>{
 source.lines().filter_map(|line|line.strip_prefix(MARKER)).map(|json|serde_json::from_str(json).map_err(|error|format!("Invalid binding package: {error}"))).collect()
}
pub(crate) fn checked(packages:&[Package])->Result<Option<&Package>,String>{
 let Some(package)=packages.first() else{return Ok(None)};
 if packages.iter().any(|other|other.manifest!=package.manifest || other.lock!=package.lock){return Err("All binding modules in one program must share the same dependency versions and features.".into());}
 Ok(Some(package))
}
