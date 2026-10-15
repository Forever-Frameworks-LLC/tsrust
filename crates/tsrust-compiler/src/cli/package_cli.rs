//! Cargo-backed package changes publish one validated, atomic configuration generation.
use std::{collections::BTreeMap,fs,io::Write,path::{Path,PathBuf}};
use serde_json::Value;
use tsrust_compiler::project::{SourceId,rust::dependencies::{Bindings,Dependency}};
use crate::package_validation::{Candidate,normalize,read_optional};
#[derive(Default)]
struct Options {root:Option<PathBuf>,manifest:Option<PathBuf>,source:Option<String>,name:Option<String>,version:Option<String>,features:Option<Vec<String>>,enable:Vec<String>,disable:Vec<String>,defaults:Option<bool>,offline:bool,preview:bool}
fn list(value:String)->Vec<String>{value.split(',').filter(|value|!value.is_empty()).map(str::to_owned).collect()}
fn options(command:&str,args:impl Iterator<Item=String>)->Result<Options,String>{
 let mut options=Options::default();let mut args=args.peekable();
 while let Some(arg)=args.next(){match arg.as_str(){
  "--project"=>options.root=Some(PathBuf::from(args.next().ok_or("--project needs a directory")?)),
  "--bindings"=>options.manifest=Some(PathBuf::from(args.next().ok_or("--bindings needs a path")?)),
  "--source"=>options.source=Some(args.next().ok_or("--source needs a relative .rs path")?),
  "--to"=>{let version=args.next().ok_or("--to needs an exact version")?;if options.version.replace(version).is_some(){return Err("Specify the version once.".into());}},
  "--features"=>options.features=Some(list(args.next().ok_or("--features needs a comma-separated set (empty clears it)")?)),
  "--enable"=>options.enable.extend(list(args.next().ok_or("--enable needs features")?)),
  "--disable"=>options.disable.extend(list(args.next().ok_or("--disable needs features")?)),
  "--no-default-features"|"--default-features"=>{let enabled=arg=="--default-features";if options.defaults.is_some_and(|old|old!=enabled){return Err("Conflicting default-feature options.".into());}options.defaults=Some(enabled);},
  "--offline"=>options.offline=true,"--dry-run"=>options.preview=true,
  _ if arg.starts_with('-')=>return Err(format!("Unknown package option {arg}")),
  _ if options.name.is_none()=>{let (name,version)=arg.split_once('@').map_or((arg.as_str(),None),|(name,version)|(name,Some(version.to_owned())));options.name=Some(name.into());if let Some(version)=version{if options.version.replace(version).is_some(){return Err("Specify the version once.".into());}}},
  _=>return Err("Expected one package name.".into())
 }}
 if command!="packages" && options.name.is_none(){return Err(format!("Usage: tsrust {command} <crate> [--project directory] [--bindings path] [--offline] [--dry-run]"));}
 if command=="packages" && (options.name.is_some() || options.source.is_some() || options.version.is_some() || options.features.is_some() || !options.enable.is_empty() || !options.disable.is_empty() || options.defaults.is_some() || options.preview){return Err("packages only accepts --project, --bindings and --offline.".into());}
 if !matches!(command,"add"|"update") && options.version.is_some(){return Err("Only add/update accepts a version.".into());}
 if options.source.is_some() && command!="add"{return Err("--source applies only when adding a package.".into());}
 if command=="remove" && (options.features.is_some() || !options.enable.is_empty() || !options.disable.is_empty() || options.defaults.is_some()){return Err("remove does not accept feature options.".into());}
 if options.enable.iter().any(|value|options.disable.contains(value)){return Err("A feature cannot be both enabled and disabled.".into());}
 Ok(options)
}
pub(crate) fn run(command:&str,args:impl Iterator<Item=String>)->Result<u8,String>{
 let mut options=options(command,args)?;
 if let Some(source)=&mut options.source{if !source.starts_with("./") && !source.starts_with("../"){*source=format!("./{source}");}}
 let root=match options.root.clone(){Some(root)=>root,None=>crate::project_config::default_root()?};
 let root=fs::canonicalize(root).map_err(|error|error.to_string())?;
 let path=select_manifest(&root,options.manifest.as_deref())?;
 if !path.starts_with(&root){return Err("Binding configuration must belong to the selected project.".into());}
 let lock_root=path.parent().unwrap().join("target/tsrust-packages");fs::create_dir_all(&lock_root).map_err(|error|error.to_string())?;
 fs::write(lock_root.join(".gitignore"),"*\n").map_err(|error|error.to_string())?;
 let lock=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(lock_root.join(".lock")).map_err(|error|error.to_string())?;
 lock.lock().map_err(|error|error.to_string())?;
 if select_manifest(&root,options.manifest.as_deref())?!=path{return Err("Binding selection changed while waiting for the project lock.".into());}
 let previous=read_optional(&path)?;
 if command!="add" && previous.is_none(){return Err("No binding package configuration exists; use tsrust add first.".into());}
 let mut binding=match &previous{Some(text)=>Bindings::parse(text)?,None=>Bindings{schema_version:1,source:options.source.clone().unwrap_or("./native.rs".into()),dependencies:BTreeMap::new(),lockfile:None}};
 if let Some(source)=&options.source {if previous.is_some() && source!=&binding.source{return Err("Package changes cannot replace an existing adapter source.".into());}binding.source=source.clone();}
 Bindings::parse(&serde_json::to_string(&binding).map_err(|error|error.to_string())?)?;
 let native=normalize(&path.parent().unwrap().join(&binding.source))?;
 let old_source=read_optional(&native)?;
 if old_source.is_none() && !native.starts_with(&root){return Err("New adapters must be created within the selected project.".into());}
 let native_source=old_source.clone().unwrap_or_else(||if options.name.as_deref()==Some("regex"){include_str!("../../../../examples/rust-regex/regex.rs").into()}else{"// Export selected public functions and owned data for Tsrust here.\n".into()});
 let legacy=binding.lockfile.is_none();
 let old_lock=match &binding.lockfile{Some(lock)=>Some(lock.clone()),None=>read_optional(&Bindings::lock_path(&path))?};
 let temporary=crate::TempDirectory::new().map_err(|error|error.to_string())?;let cargo=temporary.0.join("cargo");fs::create_dir(&cargo).map_err(|error|error.to_string())?;
 if command=="packages" || command=="features" && options.features.is_none() && options.enable.is_empty() && options.disable.is_empty() && options.defaults.is_none() {
  let old_lock=old_lock.ok_or("Missing lockfile; resolve this configuration first.")?;
  prepare(&cargo,&binding,Some(&old_lock))?;
  let package=binding.package(&path,old_lock);
  let config=format!("[source.crates-io]\nreplace-with='vendored'\n[source.vendored]\ndirectory={:?}\n",package.vendor.to_string_lossy());fs::write(cargo.join("vendor.toml"),config).map_err(|error|error.to_string())?;
  let metadata=cargo_output(&cargo,&["metadata","--format-version","1","--offline","--locked","--config","vendor.toml"],false)?;
  let metadata:Value=serde_json::from_slice(&metadata).map_err(|error|error.to_string())?;
  report(&metadata,&binding,None);
  for name in binding.dependencies.keys().filter(|name|options.name.as_ref().is_none_or(|selected|selected==*name)){if let Ok((_,node))=resolved(&metadata,name){let package=metadata["packages"].as_array().unwrap().iter().find(|package|package["id"]==node["id"]).unwrap();if command=="features"{for (feature,enables) in package["features"].as_object().unwrap(){println!("{} {feature}: {}",if node["features"].as_array().unwrap().iter().any(|enabled|enabled==feature){"+"}else{"-"},enables.as_array().unwrap().iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "));}}}}
  let provider=Candidate::new(BTreeMap::new())?;
  let graph=tsrust_compiler::project::load_project_headers(&provider,path.to_str().ok_or("Non-UTF8 path")?).map_err(|error|error.message)?;
  println!("Adapter: {}",native.display());println!("Exports: {}",graph.modules[&graph.entry].exports.keys().cloned().collect::<Vec<_>>().join(", "));
  if graph.modules[&graph.entry].exports.is_empty(){println!("No Tsrust bindings are exported; write public Rust adapter declarations.");}return Ok(0);
 }
 let name=options.name.as_ref().unwrap();
 let existing=binding.dependencies.get(name).cloned();
 if command!="add" && existing.is_none(){return Err(format!("Package {name} is not declared."));}
 if command=="remove"{binding.dependencies.remove(name);}else{
  let mut dependency=existing.clone().unwrap_or(Dependency{version:"=0.0.0".into(),features:vec![],default_features:true});
  if let Some(version)=&options.version {dependency.version=format!("={}",version.trim_start_matches('='));}
  if let Some(features)=&options.features{dependency.features=features.clone();}
  for feature in &options.enable{if !dependency.features.contains(feature){dependency.features.push(feature.clone());}}
  dependency.features.retain(|feature|!options.disable.contains(feature));dependency.features.sort();dependency.features.dedup();
  if let Some(defaults)=options.defaults{dependency.default_features=defaults;}
  binding.dependencies.insert(name.clone(),dependency);
 }
 // Validate names and feature syntax before invoking Cargo; versions are pinned after resolution.
 Bindings::parse(&serde_json::to_string(&binding).map_err(|error|error.to_string())?)?;
 let latest=command=="add" && existing.is_none() && options.version.is_none();
 let latest_dependency=binding.dependencies.get(name).cloned();
 if latest{binding.dependencies.remove(name);}
 if command=="update" && options.version.is_none(){binding.dependencies.get_mut(name).unwrap().version=format!("^{}",existing.as_ref().unwrap().version.trim_start_matches('='));}
 prepare(&cargo,&binding,old_lock.as_deref())?;
 if latest {
  let mut args=vec!["add".to_owned(),name.clone()];
  let dependency=latest_dependency.as_ref().unwrap();
  if !dependency.features.is_empty(){args.extend(["--features".into(),dependency.features.join(",")]);}
  if !dependency.default_features{args.push("--no-default-features".into());}
  cargo_owned(&cargo,&args,options.offline)?;
 }else if command=="update"{cargo_output(&cargo,&["update","--package",name],options.offline)?;}
 let metadata:Value=serde_json::from_slice(&cargo_output(&cargo,&["metadata","--format-version","1"],options.offline)?).map_err(|error|error.to_string())?;
 let root_package=root_package(&metadata)?;
 for dependency in root_package["dependencies"].as_array().ok_or("Cargo dependencies missing")? {
  let name=dependency["name"].as_str().ok_or("Cargo dependency name missing")?;
  if matches!(name,"rayon"|"serde"|"serde_json"|"tokio"|"axum"|"reqwest"|"http-body"|"http-body-util"|"hyper-util"|"sqlx"|"rusqlite"|"libsqlite3-sys"|"tokio-rustls"){continue;}
  let version=resolved(&metadata,name)?.0;
  binding.dependencies.insert(name.into(),Dependency{version:format!("={version}"),features:dependency["features"].as_array().unwrap().iter().map(|feature|feature.as_str().unwrap().into()).collect(),default_features:dependency["uses_default_features"].as_bool().unwrap()});
 }
 Bindings::parse(&serde_json::to_string(&binding).map_err(|error|error.to_string())?)?;
 // Cargo add may have used a range. Re-pin without re-resolving unrelated packages.
 fs::write(cargo.join("Cargo.toml"),binding.manifest()).map_err(|error|error.to_string())?;
 cargo_output(&cargo,&["metadata","--format-version","1","--locked"],options.offline)?;
 let lockfile=fs::read_to_string(cargo.join("Cargo.lock")).map_err(|error|error.to_string())?;
 binding.lockfile=Some(lockfile.clone());let encoded=serde_json::to_string_pretty(&binding).map_err(|error|error.to_string())?+"\n";
 let vendor=temporary.0.join("vendor");let vendor_text=vendor.to_str().ok_or("Non-UTF8 vendor path")?;
 cargo_output(&cargo,&["vendor","--locked","--versioned-dirs",vendor_text],options.offline)?;
 let package=binding.package(&path,lockfile);let created_vendor=install_vendor(&vendor,&package.vendor)?;
 let mut cache=CandidateCache{root:package.vendor.parent().unwrap().to_owned(),remove:created_vendor};
 let mut overlay=BTreeMap::from([(SourceId(path.to_string_lossy().into_owned()),encoded.clone())]);
 if old_source.is_none(){overlay.insert(SourceId(native.to_string_lossy().into_owned()),native_source.clone());}
 let provider=Candidate::new(overlay)?;
 report(&metadata,&binding,old_lock.as_deref());
 let (tests,exports)=crate::package_validation::validate(&provider,&root,&path,&temporary.0)?;
 println!("Adapter: {}",native.display());
 if exports.is_empty(){println!("No Tsrust bindings exported; the crate is available to custom public Rust declarations.");}else{println!("Bindings: {}",exports.join(", "));}
 println!("Validated Rust adapters and project callers; executed {tests} test file(s).");if tests==0{println!("No project test files found; behavioral compatibility is unverified.");}
 if options.preview{println!("Dry run passed; working setup unchanged.");return Ok(0);}
 provider.unchanged()?;
 if read_optional(&path)?!=previous || read_optional(&native)?!=old_source || legacy && read_optional(&Bindings::lock_path(&path))?!=old_lock{return Err("Package inputs changed during validation; nothing applied.".into());}
 let mut created=false;
 if old_source.is_none(){let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&native).map_err(|error|error.to_string())?;if let Err(error)=file.write_all(native_source.as_bytes()).and_then(|_|file.sync_all()).and_then(|_|fs::File::open(native.parent().unwrap())?.sync_all()){drop(file);let _=fs::remove_file(&native);return Err(error.to_string());}created=true;}
 if let Err(error)=publish(&path,&encoded){if created{let _=fs::remove_file(&native);}return Err(error);}
 cache.remove=false;
 println!("Applied {command} {name}. Import bindings from {}",path.display());Ok(0)
}
fn select_manifest(root:&Path,explicit:Option<&Path>)->Result<PathBuf,String>{
 if let Some(path)=explicit{let path=normalize(&if path.is_absolute(){path.to_owned()}else{root.join(path)})?;if !path.to_string_lossy().ends_with(".bindings.json"){return Err("--bindings requires a .bindings.json path.".into());}return Ok(path);}
 fn visit(root:&Path,result:&mut Vec<PathBuf>)->Result<(),String>{for entry in fs::read_dir(root).map_err(|error|error.to_string())?{let entry=entry.map_err(|error|error.to_string())?;let kind=entry.file_type().map_err(|error|error.to_string())?;if kind.is_symlink(){continue;}if kind.is_dir() && !matches!(entry.file_name().to_str(),Some("target"|".git"|"node_modules")){visit(&entry.path(),result)?;}else if kind.is_file() && entry.file_name().to_string_lossy().ends_with(".bindings.json"){result.push(entry.path());}}Ok(())}
 let mut files=Vec::new();visit(root,&mut files)?;match files.as_slice(){[]=>Ok(root.join("native.bindings.json")),[path]=>normalize(path),_=>Err("Multiple binding configurations found; select --bindings explicitly.".into())}
}
fn prepare(root:&Path,binding:&Bindings,lock:Option<&str>)->Result<(),String>{fs::create_dir(root.join("src")).map_err(|error|error.to_string())?;fs::write(root.join("src/lib.rs"),"").map_err(|error|error.to_string())?;fs::write(root.join("Cargo.toml"),binding.manifest()).map_err(|error|error.to_string())?;if let Some(lock)=lock{fs::write(root.join("Cargo.lock"),lock).map_err(|error|error.to_string())?;}Ok(())}
fn cargo_owned(root:&Path,args:&[String],offline:bool)->Result<Vec<u8>,String>{let args=args.iter().map(String::as_str).collect::<Vec<_>>();cargo_output(root,&args,offline)}
fn cargo_output(root:&Path,args:&[&str],offline:bool)->Result<Vec<u8>,String>{let mut command=crate::backend_cargo_command();command.current_dir(root).args(args);if offline{command.arg("--offline");}let output=command.output().map_err(|error|error.to_string())?;if !output.status.success(){return Err(format!("Cargo failed; working setup unchanged:\n{}",String::from_utf8_lossy(&output.stderr)));}Ok(output.stdout)}
fn root_package(metadata:&Value)->Result<&Value,String>{let id=metadata["resolve"]["root"].as_str().ok_or("Cargo root missing")?;metadata["packages"].as_array().unwrap().iter().find(|package|package["id"]==id).ok_or("Cargo root package missing".into())}
fn resolved<'a>(metadata:&'a Value,name:&str)->Result<(&'a str,&'a Value),String>{let root=metadata["resolve"]["root"].as_str().ok_or("Cargo root missing")?;let node=metadata["resolve"]["nodes"].as_array().unwrap().iter().find(|node|node["id"]==root).unwrap();let id=&node["deps"].as_array().unwrap().iter().find(|dependency|dependency["name"]==name.replace('-',"_")).ok_or_else(||format!("Cargo dependency {name} missing"))?["pkg"];let package=metadata["packages"].as_array().unwrap().iter().find(|package|&package["id"]==id).unwrap();let node=metadata["resolve"]["nodes"].as_array().unwrap().iter().find(|node|&node["id"]==id).unwrap();Ok((package["version"].as_str().unwrap(),node))}
fn report(metadata:&Value,binding:&Bindings,old:Option<&str>){
 for (name,dependency) in &binding.dependencies {if let Ok((version,node))=resolved(metadata,name){println!("{name}: declared {}, resolved {version}; defaults {}; declared features [{}]; effective features {}",dependency.version,dependency.default_features,dependency.features.join(","),node["features"]);for parent in metadata["resolve"]["nodes"].as_array().unwrap().iter().filter(|parent|parent["deps"].as_array().unwrap().iter().any(|dependency|dependency["pkg"]==node["id"])) {
 let package=metadata["packages"].as_array().unwrap().iter().find(|package|package["id"]==parent["id"]).unwrap();
 for request in package["dependencies"].as_array().unwrap().iter().filter(|request|request["name"]==name.as_str()){println!("  requested by {} {}: features {}, defaults {}",package["name"].as_str().unwrap(),package["version"].as_str().unwrap(),request["features"],request["uses_default_features"]);}
}}}
 if let Some(old)=old {let before=lock_packages(old);let after=lock_packages(binding.lockfile.as_deref().unwrap_or(""));for (name,version) in before.difference(&after){println!("- {name} {version}");}for (name,version) in after.difference(&before){println!("+ {name} {version}");}}
}
fn install_vendor(source:&Path,destination:&Path)->Result<bool,String>{let parent=destination.parent().unwrap();fs::create_dir_all(parent).map_err(|error|error.to_string())?;fs::write(parent.parent().unwrap().join(".gitignore"),"*\n").map_err(|error|error.to_string())?;let lock=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(parent.join(".lock")).map_err(|error|error.to_string())?;lock.lock().map_err(|error|error.to_string())?;if destination.is_dir(){return Ok(false);}let staged=parent.join(format!("pending-{}",std::process::id()));if let Err(error)=crate::rust_binding_packages::copy(source,&staged){let _=fs::remove_dir_all(&staged);return Err(error);}fs::rename(staged,destination).map_err(|error|error.to_string())?;Ok(true)}
fn publish(path:&Path,text:&str)->Result<(),String>{let staged=path.with_extension(format!("json-pending-{}",std::process::id()));let mut created=false;let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&staged).map_err(|error|error.to_string())?;created=true;file.write_all(text.as_bytes()).and_then(|_|file.sync_all()).map_err(|error|error.to_string())?;fs::rename(&staged,path).map_err(|error|error.to_string())?;if let Err(error)=fs::File::open(path.parent().unwrap()).and_then(|file|file.sync_all()){eprintln!("Configuration committed, but directory sync failed: {error}");}Ok(())})();if created && result.is_err(){let _=fs::remove_file(staged);}result}

