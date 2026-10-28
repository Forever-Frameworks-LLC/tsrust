//! Capture the entry's resolved module closure, including paths outside the root.
use std::{cell::{Cell,RefCell},collections::BTreeMap,path::{Path,PathBuf}};
use tsrust_compiler::project::{SourceId,SourceProvider,FileSourceProvider};
use crate::dev_watch::{Contents,Inputs,Stamp};
struct Provider {invalid:Cell<bool>,inputs:RefCell<Inputs>,observed:RefCell<BTreeMap<PathBuf,Option<Stamp>>>}
impl Provider {
 fn observe(&self,path:&Path)->Option<Stamp>{let stamp=Stamp::read(path).ok();self.observed.borrow_mut().insert(path.to_owned(),stamp.clone());stamp}
 fn file(&self,path:&Path)->Result<Option<String>,String>{
  let before=self.observe(path);
  if let Some(contents)=self.inputs.borrow().files.get(path){return String::from_utf8(contents.bytes().to_vec()).map(Some).map_err(|error|error.to_string());}
  let text=match std::fs::read_to_string(path){Ok(text)=>text,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(format!("{}: {error}",path.display()))};
  if self.observe(path)!=before{self.invalid.set(true);return Err(format!("{} changed during dependency capture",path.display()));}
  self.inputs.borrow_mut().files.insert(path.to_owned(),Contents::new(text.as_bytes()));Ok(Some(text))
 }
 fn resolve_file(&self,importer:Option<&SourceId>,specifier:&str)->Result<SourceId,String>{
  let candidates=if let Some(importer)=importer {
   let aliases=tsrust_compiler::project_resolution::discover(Path::new(&importer.0),|path|self.file(path))?;
   tsrust_compiler::project_resolution::candidates(Path::new(&importer.0),specifier,aliases.as_ref())?
  }else{vec![PathBuf::from(specifier)]};
  for candidate in &candidates {
   self.observe(candidate);
   match std::fs::metadata(candidate){
    Ok(metadata) if metadata.is_file()=>{let canonical=std::fs::canonicalize(candidate).map_err(|error|error.to_string())?;self.observe(&canonical);return Ok(SourceId(canonical.to_str().ok_or("Source path must be valid UTF-8.")?.into()));},
    Ok(_)=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(format!("{}: {error}",candidate.display())),
   }
  }
  Err(format!("Source {specifier} is absent from this build snapshot; tried {}",candidates.iter().map(|path|path.display().to_string()).collect::<Vec<_>>().join(", ")))
 }
}
impl SourceProvider for Provider {
 fn resolve(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{
  let result=self.resolve_file(importer,path);self.inputs.borrow_mut().resolutions.insert((importer.cloned(),path.into()),result.clone());result
 }
 fn read(&self,id:&SourceId)->Result<String,String>{
  if id.0.starts_with("std:"){return FileSourceProvider.read(id);}
  let result=self.file(Path::new(&id.0)).and_then(|text|text.ok_or_else(||format!("Missing source {}",id.0)));
  if let Err(error)=&result{self.inputs.borrow_mut().failures.insert(PathBuf::from(&id.0),error.clone());}
  result
 }
}
pub(crate) fn capture(root:&Path,files:BTreeMap<PathBuf,Contents>)->Result<(Inputs,BTreeMap<PathBuf,Option<Stamp>>),String>{
 let config=root.join(crate::project_config::FILE);
 let entry=files.get(&config).and_then(|text|std::str::from_utf8(text.bytes()).ok()).and_then(|text|crate::project_config::parse(root,text).ok()).map(|project|project.entry);
 let provider=Provider{invalid:Cell::new(false),inputs:RefCell::new(Inputs{files,failures:BTreeMap::new(),resolutions:BTreeMap::new()}),observed:RefCell::new(BTreeMap::new())};
 if let Some(entry)=entry {
  // Header loading records dependency identities even when checking will fail.
  // The ordinary frontend owns the diagnostic; capture must retain its inputs.
  let _=tsrust_compiler::project::load_project_headers(&provider,entry.to_str().ok_or("Source path must be valid UTF-8.")?);
 }
 if provider.invalid.get(){return Err("Dependency changed during capture; retrying".into());}
 for(path,stamp)in provider.observed.borrow().iter(){if Stamp::read(path).ok()!=*stamp{return Err(format!("{} changed during dependency capture",path.display()));}}
 // Native events already cover ordinary in-root paths. Poll only dependencies
 // outside that tree (including excluded directories reached through imports).
 let observed=provider.observed.into_inner().into_iter().filter(|(path,_)|{
  let mut normalized=PathBuf::new();for part in path.components(){match part{std::path::Component::CurDir=>{},std::path::Component::ParentDir=>{normalized.pop();},other=>normalized.push(other.as_os_str())}}
  normalized.strip_prefix(root).map_or(true,|relative|relative.components().any(|part|matches!(part.as_os_str().to_str(),Some(".git"|"node_modules"|"target"))))
 }).collect();
 Ok((provider.inputs.into_inner(),observed))
}
