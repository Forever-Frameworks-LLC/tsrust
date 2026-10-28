//! Explicit prebuilt CLI/LSP management, separate from Rust setup and crate updates.
use std::path::PathBuf;
use super::{toolchain_install as install,toolchain_release as release};
const USAGE:&str="Usage: tsrust toolchain install --root NEW_DIRECTORY | check [--json] | update | rollback";
fn managed()->Result<PathBuf,String>{let root=install::current_root()?.ok_or_else(||"This executable is not managed by the Tsrust installer. Upgrade it with its package manager, or run `tsrust toolchain install --root NEW_DIRECTORY` and put that installation's bin directory on PATH. Existing installations are never overwritten.".to_owned())?;install::state(&root)?;Ok(root)}
pub(super) fn run(mut args:impl Iterator<Item=String>)->Result<u8,String>{
 let command=args.next().ok_or(USAGE)?;let arguments=args.collect::<Vec<_>>();
 if command=="rollback"{
  if !arguments.is_empty(){return Err(USAGE.into());}let root=managed()?;let _lock=install::lock(&root)?;
  let state=install::rollback(&root)?;println!("Selected Tsrust {} and its paired LSP. Restart editor servers to use this pair.",state.current.release.version);return Ok(0);
 }
 let root=match command.as_str(){
  "install" if arguments.len()==2&&arguments[0]=="--root"=>{let root=super::absolute(std::path::Path::new(&arguments[1]))?;if root.exists(){return Err("Installation requires a new directory; existing files are preserved".into());}Some(root)},
  "update" if arguments.is_empty()=>Some(managed()?),
  "check" if arguments.is_empty()||arguments==["--json"]=>install::current_root()?,
  _=>return Err(USAGE.into())
 };
 let _lock=if command=="update"{Some(install::lock(root.as_ref().unwrap())?)}else{None};
 let current=if command=="install"{None}else{match root.as_ref(){Some(root)=>Some(install::state(root)?.current.release.version),None=>Some(env!("CARGO_PKG_VERSION").into())}};
 let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e|e.to_string())?;
 let latest=runtime.block_on(release::latest())?;
 let newer=current.as_ref().map(|version|Ok::<_,String>(release::version(&latest.version)?>release::version(version)?)).transpose()?.unwrap_or(true);
 if command=="check"{
  let report=serde_json::json!({"schemaVersion":1,"installedVersion":current,"latestStableVersion":latest.version,"target":latest.target,"updateAvailable":newer,"managed":root.is_some(),"networkUsed":true});
  if arguments==["--json"]{println!("{}",serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?);}else{println!("Latest stable Tsrust: {} ({})",latest.version,if newer{"update available"}else{"no newer release"});if root.is_none(){println!("Use your package manager to upgrade, or install into a new managed directory.");}}return Ok(0);
 }
 if !newer{println!("No newer stable toolchain is available; nothing changed.");return Ok(0);}
 let bytes=runtime.block_on(release::download(&latest))?;let root=root.unwrap();
 let state=if command=="install"{install::install(&root,&latest,&bytes,&std::env::current_exe().map_err(|e|e.to_string())?)?}else{install::update(&root,&latest,&bytes)?};
 println!("Selected Tsrust {} and its paired LSP in {}.",state.current.release.version,root.display());
 if command=="install"{println!("Add {} to PATH and select its tsrust-lsp in your editor. No shell profile or Rust configuration was changed.",root.join("bin").display());}else{println!("Restart editor servers to use the new pair. The previous verified pair is available through tsrust toolchain rollback.");}
 Ok(0)
}
pub(super) fn readiness()->serde_json::Value{
 match install::current_root(){
  Ok(Some(root))=>match install::state(&root).and_then(|state|{install::manifest(&root,&state.current)?;release::verify(&root.join("versions").join(&state.current.release.sha256),&state.current.release)}){
   Ok(manifest)=>serde_json::json!({"status":"ok","managed":true,"compilerVersion":manifest.version,"lspVersion":manifest.lsp_version,"scope":"selected pair, file checksums and manifest integrity; no update request"}),
   Err(error)=>serde_json::json!({"status":"error","managed":true,"message":error})
  },
  Ok(None)=>serde_json::json!({"status":"unmanaged","managed":false,"message":"Use the owning package manager for upgrades, or install into a new managed directory"}),
  Err(error)=>serde_json::json!({"status":"error","message":error})
 }
}
