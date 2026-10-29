//! Stand-in for the frozen UI commands when the legacy-ui feature is disabled.
//! The bundled component catalog and its notices stay out of release builds.
use std::path::{Path,PathBuf};
use tsrust_compiler::ui_project::{Config,Targets,safe_relative};

pub(super) const UNAVAILABLE:&str="UI commands belong to the frozen legacy UI and are not part of this backend release. Tsrust applications expose HTTP APIs; bring your own UI.";

pub(super) fn portable_generated_path(path:&Path)->Result<PathBuf,String> {
 let path=PathBuf::from(path.to_str().ok_or("Invalid generated UI path")?.replace('\\',"/"));
 safe_relative(&path)?;Ok(path)
}
pub(super) fn run(_args:impl Iterator<Item=String>)->Result<u8,String> {Err(UNAVAILABLE.into())}
pub(super) fn browse()->Result<Option<Vec<String>>,String> {Err(UNAVAILABLE.into())}
pub(super) fn install_defaults(_root:&Path,_config:&Config)->Result<(),String> {Err(UNAVAILABLE.into())}
pub(super) fn select_targets(_existing:Option<&Targets>,_flags:&[String],_noninteractive:bool)->Result<Targets,String> {Err(UNAVAILABLE.into())}
/// Release builds install no bundled components, so projects carry no component notices.
pub(super) fn attribution_files(_entry:&Path)->Result<Vec<(PathBuf,Vec<u8>)>,String> {Ok(vec![])}
pub(super) fn check_setup(_entry:&Path)->Result<(),String> {Ok(())}
pub(super) fn check_default_setup()->Result<(),String> {Ok(())}