fn lock_packages(text:&str)->std::collections::BTreeSet<(String,String)>{
 text.split("[[package]]").filter_map(|block|{let get=|key:&str|block.lines().find_map(|line|line.strip_prefix(key).and_then(|text|serde_json::from_str::<String>(text).ok()));Some((get("name = ")?,get("version = ")?))}).collect()
}

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn failed_or_interrupted_publication_keeps_the_old_generation(){
  let dir=crate::TempDirectory::new().unwrap();let path=dir.0.join("api.bindings.json");let old="{\"generation\":0,\"lock\":0}";fs::write(&path,old).unwrap();
  let pending=path.with_extension(format!("json-pending-{}",std::process::id()));fs::write(&pending,"{\"unfinished\":").unwrap();
  assert!(publish(&path,"{\"generation\":1,\"lock\":1}").is_err());assert_eq!(fs::read_to_string(&path).unwrap(),old);assert_eq!(fs::read_to_string(&pending).unwrap(),"{\"unfinished\":");fs::remove_file(pending).unwrap();
  let reading=path.clone();let active=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));let running=active.clone();
  let reader=std::thread::spawn(move||while running.load(std::sync::atomic::Ordering::SeqCst){let value:Value=serde_json::from_str(&fs::read_to_string(&reading).unwrap()).unwrap();assert_eq!(value["generation"],value["lock"]);});
  for generation in 1..20{publish(&path,&format!("{{\"generation\":{generation},\"lock\":{generation}}}")).unwrap();}active.store(false,std::sync::atomic::Ordering::SeqCst);reader.join().unwrap();
 }
 #[test]fn options_reject_ambiguous_or_unsupported_changes(){
  for args in [vec!["regex","--default-features","--no-default-features"],vec!["regex","--enable","unicode","--disable","unicode"],vec!["regex","--unknown"]]{assert!(options("features",args.into_iter().map(str::to_owned)).is_err());}
 }
}

struct CandidateCache{root:PathBuf,remove:bool}
impl Drop for CandidateCache{fn drop(&mut self){if self.remove{let _=fs::remove_dir_all(&self.root);}}}
