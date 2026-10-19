//! Embedded immutable source installation. No registry, subprocess hooks or package manager.
use std::{collections::BTreeMap,fs,path::{Path,PathBuf},io::{Read,Write}};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use tsrust_compiler::ui_project::{Config,Targets,safe_relative};
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
struct LockedFile {path:PathBuf,#[serde(default,skip_serializing_if="Option::is_none")] json_pointer:Option<String>,template_sha256:String,generated_sha256:String}
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
struct Update {path:PathBuf,previous:Option<Vec<u8>>,bytes:Vec<u8>}
struct Plan {files:Vec<(PathBuf,Vec<u8>)>,updates:Vec<Update>,directories:Vec<PathBuf>,lock_path:PathBuf,previous:Option<Vec<u8>>,lock:Vec<u8>,messages:Vec<String>}
fn plan(root:&Path,config:&Config,requests:&[Identity],destination:Option<&Path>,compiler_hash:&str)->Result<Plan,String> {
 let destination=destination.unwrap_or(&config.components);no_symlinks(root,destination)?;
 let items=catalog()?;let mut resolved=Vec::new();for request in requests {resolve(request,&items,&mut resolved,&mut vec![])?;}
 let (mut lock,previous)=read_lock(root,config)?;let installed_before=lock.installed.len();let mut files=BTreeMap::new();let mut messages=vec![];
 for item in resolved {
  let id=identity(&item);let target=if item.name=="tokens" {config.theme_destination()}else{destination};
  if let Some(installed)=lock.installed.iter().find(|value|value.identity.name==id.name && value.destination==target) {
   if installed.identity!=id {return Err(format!("{} contains {}@{}; upgrades never overwrite owned source. Select another --path.",target.display(),id.name,installed.identity.version));}
   if installed.payload_sha256!=hash(&serde_json::to_vec(&item).map_err(|e|e.to_string())?) {return Err("Installed UI payload identity does not match the bundled immutable version".into());}
   for file in &installed.files {if !no_symlinks(root,&file.path)?.is_file(){return Err(format!("Installed UI file missing: {}. Restore it from version control.",file.path.display()));}}
   messages.push(format!("preserve {}@{} in {} (including local edits)",id.name,id.version,target.display()));continue;
  }
  let mut locked=vec![];
  for file in &item.files {
   safe_relative(Path::new(&file.path))?;
   let path=match file.destination.as_str() {"theme"=>config.theme_destination().to_owned(),"components"=>target.join(&file.path),_=>return Err("Unsupported bundled destination".into())};
   if file.destination=="theme" && config.external_theme().is_none() {
    locked.push(LockedFile{path,json_pointer:Some("/ui/theme".into()),template_sha256:hash(file.content.as_bytes()),generated_sha256:hash(&serde_json::to_vec_pretty(&config.theme).map_err(|e|e.to_string())?)});
    messages.push("use project-owned tsrust.config.json ui.theme".into());continue;
   }
   if no_symlinks(root,&path)?.exists() || files.contains_key(&path) {return Err(format!("Refusing to overwrite {}. Existing files need an installation record.",path.display()));}
   let generated=if file.path.ends_with(".tsrx") {tsrust_compiler::format(&file.content).map_err(|e|format!("Invalid bundled component: {}",e.message))?}else {tsrust_compiler::ui_project::theme::Theme::parse(&file.content)?;file.content.clone()};
   locked.push(LockedFile{path:path.clone(),json_pointer:None,template_sha256:hash(file.content.as_bytes()),generated_sha256:hash(generated.as_bytes())});
   messages.push(format!("create {} ({}@{})",path.display(),id.name,id.version));files.insert(path,generated.into_bytes());
  }
  let payload=serde_json::to_vec(&item).map_err(|e|e.to_string())?;
  lock.installed.push(Installed{identity:id,destination:target.to_owned(),catalog:CATALOG.into(),payload_sha256:hash(&payload),compiler_sha256:compiler_hash.into(),compiler_version:env!("CARGO_PKG_VERSION").into(),compiler_api:tsrust_compiler::ui_project::COMPILER_API as u32,runtime_abi:ABI.into(),rust:tsrust_compiler::RUST_BACKEND_VERSION.into(),license:item.license,license_text:item.license_text,dependencies:item.dependencies,files:locked});
 }
 let bytes=if lock.installed.len()==installed_before {previous.clone().unwrap_or_default()}else {let mut bytes=serde_json::to_vec_pretty(&lock).map_err(|e|e.to_string())?;bytes.push(b'\n');bytes};
 Ok(Plan{files:files.into_iter().collect(),updates:vec![],directories:vec![PathBuf::from("src/app"),destination.to_owned()],lock_path:config.lock.clone(),previous,lock:bytes,messages})
}
fn read_optional(path:&Path)->Result<Option<Vec<u8>>,String> {
 match fs::read(path){Ok(bytes)=>Ok(Some(bytes)),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(e)=>Err(format!("{}: {e}",path.display()))}
}
fn apply(root:&Path,plan:&Plan)->Result<(),String> {
 let mut changes=plan.files.iter().map(|(path,bytes)|Update{path:path.clone(),previous:None,bytes:bytes.clone()}).collect::<Vec<_>>();
 changes.extend(plan.updates.iter().map(|u|Update{path:u.path.clone(),previous:u.previous.clone(),bytes:u.bytes.clone()}));
 if plan.previous.as_deref()!=Some(plan.lock.as_slice()) {changes.push(Update{path:plan.lock_path.clone(),previous:plan.previous.clone(),bytes:plan.lock.clone()});}
 let mut unique=std::collections::BTreeSet::new();
 for change in &changes {
  if !unique.insert(&change.path){return Err(format!("UI destinations overlap: {}",change.path.display()));}
  let path=no_symlinks(root,&change.path)?;
  if read_optional(&path)?!=change.previous {return Err(format!("{} changed during installation; retry",change.path.display()));}
  if fs::metadata(&path).is_ok_and(|m|m.permissions().readonly()){return Err(format!("{} is read-only",change.path.display()));}
 }
 for directory in &plan.directories {let path=no_symlinks(root,directory)?;if path.exists() && !path.is_dir(){return Err(format!("{} must be a directory",path.display()));}}
 if changes.is_empty() && plan.directories.iter().all(|path|root.join(path).is_dir()){return Ok(());}
 let mutex=no_symlinks(root,Path::new("target/ui-install.lock"))?;
 fs::create_dir_all(mutex.parent().unwrap()).map_err(|e|e.to_string())?;
 let mutex=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(mutex).map_err(|e|e.to_string())?;
 mutex.lock().map_err(|e|e.to_string())?;
 let mut directories=vec![];let mut staged=vec![];let mut applied=vec![];
 let mut create_directory=|relative:&Path|->Result<(),String>{
  let mut path=root.to_owned();for part in relative.components(){path.push(part);match fs::create_dir(&path){Ok(())=>directories.push(path.clone()),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists && path.is_dir() && !path.is_symlink()=>{},Err(e)=>return Err(format!("{}: {e}",path.display()))}}Ok(())
 };
 let result=(|| {
  // Stage every byte before publishing; the lock is always published last.
  for (index,change) in changes.iter().enumerate() {
   let path=no_symlinks(root,&change.path)?;if let Some(parent)=change.path.parent(){create_directory(parent)?;}
   let temporary=path.with_file_name(format!(".tsrust-ui-{}-{index}.tmp",std::process::id()));
   let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;staged.push(temporary);
   if let Ok(metadata)=fs::metadata(&path){file.set_permissions(metadata.permissions()).map_err(|e|e.to_string())?;}
   file.write_all(&change.bytes).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;
  }
  for directory in &plan.directories {no_symlinks(root,directory)?;create_directory(directory)?;}
  for (index,change) in changes.iter().enumerate() {
   let path=no_symlinks(root,&change.path)?;
   if read_optional(&path)?!=change.previous {return Err(format!("{} changed during installation; retry",change.path.display()));}
   if change.previous.is_some(){fs::rename(&staged[index],&path).map_err(|e|e.to_string())?;}else{fs::hard_link(&staged[index],&path).map_err(|e|e.to_string())?;}
   applied.push(index);
  }
  Ok(())
 })();
 drop(create_directory);
 if result.is_err(){
  for index in applied.into_iter().rev(){let change=&changes[index];let path=root.join(&change.path);
   if fs::read(&path).ok().as_deref()!=Some(change.bytes.as_slice()){continue;}
   if let Some(previous)=&change.previous {
    let temporary=path.with_file_name(format!(".tsrust-ui-{}-rollback-{index}.tmp",std::process::id()));
    if let Ok(mut file)=fs::OpenOptions::new().write(true).create_new(true).open(&temporary){
     if let Ok(metadata)=fs::metadata(&path){let _=file.set_permissions(metadata.permissions());}
     if file.write_all(previous).and_then(|_|file.sync_all()).is_ok(){let _=fs::rename(&temporary,&path);}let _=fs::remove_file(temporary);
    }
   }else{let _=fs::remove_file(path);}
  }
 }
 for path in staged {let _=fs::remove_file(path);}
 if result.is_err(){for path in directories.into_iter().rev(){let _=fs::remove_dir(path);}}
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
 let mut targets=vec![];let mut unattended=false;
 let(mut cwd,mut path,mut variant,mut dry,mut json)=(None,None,"default".to_owned(),false,false);let mut requests=vec![];
 while let Some(arg)=args.next(){match arg.as_str(){"--cwd"=>cwd=Some(PathBuf::from(args.next().ok_or("--cwd requires a directory")?)),"--path"=>path=Some(PathBuf::from(args.next().ok_or("--path requires a relative directory")?)),"--target"=>targets.push(args.next().ok_or("--target requires native or web")?),"--variant"=>variant=args.next().ok_or("--variant requires a name")?,"--dry-run"=>dry=true,"--json"=>json=true,"--yes"=>unattended=true,"--bundled"|"--offline"=>{},_ if !arg.starts_with('-')=>requests.push(arg),_=>return Err(format!("Unknown UI option: {arg}"))}}
 if command=="list" {if !requests.is_empty() || path.is_some() || dry || !targets.is_empty() {return Err("Usage: tsrust ui list --bundled [--json]".into());}let items=catalog()?;if json {println!("{}",serde_json::to_string_pretty(&items).unwrap());}else {for item in items {println!("{}@{} --variant {} ({})",item.name,item.version,item.variant,item.license);}}return Ok(0);}
 let selected=match cwd {Some(path)=>path,None=>std::env::current_dir().map_err(|e|e.to_string())?};
 let selected=fs::canonicalize(selected).map_err(|e|e.to_string())?;
 let root=selected.ancestors().find(|root|root.join(super::project_config::FILE).exists()).unwrap_or(&selected).to_owned();
 if command!="init" && (command!="add" || requests.is_empty()) {return Err("Usage: tsrust ui add name@version ... [--cwd directory] [--path directory] [--dry-run] [--json]".into());}
 if command=="init" && !requests.is_empty(){return Err("ui init accepts no component names".into());}
 let requests=if command=="init" {vec![Identity{name:"tokens".into(),version:"1.0.0".into(),variant:"default".into()}]}else{
  requests.into_iter().map(|request|{let (name,version)=request.split_once('@').ok_or("UI installation requires an exact bundled version, for example button@1.0.0")?;Ok(Identity{name:name.into(),version:version.into(),variant:variant.clone()})}).collect::<Result<Vec<_>,String>>()?
 };
 let plan=setup_plan_with_targets(&root,&requests,path.as_deref(),&compiler_hash()?,&targets,dry || unattended)?;print_plan(&plan,json);
 if !dry {super::verify_backend()?;apply(&root,&plan)?;}Ok(0)
}
fn setup_plan_with_targets(root:&Path,requests:&[Identity],destination:Option<&Path>,compiler_hash:&str,targets:&[String],dry:bool)->Result<Plan,String> {
 let items=catalog()?;let mut checked=vec![];for request in requests{resolve(request,&items,&mut checked,&mut vec![])?;}
 let config_path=Path::new(super::project_config::FILE);let previous=read_optional(&no_symlinks(root,config_path)?)?;
 let mut value=if let Some(bytes)=&previous {
  let text=std::str::from_utf8(bytes).map_err(|e|e.to_string())?;
  serde_json::from_str::<serde_json::Value>(text).map_err(|e|e.to_string())?
 }else{
  let name=root.file_name().and_then(|v|v.to_str()).filter(|v|super::project_config::valid_name(v)).unwrap_or("ui-app");
  let entry=["src/main.tsr","src/main.tsrx","main.tsr","main.tsrx"].into_iter().find(|v|root.join(v).is_file()).unwrap_or("src/main.tsr");
  serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":entry,"output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)}})
 };
 if !value.is_object(){return Err("tsrust.config.json must be an object".into());}
 let mut shape=value.clone();shape.as_object_mut().unwrap().remove("ui");super::project_config::parse(root,&shape.to_string())?;
 if value.get("ui").is_some_and(|ui|!ui.is_object()){return Err("ui must be an object".into());}
 let previous_target=value.get("ui").and_then(|v|v.get("target")).map(|v|serde_json::from_value::<Targets>(v.clone()).map_err(|e|e.to_string())).transpose()?;
 let selected=select_targets(previous_target.as_ref(),targets,dry)?;
 let target_changed=previous_target.as_ref()!=Some(&selected);
 let first=value.get("ui").is_none();let defaults=serde_json::to_value(Config::default()).unwrap();let mut configured=first || target_changed;
 if first{value["ui"]=defaults.clone();}else if let Some(object)=value.get_mut("ui").and_then(serde_json::Value::as_object_mut){for (key,default) in defaults.as_object().unwrap(){if !object.contains_key(key){object.insert(key.clone(),default.clone());configured=true;}}}
 value["ui"]["target"]=serde_json::to_value(&selected).unwrap();
 let mut config=Config::parse(&value["ui"])?;
 if first {if let Some(path)=destination{config.components=path.to_owned();}Config::parse(&serde_json::to_value(&config).unwrap())?;value["ui"]=serde_json::to_value(&config).unwrap();}
 if configured{value["ui"]=serde_json::to_value(&config).unwrap();}
 super::project_config::parse(root,&value.to_string())?;
 let mut plan=plan(root,&config,requests,destination,compiler_hash)?;
 if configured {
  plan.messages.push(format!("Configured UI theme in tsrust.config.json; components in {} and root screen convention src/app/page.tsrx.",config.components.display()));
  plan.messages.push("Import config from tsrust.config.json and apply withTheme(Page(), config.ui.theme) in your bootstrap. Import controls from their local source paths.".into());
  plan.messages.push(if selected.native{"Native selected: run tsrust dev from the project directory."}else{"Web selected: use the bounded Wasm web builder. A general web dev server is not implemented yet; native dev/run is disabled."}.into());
 }
 if previous.is_none() && !root.join(value["build"]["entry"].as_str().unwrap()).exists() {
  for (path,source) in [("src/main.tsr",super::new_cli::UI_MAIN),("src/app/page.tsrx",super::new_cli::UI_EMPTY_PAGE)] {
   if root.join(path).exists(){if path.ends_with("page.tsrx") && no_symlinks(root,Path::new(path))?.is_file(){continue;}return Err(format!("Refusing to overwrite {path}"));}
   plan.files.push((path.into(),tsrust_compiler::format(source).map_err(|e|e.message)?.into_bytes()));
  }
  plan.messages.push("Created src/main.tsr bootstrap and src/app/page.tsrx. Edit the page to compose installed components.".into());
 }
 if configured || previous.is_none(){plan.updates.push(Update{path:config_path.into(),previous,bytes:format!("{}\n",serde_json::to_string_pretty(&value).map_err(|e|e.to_string())?).into_bytes()});}
 Ok(plan)
}

