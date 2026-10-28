//! Project notices are shared files; source retains a short relative reference.
use super::*;
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
pub(super) struct Attribution {pub path:PathBuf,pub sha256:String}
pub(super) fn record(text:&str)->Result<Attribution,String> {
 if text.trim().is_empty(){return Err("Bundled source has no license notice".into());}
 let digest=hash(text.as_bytes());
 let name=if text==include_str!("../../../../attributions/shadcn-MIT.txt"){"shadcn-MIT.txt".to_owned()}
 else if text==include_str!("../../../../attributions/tsrust-MIT.txt"){"tsrust-MIT.txt".to_owned()}
 else {format!("notice-{digest}.txt")};
 Ok(Attribution{path:portable_generated_path(&PathBuf::from("attributions").join(name))?,sha256:digest})
}
pub(super) fn ensure(root:&Path,notice:&Attribution,text:&str,files:&mut BTreeMap<PathBuf,Vec<u8>>,messages:&mut Vec<String>)->Result<(),String> {
 let path=no_symlinks(root,&notice.path)?;
 if let Some(bytes)=read_optional(&path)?.or_else(||files.get(&notice.path).cloned()) {
  if hash(&bytes)!=notice.sha256{return Err(format!("Attribution {} differs from its recorded notice; restore it or preserve your notice under a different filename",notice.path.display()));}
 }else{files.insert(notice.path.clone(),text.as_bytes().to_vec());messages.push(format!("create {} (shared license notice; include when distributing source)",notice.path.display()));}
 Ok(())
}
pub(super) fn source(source:&str,license:&str,text:&str,path:&Path,notice:&Attribution)->Result<String,String> {
 let from=path.parent().unwrap_or(Path::new(""));
 let relative=tsrust_compiler::native_path_relative::relative_host(from.to_str().ok_or("Invalid UI path")?,notice.path.to_str().ok_or("Invalid attribution path")?)?.replace('\\',"/");
 let reference=format!("// License: {license}. See {relative}.\n");
 if source.starts_with(&reference){return Ok(source.to_owned());}
 if source.starts_with("/*") && let Some(end)=source.find("*/") {
  let header=&source[2..end];
  if !text.trim().is_empty() && header.contains(text.trim()) {
   let remaining=header.replacen(text.trim(),"",1);let remaining=remaining.trim();
   let provenance=if remaining.is_empty(){String::new()}else{format!("/*\n{remaining}\n*/\n")};
   return Ok(format!("{reference}{provenance}{}",source[end+2..].trim_start_matches(['\r','\n'])));
  }
 }
 Ok(reference+source)
}
/// Explicitly move legacy notices without regenerating any application/component body.
pub(super) fn migrate(root:&Path,config:&Config)->Result<Plan,String> {
 let(mut lock,previous)=read_lock(root,config)?;
 if previous.is_none(){return Err("No installed UI components to attribute; run tsrust ui add first".into());}
 let mut files=BTreeMap::new();let mut updates=Vec::new();let mut messages=Vec::new();
 for entry in &mut lock.installed {
  let text=match (&entry.license_text,&entry.attribution) {
   (Some(text),_)=>text.clone(),
   (None,Some(notice))=>{let text=fs::read_to_string(no_symlinks(root,&notice.path)?).map_err(|e|format!("Missing attribution {}: {e}",notice.path.display()))?;if hash(text.as_bytes())!=notice.sha256{return Err(format!("Attribution {} changed; restore its recorded notice",notice.path.display()));}text},
   _=>return Err("Installed component has no license notice or attribution".into()),
  };
  let notice=record(&text)?;ensure(root,&notice,&text,&mut files,&mut messages)?;
  for file in entry.files.iter().filter(|file|file.json_pointer.is_none() && file.path.extension().is_some_and(|extension|extension=="tsrx")) {
   let path=no_symlinks(root,&file.path)?;let original=fs::read(&path).map_err(|e|format!("Cannot migrate {}: {e}",file.path.display()))?;
   let generated=source(std::str::from_utf8(&original).map_err(|e|e.to_string())?,&entry.license,&text,&file.path,&notice)?.into_bytes();
   if generated!=original {messages.push(format!("shorten license header in {} (body and local edits preserved)",file.path.display()));updates.push(Update{path:file.path.clone(),previous:Some(original),bytes:generated});}
  }
  // Original generated hashes remain provenance, not a requirement to discard local edits.
  entry.attribution=Some(notice);entry.license_text=None;
 }
 let mut bytes=serde_json::to_vec_pretty(&lock).map_err(|e|e.to_string())?;bytes.push(b'\n');
 Ok(Plan{files:files.into_iter().collect(),updates,directories:vec![],lock_path:config.lock.clone(),previous,lock:bytes,messages})
}

