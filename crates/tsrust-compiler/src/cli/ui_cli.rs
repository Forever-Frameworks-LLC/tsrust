//! Embedded immutable source installation. No registry, subprocess hooks or package manager.
use std::{collections::BTreeMap,fs,path::{Path,PathBuf},io::{Read,Write}};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use tsrust_compiler::ui_project::{Config,safe_relative};
const ABI:&str=tsrust_compiler::ui_project::RUNTIME_ABI;
const CATALOG:&str="bundled-2026-10-18";
#[derive(Clone,Debug,Deserialize,Serialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
struct Identity {name:String,version:String,variant:String}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct File {path:String,destination:String,content:String}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Item {name:String,version:String,variant:String,license:String,license_text:String,runtime_abi:String,compiler_api:u32,rust:String,dependencies:Vec<Identity>,files:Vec<File>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct LockedFile {path:PathBuf,template_sha256:String,generated_sha256:String}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Installed {identity:Identity,destination:PathBuf,catalog:String,payload_sha256:String,compiler_sha256:String,compiler_version:String,compiler_api:u32,runtime_abi:String,rust:String,license:String,license_text:String,dependencies:Vec<Identity>,files:Vec<LockedFile>}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Lock {schema_version:u32,installed:Vec<Installed>}
fn hash(bytes:&[u8])->String {Sha256::digest(bytes).iter().map(|byte|format!("{byte:02x}")).collect()}
fn catalog()->Result<Vec<Item>,String> {
 let mut items=Vec::new();
 for text in [include_str!("../ui-catalog/tokens-1.0.0.json"),include_str!("../ui-catalog/components-1.0.0.json"),include_str!("../ui-catalog/components-2.0.0.json")] {items.extend(serde_json::from_str::<Vec<Item>>(text).map_err(|e|e.to_string())?);}
 Ok(items)
}
fn identity(item:&Item)->Identity {Identity{name:item.name.clone(),version:item.version.clone(),variant:item.variant.clone()}}
fn resolve(request:&Identity,items:&[Item],resolved:&mut Vec<Item>,active:&mut Vec<Identity>)->Result<(),String> {
 if resolved.iter().any(|item|identity(item)==*request) {return Ok(());}
 if active.contains(request) {return Err("Bundled UI dependency cycle".into());}
 if resolved.iter().any(|item|item.name==request.name && item.version!=request.version) {return Err(format!("Conflicting UI dependency versions for {}",request.name));}
 let item=items.iter().find(|item|identity(item)==*request).ok_or_else(||format!("UI component {}@{} ({}) is not bundled; run tsrust ui list --bundled",request.name,request.version,request.variant))?;
 if item.runtime_abi!=ABI || u64::from(item.compiler_api)!=tsrust_compiler::ui_project::COMPILER_API || item.rust!=tsrust_compiler::RUST_BACKEND_VERSION {return Err(format!("Unsupported bundled prerequisites for {}@{}",item.name,item.version));}
 active.push(request.clone());for dependency in &item.dependencies {resolve(dependency,items,resolved,active)?;}active.pop();resolved.push(item.clone());Ok(())
}
fn no_symlinks(root:&Path,relative:&Path)->Result<PathBuf,String> {
 safe_relative(relative)?;let mut current=root.to_owned();
 for part in relative.components() {current.push(part);match fs::symlink_metadata(&current) {Ok(metadata) if metadata.file_type().is_symlink()=>return Err(format!("UI installation refuses symlink: {}",current.display())),Ok(_)=>{},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(e)=>return Err(e.to_string())}}
 Ok(current)
}
fn read_lock(root:&Path,config:&Config)->Result<(Lock,Option<Vec<u8>>),String> {
 let path=no_symlinks(root,&config.lock)?;
 let bytes=match fs::read(&path) {Ok(bytes)=>Some(bytes),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>None,Err(e)=>return Err(e.to_string())};
 let lock=match &bytes {Some(bytes)=>serde_json::from_slice::<Lock>(bytes).map_err(|e|format!("Invalid {}: {e}",config.lock.display()))?,None=>Lock{schema_version:1,installed:vec![]}};
 if lock.schema_version!=1 {return Err("Unsupported UI lock schemaVersion".into());}
 for entry in &lock.installed {
  if entry.runtime_abi!=ABI || u64::from(entry.compiler_api)!=tsrust_compiler::ui_project::COMPILER_API || entry.rust!=tsrust_compiler::RUST_BACKEND_VERSION {return Err("Installed UI component requires another compiler/runtime/toolchain".into());}
  safe_relative(&entry.destination)?;for file in &entry.files {no_symlinks(root,&file.path)?;}
 }
 Ok((lock,bytes))
}
struct Plan {files:Vec<(PathBuf,Vec<u8>)>,lock_path:PathBuf,previous:Option<Vec<u8>>,lock:Vec<u8>,messages:Vec<String>}
fn plan(root:&Path,config:&Config,requests:&[Identity],destination:Option<&Path>,compiler_hash:&str)->Result<Plan,String> {
 let destination=destination.unwrap_or(&config.components);no_symlinks(root,destination)?;
 let items=catalog()?;let mut resolved=Vec::new();for request in requests {resolve(request,&items,&mut resolved,&mut vec![])?;}
 let (mut lock,previous)=read_lock(root,config)?;let mut files=BTreeMap::new();let mut messages=vec![];
 for item in resolved {
  let id=identity(&item);let target=if item.name=="tokens" {&config.theme}else{destination};
  if let Some(installed)=lock.installed.iter().find(|value|value.identity.name==id.name && value.destination==target) {
   if installed.identity!=id {return Err(format!("{} contains {}@{}; upgrades never overwrite owned source. Select another --path.",target.display(),id.name,installed.identity.version));}
   if installed.payload_sha256!=hash(&serde_json::to_vec(&item).map_err(|e|e.to_string())?) {return Err("Installed UI payload identity does not match the bundled immutable version".into());}
   for file in &installed.files {if !no_symlinks(root,&file.path)?.is_file(){return Err(format!("Installed UI file missing: {}. Restore it from version control.",file.path.display()));}}
   messages.push(format!("preserve {}@{} in {} (including local edits)",id.name,id.version,target.display()));continue;
  }
  let mut locked=vec![];
  for file in &item.files {
   safe_relative(Path::new(&file.path))?;
   let path=match file.destination.as_str() {"theme"=>config.theme.clone(),"components"=>target.join(&file.path),_=>return Err("Unsupported bundled destination".into())};
   if no_symlinks(root,&path)?.exists() || files.contains_key(&path) {return Err(format!("Refusing to overwrite {}. Existing files need an installation record.",path.display()));}
   let generated=if file.path.ends_with(".tsrx") {tsrust_compiler::format(&file.content).map_err(|e|format!("Invalid bundled component: {}",e.message))?}else {tsrust_compiler::ui_project::theme::Theme::parse(&file.content)?;file.content.clone()};
   locked.push(LockedFile{path:path.clone(),template_sha256:hash(file.content.as_bytes()),generated_sha256:hash(generated.as_bytes())});
   messages.push(format!("create {} ({}@{})",path.display(),id.name,id.version));files.insert(path,generated.into_bytes());
  }
  let payload=serde_json::to_vec(&item).map_err(|e|e.to_string())?;
  lock.installed.push(Installed{identity:id,destination:target.to_owned(),catalog:CATALOG.into(),payload_sha256:hash(&payload),compiler_sha256:compiler_hash.into(),compiler_version:env!("CARGO_PKG_VERSION").into(),compiler_api:tsrust_compiler::ui_project::COMPILER_API as u32,runtime_abi:ABI.into(),rust:tsrust_compiler::RUST_BACKEND_VERSION.into(),license:item.license,license_text:item.license_text,dependencies:item.dependencies,files:locked});
 }
 let bytes=if files.is_empty(){previous.clone().unwrap_or_default()}else {let mut bytes=serde_json::to_vec_pretty(&lock).map_err(|e|e.to_string())?;bytes.push(b'\n');bytes};
 Ok(Plan{files:files.into_iter().collect(),lock_path:config.lock.clone(),previous,lock:bytes,messages})
}
fn apply(root:&Path,plan:&Plan)->Result<(),String> {
 if plan.files.is_empty(){return Ok(());}
 let mutex=no_symlinks(root,Path::new("target/ui-install.lock"))?;
 fs::create_dir_all(mutex.parent().unwrap()).map_err(|e|e.to_string())?;
 let mutex=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(mutex).map_err(|e|e.to_string())?;
 mutex.lock().map_err(|e|e.to_string())?;
 // Preflight is complete. Exclusive creation protects against concurrent installation.
 let lock_path=no_symlinks(root,&plan.lock_path)?;
 if fs::read(&lock_path).ok()!=plan.previous {return Err("UI lock changed during installation; retry".into());}
 let mut created:Vec<(PathBuf,Vec<u8>)>=vec![];
 let result=(|| {
  for (path,bytes) in &plan.files {
   let path=no_symlinks(root,path)?;if let Some(parent)=path.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
   let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e|format!("{}: {e}",path.display()))?;
   created.push((path,bytes.clone()));file.write_all(bytes).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;
  }
  if let Some(parent)=lock_path.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
  let temporary=lock_path.with_extension(format!("json.{}.tmp",std::process::id()));
  let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;
  let result=(||{file.write_all(&plan.lock).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;
   if fs::read(&lock_path).ok()!=plan.previous{return Err("UI lock changed during installation; retry".into());}
   fs::rename(&temporary,&lock_path).map_err(|e|e.to_string())})();let _=fs::remove_file(&temporary);result
 })();
 if result.is_err(){for (path,bytes) in created {if fs::read(&path).ok().as_deref()==Some(bytes.as_slice()){let _=fs::remove_file(path);}}}
 result
}
fn compiler_hash()->Result<String,String> {
 let executable=std::env::current_exe().map_err(|e|e.to_string())?;let mut file=fs::File::open(executable).map_err(|e|e.to_string())?;
 let mut digest=Sha256::new();let mut buffer=[0u8;65536];loop{let read=file.read(&mut buffer).map_err(|e|e.to_string())?;if read==0{break;}digest.update(&buffer[..read]);}
 Ok(digest.finalize().iter().map(|byte|format!("{byte:02x}")).collect())
}
pub(super) fn install_defaults(root:&Path,config:&Config)->Result<(),String> {
 super::verify_backend()?;let hash=compiler_hash()?;
 let requests=["button","card"].map(|name|Identity{name:name.into(),version:"1.0.0".into(),variant:"default".into()});
 apply(root,&plan(root,config,&requests,None,&hash)?)
}
pub(super) fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
 let mut args=args;let command=args.next().ok_or("Usage: tsrust ui <list --bundled|init|add name@version> [--cwd directory] [--path directory] [--dry-run] [--json]")?;
 let(mut cwd,mut path,mut variant,mut dry,mut json)=(None,None,"default".to_owned(),false,false);let mut requests=vec![];
 while let Some(arg)=args.next(){match arg.as_str(){"--cwd"=>cwd=Some(PathBuf::from(args.next().ok_or("--cwd requires a directory")?)),"--path"=>path=Some(PathBuf::from(args.next().ok_or("--path requires a relative directory")?)),"--variant"=>variant=args.next().ok_or("--variant requires a name")?,"--dry-run"=>dry=true,"--json"=>json=true,"--bundled"|"--offline"|"--yes"=>{},_ if !arg.starts_with('-')=>requests.push(arg),_=>return Err(format!("Unknown UI option: {arg}"))}}
 if command=="list" {if !requests.is_empty() || path.is_some() || dry {return Err("Usage: tsrust ui list --bundled [--json]".into());}let items=catalog()?;if json {println!("{}",serde_json::to_string_pretty(&items).unwrap());}else {for item in items {println!("{}@{} --variant {} ({})",item.name,item.version,item.variant,item.license);}}return Ok(0);}
 let root=fs::canonicalize(match cwd {Some(path)=>path,None=>super::project_config::default_root()?}).map_err(|e|e.to_string())?;
 let mut project=super::project_config::load(&root)?;
 if command=="init" {
  if !requests.is_empty() {return Err("ui init accepts no component names".into());}
  if project.ui.is_some(){return Err("UI configuration already exists; use tsrust ui add".into());}
  let mut config=Config::default();if let Some(path)=path{config.components=path;}Config::parse(&serde_json::to_value(&config).unwrap())?;
  let config_path=root.join(super::project_config::FILE);let original=fs::read(&config_path).map_err(|e|e.to_string())?;
  let mut value:serde_json::Value=serde_json::from_slice(&original).map_err(|e|e.to_string())?;value["ui"]=serde_json::to_value(&config).unwrap();
  let mut plan=plan(&root,&config,&[Identity{name:"tokens".into(),version:"1.0.0".into(),variant:"default".into()}],None,&compiler_hash()?)?;
  plan.messages.push("configure tsrust.config.json".into());print_plan(&plan,json);
  if !dry {super::verify_backend()?;apply(&root,&plan)?;fs::write(config_path,format!("{}\n",serde_json::to_string_pretty(&value).unwrap())).map_err(|e|e.to_string())?;}return Ok(0);
 }
 if command!="add" || requests.is_empty() {return Err("Usage: tsrust ui add name@version ... [--variant default] [--path directory] [--cwd directory] [--dry-run] [--json]".into());}
 let config=project.ui.take().ok_or("Missing ui configuration; run tsrust ui init first")?;
 let requests=requests.into_iter().map(|request|{let (name,version)=request.split_once('@').ok_or("UI installation requires an exact bundled version, for example button@1.0.0")?;Ok(Identity{name:name.into(),version:version.into(),variant:variant.clone()})}).collect::<Result<Vec<_>,String>>()?;
 let plan=plan(&root,&config,&requests,path.as_deref(),&compiler_hash()?)?;print_plan(&plan,json);
 if !dry {super::verify_backend()?;apply(&root,&plan)?;}Ok(0)
}
fn print_plan(plan:&Plan,json:bool){if json {println!("{}",serde_json::json!({"offline":true,"changes":plan.messages,"lock":plan.lock_path}));}else {for message in &plan.messages{println!("{message}");}}}