fn print_plan(plan:&Plan,json:bool){if json {println!("{}",serde_json::json!({"offline":true,"changes":plan.messages,"lock":plan.lock_path}));}else {for message in &plan.messages{println!("{message}");}}}

#[cfg(test)]mod tests {
 use super::*;
 fn scratch()->PathBuf {let path=std::env::temp_dir().join(format!("tsrust-ui-installer-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));fs::create_dir(&path).unwrap();path}
 #[test]fn versioned_offline_install_preserves_owned_source_and_lock() {
  let root=scratch();let config=Config::default();let request=|version:&str|Identity{name:"button".into(),version:version.into(),variant:"default".into()};
  let initial=setup_plan(&root,&[request("1.0.0")],Some(Path::new("local/controls")),"compiler-one",None).unwrap();assert!(!root.join(&config.lock).exists());apply(&root,&initial).unwrap();
  let source=root.join("local/controls/button.tsrx");fs::write(&source,"// owned customization\n").unwrap();let lock=fs::read(root.join(&config.lock)).unwrap();
  let same=plan(&root,&config,&[request("1.0.0")],Some(Path::new("local/controls")),"compiler-two").unwrap();apply(&root,&same).unwrap();assert_eq!(fs::read(root.join(&config.lock)).unwrap(),lock);assert_eq!(fs::read_to_string(&source).unwrap(),"// owned customization\n");
  for version in ["9.9.9","2.0.0"] {assert!(plan(&root,&config,&[request(version)],Some(Path::new("local/controls")),"compiler-two").is_err());assert_eq!(fs::read(root.join(&config.lock)).unwrap(),lock);}
  let next=plan(&root,&config,&[request("2.0.0")],Some(Path::new("alternate")),"compiler-two").unwrap();apply(&root,&next).unwrap();assert!(root.join("alternate/button.tsrx").is_file());assert!(plan(&root,&config,&[request("1.0.0")],Some(Path::new("../escape")),"compiler").is_err());
  fs::remove_dir_all(root).unwrap();
 }
 #[cfg(unix)]#[test]fn refuses_symlink_and_preflights_every_file(){let root=scratch();let config=Config::default();std::os::unix::fs::symlink(std::env::temp_dir(),root.join("src")).unwrap();let id=Identity{name:"button".into(),version:"1.0.0".into(),variant:"default".into()};assert!(plan(&root,&config,&[id],None,"compiler").is_err());assert!(!root.join("ui.theme.json").exists());fs::remove_dir_all(root).unwrap();}
}

fn project_root(entry:&Path)->Result<PathBuf,String> {
 for root in entry.parent().unwrap().ancestors(){match fs::symlink_metadata(root.join(super::project_config::FILE)){Ok(_)=>return Ok(root.to_owned()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(e)=>return Err(e.to_string())}}
 Ok(entry.ancestors().find(|p|p.file_name().is_some_and(|v|v=="src")).and_then(Path::parent).unwrap_or(entry.parent().unwrap()).to_owned())
}
fn normalized(path:&Path)->PathBuf {
 let mut result=PathBuf::new();for part in path.components(){match part{std::path::Component::CurDir=>{},std::path::Component::ParentDir=>{result.pop();},_=>result.push(part.as_os_str())}}result
}
/// Report explicit installation commands for missing UI setup; never write during execution.
pub(super) fn check_setup(entry:&Path)->Result<(),String> {
 let entry=fs::canonicalize(entry).map_err(|e|e.to_string())?;let root=project_root(&entry)?;
 let value=read_optional(&no_symlinks(&root,Path::new(super::project_config::FILE))?)?.map(|bytes|serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e|e.to_string())).transpose()?;
 let components=value.as_ref().and_then(|v|v.pointer("/ui/components")).and_then(serde_json::Value::as_str).map(PathBuf::from).unwrap_or_else(||Config::default().components);
 safe_relative(&components)?;
 let aliases=tsrust_compiler::project_resolution::Aliases::parse(&root,value.as_ref().and_then(|v|v.get("paths")))?;
 let mut pending=vec![entry.clone()];let mut seen=std::collections::BTreeSet::new();let mut requests=vec![];let mut uses_ui=false;
 let items=catalog()?;
 while let Some(path)=pending.pop(){
  let path=normalized(&path);if !seen.insert(path.clone()) || path.extension().is_none_or(|v|v!="tsr" && v!="tsrx"){continue;}
  let text=fs::read_to_string(&path).map_err(|e|e.to_string())?;
  let imports=match tsrust_compiler::ui_project::source_imports(&text){Ok(imports)=>imports,Err(_)=>return Ok(())};
  for import in imports {
   if import=="std:ui"{uses_ui=true;continue;}if import.starts_with("std:"){continue;}
   let Ok(candidates)=tsrust_compiler::project_resolution::candidates(&path,&import,Some(&aliases)) else{continue;};
   if let Some(target)=candidates.iter().find(|path|path.is_file()){pending.push(target.clone());continue;}
   for item in items.iter().filter(|item|item.version=="1.0.0" && item.name!="tokens"){
    if item.files.iter().any(|file|candidates.iter().any(|target|normalized(target)==root.join(&components).join(&file.path))){uses_ui=true;requests.push(identity(item));}
   }
  }
 }
 if !uses_ui{return Ok(());}
 if requests.is_empty() && value.as_ref().and_then(|v|v.get("ui")).is_some_and(|ui|Config::parse(ui).is_ok_and(|c|c.target.is_some())){return Ok(());}
 if requests.is_empty(){requests.push(Identity{name:"tokens".into(),version:"1.0.0".into(),variant:"default".into()});}
 requests.sort_by(|a,b|a.name.cmp(&b.name));requests.dedup_by(|a,b|a.name==b.name);
 let command=requests.iter().map(|item|format!("{}@{}",item.name,item.version)).collect::<Vec<_>>().join(" ");
 Err(format!("UI setup or bundled source is missing. Run `tsrust ui add {command} --target native` (or --target web) from {}. This command only checks; no project files were created.",root.display()))
}
pub(super) fn check_default_setup()->Result<(),String>{
 let cwd=std::env::current_dir().map_err(|e|e.to_string())?;
 for root in cwd.ancestors(){let path=root.join(super::project_config::FILE);if let Some(bytes)=read_optional(&path)?{let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;if let Some(entry)=value.pointer("/build/entry").and_then(serde_json::Value::as_str){safe_relative(Path::new(entry))?;return check_setup(&root.join(entry));}return Ok(());}}
 if let Some(entry)=["src/main.tsr","src/main.tsrx","main.tsr","main.tsrx"].into_iter().map(|v|cwd.join(v)).find(|p|p.is_file()){check_setup(&entry)?;}Ok(())
}

#[cfg(test)]mod setup_tests {
 use super::*;
 fn scratch()->PathBuf{let root=std::env::temp_dir().join(format!("tsrust-ui-setup-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));fs::create_dir(&root).unwrap();root}
 fn button()->Vec<Identity>{vec![Identity{name:"button".into(),version:"1.0.0".into(),variant:"default".into()}]}
 #[test]fn fills_missing_slots_preserves_existing_app_and_custom_theme(){
  let root=scratch();fs::create_dir(root.join("src")).unwrap();fs::write(root.join("src/start.tsr"),"// owner entry\n").unwrap();
  let value=serde_json::json!({"schemaVersion":1,"name":"owner","build":{"entry":"src/start.tsr","output":"target/owner"},"paths":{"@/*":"src/*"},"ui":{"components":"src/custom"}});fs::write(root.join("tsrust.config.json"),value.to_string()).unwrap();
  let first=setup_plan(&root,&button(),None,"compiler",None).unwrap();apply(&root,&first).unwrap();let mut config:serde_json::Value=serde_json::from_slice(&fs::read(root.join("tsrust.config.json")).unwrap()).unwrap();assert_eq!(config["paths"],value["paths"]);assert_eq!(config["build"],value["build"]);assert!(config["ui"]["theme"].is_object());assert!(root.join("src/custom/button.tsrx").is_file());assert!(!root.join("src/main.tsr").exists());
  config["ui"]["theme"]["light"]["primary"]="#123456".into();fs::write(root.join("tsrust.config.json"),config.to_string()).unwrap();let before=fs::read(root.join("tsrust.config.json")).unwrap();let lock=fs::read(root.join("ui-components.lock.json")).unwrap();
  apply(&root,&setup_plan(&root,&button(),None,"new-compiler",None).unwrap()).unwrap();assert_eq!(fs::read(root.join("tsrust.config.json")).unwrap(),before);assert_eq!(fs::read(root.join("ui-components.lock.json")).unwrap(),lock);fs::remove_dir_all(root).unwrap();
 }
 #[test]fn stale_config_and_occupied_directories_fail_before_publication(){
  let root=scratch();let first=setup_plan(&root,&button(),None,"compiler",None).unwrap();fs::write(root.join("tsrust.config.json"),"owner changed it").unwrap();assert!(apply(&root,&first).is_err());assert!(!root.join("src").exists());
  fs::remove_file(root.join("tsrust.config.json")).unwrap();fs::write(root.join("src"),"occupied").unwrap();assert!(setup_plan(&root,&button(),None,"compiler",None).is_err());assert!(!root.join("ui-components.lock.json").exists());fs::remove_dir_all(root).unwrap();
 }
 #[test]fn malformed_and_unsafe_configuration_never_gets_replaced(){
  for ui in [serde_json::json!({"theme":null}),serde_json::json!({"components":"../outside"}),serde_json::json!({"lock":"tsrust.config.json"}),serde_json::json!({"surprise":true})]{
   let root=scratch();let config=serde_json::json!({"schemaVersion":1,"name":"owner","build":{"entry":"src/main.tsr","output":"target/app"},"ui":ui}).to_string();fs::write(root.join("tsrust.config.json"),&config).unwrap();assert!(setup_plan(&root,&button(),None,"compiler",None).is_err());assert_eq!(fs::read_to_string(root.join("tsrust.config.json")).unwrap(),config);assert!(!root.join("src").exists());fs::remove_dir_all(root).unwrap();
  }
 }
 #[test]fn invalid_json_and_staging_failure_leave_owner_bytes_intact(){
  let root=scratch();for value in ["[1]","null","\"literal\"","not-json"]{fs::write(root.join("tsrust.config.json"),value).unwrap();assert!(setup_plan(&root,&button(),None,"compiler",None).is_err());assert_eq!(fs::read_to_string(root.join("tsrust.config.json")).unwrap(),value);}
  fs::remove_file(root.join("tsrust.config.json")).unwrap();let plan=setup_plan(&root,&button(),None,"compiler",None).unwrap();fs::create_dir(root.join("src")).unwrap();let foreign=root.join(format!("src/.tsrust-ui-{}-1.tmp",std::process::id()));fs::write(&foreign,"owner file").unwrap();
  assert!(apply(&root,&plan).is_err());assert_eq!(fs::read_to_string(&foreign).unwrap(),"owner file");assert!(!root.join("src/components/ui/button.tsrx").exists());assert!(!root.join("tsrust.config.json").exists());assert!(!root.join("ui-components.lock.json").exists());fs::remove_dir_all(root).unwrap();
 }
 #[cfg(unix)]#[test]fn readonly_configuration_is_rejected_before_creating_sources(){
  use std::os::unix::fs::PermissionsExt;let root=scratch();let path=root.join("tsrust.config.json");fs::write(&path,r#"{"schemaVersion":1,"name":"owner","build":{"entry":"src/main.tsr","output":"target/app"}}"#).unwrap();let plan=setup_plan(&root,&button(),None,"compiler",None).unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o444)).unwrap();assert!(apply(&root,&plan).is_err());assert!(!root.join("src").exists());fs::set_permissions(&path,fs::Permissions::from_mode(0o644)).unwrap();fs::remove_dir_all(root).unwrap();
 }
 #[test]fn import_detection_ignores_comments_and_string_literals(){
  assert!(tsrust_compiler::ui_project::source_imports("// import {Text} from 'std:ui';\nconst x=\"import {Text} from 'std:ui';\";").unwrap().is_empty());
  assert_eq!(tsrust_compiler::ui_project::source_imports("import {Text as Label} from 'std:ui';").unwrap(),vec!["std:ui"]);
 }
}