#[cfg(test)]mod tests {
 use super::*;
 fn root()->super::super::super::TempDirectory {super::super::super::TempDirectory::new().unwrap()}
 fn request(name:&str)->Identity {Identity{name:name.into(),version:"1.0.0".into(),variant:"default".into()}}
 #[test]fn installed_sources_share_notices_with_relative_references_and_hash_provenance() {
  let dir=root();let config=Config::default();let initial=setup_plan(&dir.0,&[request("kbd"),request("separator")],Some(Path::new("local/controls")),"compiler",None).unwrap();apply(&dir.0,&initial).unwrap();
  assert_eq!(fs::read_dir(dir.0.join("attributions")).unwrap().count(),2);
  for name in ["kbd","separator"] {let source=fs::read_to_string(dir.0.join(format!("local/controls/{name}.tsrx"))).unwrap();assert!(source.starts_with("// License: MIT. See ../../attributions/shadcn-MIT.txt."));assert!(!source.contains("Permission is hereby granted"));}
  let(lock,_)=read_lock(&dir.0,&config).unwrap();assert!(lock.installed.iter().all(|entry|entry.license_text.is_none()));
  for entry in lock.installed {let notice=entry.attribution.unwrap();assert_eq!(hash(&fs::read(dir.0.join(notice.path)).unwrap()),notice.sha256);}
  let same=plan(&dir.0,&config,&[request("kbd")],Some(Path::new("local/controls")),"other compiler").unwrap();assert!(same.files.is_empty());
 }
 #[test]fn explicit_migration_preserves_edited_bodies_and_is_idempotent() {
  let dir=root();let config=Config::default();apply(&dir.0,&setup_plan(&dir.0,&[request("kbd")],None,"compiler",None).unwrap()).unwrap();
  let(mut lock,_)=read_lock(&dir.0,&config).unwrap();let items=catalog().unwrap();let mut original_payloads=Vec::new();
  for entry in &mut lock.installed {let item=items.iter().find(|item|identity(item)==entry.identity).unwrap();entry.license_text=Some(item.license_text.clone());entry.attribution=None;original_payloads.push(entry.payload_sha256.clone());}
  let component=dir.0.join(&config.components).join("kbd.tsrx");let raw=items.iter().find(|item|item.name=="kbd").unwrap().files[0].content.clone()+"\n// Owner edit stays byte-for-byte.\n";fs::write(&component,&raw).unwrap();
  fs::write(dir.0.join(&config.lock),serde_json::to_vec_pretty(&lock).unwrap()).unwrap();fs::remove_dir_all(dir.0.join("attributions")).unwrap();
  let pending=migrate(&dir.0,&config).unwrap();assert_eq!(fs::read_to_string(&component).unwrap(),raw);assert!(!dir.0.join("attributions").exists());apply(&dir.0,&pending).unwrap();
  let updated=fs::read_to_string(&component).unwrap();assert_eq!(updated.split_once('\n').unwrap().1,raw.split_once("*/\n").unwrap().1);assert!(updated.ends_with("// Owner edit stays byte-for-byte.\n"));
  let(lock,_)=read_lock(&dir.0,&config).unwrap();assert_eq!(lock.installed.iter().map(|entry|entry.payload_sha256.clone()).collect::<Vec<_>>(),original_payloads);
  let previous=fs::read(dir.0.join(&config.lock)).unwrap();let same=migrate(&dir.0,&config).unwrap();assert!(same.files.is_empty()&&same.updates.is_empty());apply(&dir.0,&same).unwrap();assert_eq!(fs::read(dir.0.join(&config.lock)).unwrap(),previous);
 }
 #[test]fn changed_missing_or_symlinked_notices_fail_without_component_writes() {
  let dir=root();let config=Config::default();let items=catalog().unwrap();let item=items.iter().find(|item|item.name=="kbd").unwrap();let notice=record(&item.license_text).unwrap();fs::create_dir(dir.0.join("attributions")).unwrap();fs::write(dir.0.join(&notice.path),"Owner file").unwrap();
  assert!(setup_plan(&dir.0,&[request("kbd")],None,"compiler",None).is_err());assert!(!dir.0.join(&config.lock).exists());assert!(!dir.0.join(&config.components).exists());
  fs::remove_file(dir.0.join(&notice.path)).unwrap();apply(&dir.0,&setup_plan(&dir.0,&[request("kbd")],None,"compiler",None).unwrap()).unwrap();fs::remove_file(dir.0.join(&notice.path)).unwrap();assert!(read_lock(&dir.0,&config).is_err());
  #[cfg(unix)] {std::os::unix::fs::symlink(std::env::temp_dir(),dir.0.join(&notice.path)).unwrap();assert!(read_lock(&dir.0,&config).is_err());}
 }
}
