//! Candidate checks use an in-memory overlay; project source files are never copied or rewritten.
use std::{cell::RefCell,collections::BTreeMap,fs,path::{Path,PathBuf}};
use tsrust_compiler::project::{SourceProvider,SourceId,FileSourceProvider};
pub(crate) struct Candidate {
 pub overlay:BTreeMap<SourceId,String>,
 observed:RefCell<BTreeMap<SourceId,Option<String>>>,
 resolutions:RefCell<Vec<(Option<SourceId>,String,SourceId)>>,
}
impl Candidate {
 pub fn new(overlay:BTreeMap<SourceId,String>)->Result<Self,String>{
  let observed=overlay.keys().map(|id|Ok((id.clone(),read_optional(Path::new(&id.0))?))).collect::<Result<_,String>>()?;
  Ok(Self{overlay,observed:RefCell::new(observed),resolutions:RefCell::new(Vec::new())})
 }
 pub fn unchanged(&self)->Result<(),String>{
  for (id,before) in self.observed.borrow().iter(){if &read_optional(Path::new(&id.0))?!=before{return Err(format!("{} changed during validation; nothing applied.",id.0));}}
  for (importer,path,before) in self.resolutions.borrow().iter(){if &self.resolve_current(importer.as_ref(),path)?!=before{return Err(format!("Resolution of {path} changed during validation; nothing applied."));}}
  Ok(())
 }
 fn resolve_current(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{
  let candidates=if let Some(importer)=importer {
   let aliases=tsrust_compiler::project_resolution::discover(Path::new(&importer.0),|path|{
    let id=SourceId(path.to_string_lossy().into_owned());let text=read_optional(path)?;self.observed.borrow_mut().entry(id).or_insert(text.clone());Ok(text)
   })?;
   tsrust_compiler::project_resolution::candidates(Path::new(&importer.0),path,aliases.as_ref())?
  }else{vec![PathBuf::from(path)]};
  for path in candidates {let path=normalize(&path)?;let id=SourceId(path.to_string_lossy().into_owned());if self.overlay.contains_key(&id) || path.is_file(){return Ok(id);}}
  Err(format!("Cannot resolve {path}"))
 }
}
impl SourceProvider for Candidate {
 fn resolve(&self,importer:Option<&SourceId>,path:&str)->Result<SourceId,String>{
  let id=self.resolve_current(importer,path)?;self.resolutions.borrow_mut().push((importer.cloned(),path.into(),id.clone()));Ok(id)
 }
 fn read(&self,id:&SourceId)->Result<String,String>{
  if let Some(text)=self.overlay.get(id){return Ok(text.clone());}
  if id.0.starts_with("std:"){return FileSourceProvider.read(id);}
  if let Some(value)=self.observed.borrow().get(id){return value.clone().ok_or_else(||format!("Missing {}",id.0));}
  let text=read_optional(Path::new(&id.0))?;self.observed.borrow_mut().insert(id.clone(),text.clone());text.ok_or_else(||format!("Missing {}",id.0))
 }
}
pub(crate) fn read_optional(path:&Path)->Result<Option<String>,String>{match fs::read_to_string(path){Ok(text)=>Ok(Some(text)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(format!("{}: {error}",path.display()))}}
pub(crate) fn normalize(path:&Path)->Result<PathBuf,String>{
 if path.exists(){return fs::canonicalize(path).map_err(|error|error.to_string());}
 // Virtual generated imports can traverse parents which do not exist yet.
 // Keep existing-path canonicalization (including symlink semantics) unchanged.
 let mut lexical=PathBuf::new();
 for part in path.components(){match part{std::path::Component::CurDir=>{},std::path::Component::ParentDir=>{if !lexical.pop(){return Err("Path escapes its filesystem root".into());}},part=>lexical.push(part.as_os_str())}}
 let mut ancestor=lexical.as_path();let mut missing=Vec::new();
 while !ancestor.exists(){missing.push(ancestor.file_name().ok_or("Path requires an existing ancestor")?.to_owned());ancestor=ancestor.parent().ok_or("Path requires an existing ancestor")?;}
 let mut normalized=fs::canonicalize(ancestor).map_err(|error|error.to_string())?;
 for name in missing.into_iter().rev(){normalized.push(name);}
 Ok(normalized)
}
pub(crate) fn validate(provider:&Candidate,root:&Path,manifest:&Path,temporary:&Path)->Result<(usize,Vec<String>),String>{
 let backend=crate::verify_backend()?;
 let files=tsrust_compiler::source_discovery::discover(&[root.to_owned()])?;
 let mut session=tsrust_compiler::project_session::ProjectSession::default();
 // Compile even an unused adapter: source checks alone do not check Rust bodies.
 let graph=tsrust_compiler::project::load_project(provider,manifest.to_str().ok_or("Non-UTF8 manifest")?).map_err(|error|format!("{}: {}",error.code,error.message))?;
 let exports=graph.modules[&graph.entry].exports.keys().cloned().collect::<Vec<_>>();
 let adapter=tsrust_compiler::project::check_graph(graph).map_err(|error|format!("{}: {}",error.code,error.message))?;
 compile(&adapter.emit_rust(),&backend,&temporary.join("adapter"),None)?;
 for (index,path) in files.iter().enumerate() {
  let source=provider.read(&SourceId(path.to_string_lossy().into_owned()))?;
  let analysis=tsrust_compiler::project_cli::check_input_cached_with_provider(path,&source,&mut session,provider).map_err(|error|error.render(&path.to_string_lossy(),&source,false))?;
  compile(&analysis.emit_rust(),&backend,&temporary.join(format!("caller-{index}")),None)?;
 }
 let tests=tsrust_compiler::project_cli::discover_test_inputs_optional(&[root.to_owned()])?;
 if !tests.is_empty(){
  let analysis=tsrust_compiler::project_cli::check_test_inputs_with_provider(&tests,provider).map_err(|error|error.render("tests","",false))?;
  compile(&analysis.emit_rust(),&backend,&temporary.join("tests"),Some(root))?;
 }
 if tsrust_compiler::source_discovery::discover(&[root.to_owned()])?!=files{return Err("Project files changed during validation; nothing applied.".into());}
 provider.unchanged()?;Ok((tests.len(),exports))
}
fn compile(source:&str,backend:&str,root:&Path,execute:Option<&Path>)->Result<(),String>{
 fs::create_dir(root).map_err(|error|error.to_string())?;
 let mut prepared=crate::runtime_build::Prepared::new(source,backend,&root.parent().unwrap().join("runtime-cache"),root,false)?;
 if let Some(mut command)=prepared.command.take(){let output=command.output().map_err(|error|error.to_string())?;if !output.status.success(){return Err(format!("Candidate Rust dependency build failed:\n{}",String::from_utf8_lossy(&output.stderr)));}prepared.publish();}
 let executable=root.join("program");let mut command=crate::backend_command();prepared.link(&mut command)?;
 let output=command.args(["--edition=2024","-C","overflow-checks=on"]).arg(&prepared.application).arg("-o").arg(&executable).output().map_err(|error|error.to_string())?;
 if !output.status.success(){return Err(format!("Candidate adapter/caller compilation failed:\n{}",String::from_utf8_lossy(&output.stderr)));}
 if let Some(project)=execute{let output=std::process::Command::new(&executable).current_dir(project).env("TSRUST_TEST_REPORTER","pretty").env("TSRUST_COLOR","never").output().map_err(|error|error.to_string())?;print!("{}",String::from_utf8_lossy(&output.stdout));if !output.status.success(){return Err(format!("Candidate project tests failed:\n{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr)));}}
 Ok(())
}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn candidate_reads_are_frozen_and_concurrent_edits_abort(){
  let dir=crate::TempDirectory::new().unwrap();let source=dir.0.join("main.tsr");fs::write(&source,"function main(){}").unwrap();let source=fs::canonicalize(source).unwrap();let id=SourceId(source.to_string_lossy().into_owned());let candidate=Candidate::new(BTreeMap::new()).unwrap();let original=candidate.read(&id).unwrap();fs::write(&source,"function main(){console.log('edited');}").unwrap();assert_eq!(candidate.read(&id).unwrap(),original);assert!(candidate.unchanged().is_err());
 }
}