fn parse_targets(flags:&[String],mut target:Targets)->Result<Targets,String>{
 for flag in flags{match flag.as_str(){"native"|"1"=>target.native=true,"web"|"2"=>target.web=true,"embedded"=>target.embedded=true,_=>return Err(format!("Unknown UI target: {flag}. Choose native or web."))}}target.validate()?;Ok(target)
}
pub(super) fn select_targets(existing:Option<&Targets>,flags:&[String],noninteractive:bool)->Result<Targets,String>{
 if !flags.is_empty(){return parse_targets(flags,existing.cloned().unwrap_or_default());}
 if let Some(target)=existing{target.validate()?;return Ok(target.clone());}
 use std::io::{IsTerminal,Write};
 if noninteractive || !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal(){return Err("No UI target selected. Pass --target native, --target web, or repeat both flags. Interactive selection requires a terminal; --dry-run never prompts.".into());}
 loop{
  eprint!("UI targets: 1) native desktop  2) web. Choose one or both (for example 1,2): ");std::io::stderr().flush().map_err(|e|e.to_string())?;
  let mut line=String::new();if std::io::stdin().read_line(&mut line).map_err(|e|e.to_string())?==0{return Err("UI target selection ended before a choice; no files created".into());}
  let flags=line.split(|ch:char|ch==',' || ch.is_whitespace()).filter(|v|!v.is_empty()).map(str::to_owned).collect::<Vec<_>>();
  match parse_targets(&flags,Targets::default()){Ok(target)=>return Ok(target),Err(error)=>eprintln!("{error}")}
 }
}
#[cfg(test)]fn setup_plan(root:&Path,requests:&[Identity],destination:Option<&Path>,compiler_hash:&str,_entry:Option<&Path>)->Result<Plan,String>{setup_plan_with_targets(root,requests,destination,compiler_hash,&["native".into()],true)}

#[cfg(test)]mod target_tests{
 use super::*;
 #[test]fn explicit_flags_can_enable_one_or_both_backends(){
  let web=parse_targets(&["web".into()],Targets::default()).unwrap();assert!(!web.native && web.web && !web.embedded);
  let both=parse_targets(&["native".into()],web).unwrap();assert!(both.native && both.web);
  for flags in [vec![],vec!["embedded".into()],vec!["unknown".into()]]{assert!(parse_targets(&flags,Targets::default()).is_err());}
  assert!(select_targets(None,&[],true).unwrap_err().contains("--target"));
 }
 #[test]fn config_rejects_empty_unknown_nonboolean_and_unsupported_targets(){
  for target in [serde_json::json!({}),serde_json::json!({"native":false,"web":false}),serde_json::json!({"native":"true"}),serde_json::json!({"mobile":true}),serde_json::json!({"web":true,"embedded":true})]{let mut config=serde_json::to_value(Config::default()).unwrap();config["target"]=target;assert!(Config::parse(&config).is_err());}
 }
}