#[cfg(test)]mod tests {
 use super::*;
 fn scratch()->PathBuf {let path=std::env::temp_dir().join(format!("tsrust-ui-installer-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));fs::create_dir(&path).unwrap();path}
 #[test]fn versioned_offline_install_preserves_owned_source_and_lock() {
  let root=scratch();let config=Config::default();let request=|version:&str|Identity{name:"button".into(),version:version.into(),variant:"default".into()};
  let initial=plan(&root,&config,&[request("1.0.0")],Some(Path::new("local/controls")),"compiler-one").unwrap();assert!(!root.join(&config.lock).exists());apply(&root,&initial).unwrap();
  let source=root.join("local/controls/button.tsrx");fs::write(&source,"// owned customization\n").unwrap();let lock=fs::read(root.join(&config.lock)).unwrap();
  let same=plan(&root,&config,&[request("1.0.0")],Some(Path::new("local/controls")),"compiler-two").unwrap();apply(&root,&same).unwrap();assert_eq!(fs::read(root.join(&config.lock)).unwrap(),lock);assert_eq!(fs::read_to_string(&source).unwrap(),"// owned customization\n");
  for version in ["9.9.9","2.0.0"] {assert!(plan(&root,&config,&[request(version)],Some(Path::new("local/controls")),"compiler-two").is_err());assert_eq!(fs::read(root.join(&config.lock)).unwrap(),lock);}
  let next=plan(&root,&config,&[request("2.0.0")],Some(Path::new("alternate")),"compiler-two").unwrap();apply(&root,&next).unwrap();assert!(root.join("alternate/button.tsrx").is_file());assert!(plan(&root,&config,&[request("1.0.0")],Some(Path::new("../escape")),"compiler").is_err());
  fs::remove_dir_all(root).unwrap();
 }
 #[cfg(unix)]#[test]fn refuses_symlink_and_preflights_every_file(){let root=scratch();let config=Config::default();std::os::unix::fs::symlink(std::env::temp_dir(),root.join("components")).unwrap();let id=Identity{name:"button".into(),version:"1.0.0".into(),variant:"default".into()};assert!(plan(&root,&config,&[id],None,"compiler").is_err());assert!(!root.join("ui.theme.json").exists());fs::remove_dir_all(root).unwrap();}
}
