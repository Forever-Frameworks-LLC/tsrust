//! Agent references are installed in the same transaction as their components.
use super::*;
use tsrust_compiler::project::SourceId;

const INTRO:&str="# Tsrust UI component guidance\n\nBefore using or editing a component, read its linked reference below and its local\nsource. References describe the original installed API; customized local source\nand owner instructions take precedence. Import paths in references are relative\nto this directory: adjust them for the calling module.\n\nUse JSX in .tsrx files, supply required props, and bind application logic in typed\nhandlers. Check inherited primitive props before adding a wrapper property.\nPreserve source notices and ui-components.lock.json. Validate complete changes\nwith tsrust check, tsrust test and tsrust format --check.\n";

fn id(root:&Path,path:&Path)->SourceId {
 let mut normalized=PathBuf::new();for component in root.join(path).components(){normalized.push(component.as_os_str());}
 SourceId(normalized.to_string_lossy().into_owned())
}
fn module_path(directory:&Path,path:&Path)->Result<String,String>{
 let mut module=tsrust_compiler::native_path_relative::relative_host(directory.to_str().ok_or("Invalid component directory")?,path.with_extension("").to_str().ok_or("Invalid component path")?)?.replace('\\',"/");
 if !module.starts_with('.') {module=format!("./{module}");}Ok(module)
}
fn reference(root:&Path,directory:&Path,item:&Item,paths:&BTreeMap<(Identity,String),PathBuf>,sources:&BTreeMap<SourceId,String>,session:&mut tsrust_compiler::project_session::ProjectSession)->Result<String,String>{
 let identity=identity(item);
 let mut text=format!("# {}@{} ({})\n\nThis reference describes the original installed component. Read the current local\nsource before changing or relying on customized behavior. Owner edits are preserved.\n\n",item.name,item.version,item.variant);
 for file in &item.files {
  let path=&paths[&(identity.clone(),file.path.clone())];
  text.push_str(&format!("Source: `{}`.\n\n",path.to_string_lossy().replace('\\',"/")));
  if file.destination=="theme" {
   text.push_str("Edit the configured ui.theme tokens or external theme file. Keep paired light/dark values and validate with tsrust check.\n\n");continue;
  }
  let source_id=id(root,path);if !sources.contains_key(&source_id){return Err("Missing component guidance source".into());}
  let provider=crate::docs_cli::PublicationInputs{entry:source_id.clone(),files:sources.clone()};
  let analysis=session.check(&provider,&source_id.0).map_err(|error|format!("Cannot document {}: {}",item.name,error.message))?;
  let api=tsrust_compiler::documentation::api(&analysis,&[source_id.clone()])?;
  let exports=api.iter().map(|entry|entry.name.as_str()).collect::<Vec<_>>();
  if !exports.is_empty(){text.push_str(&format!("From this directory (adjust for the caller):\n\n```tsrust\nimport {{ {} }} from '{}';\n```\n\n",exports.join(", "),module_path(directory,path)?));}
  for entry in api {
   text.push_str(&format!("## {}\n\n",entry.name));
   if !entry.documentation.is_empty(){text.push_str(&entry.documentation);text.push_str("\n\n");}
   text.push_str(&format!("```tsrust\n{}\n```\n\n",entry.signature));
   for field in entry.fields {
    text.push_str(&format!("- `{}`{}: `{}`",field.name,if field.optional{" (optional)"}else{" (required)"},field.signature));
    if !field.documentation.is_empty(){text.push_str(". ");text.push_str(&field.documentation.replace('\n'," "));}text.push('\n');
   }
   text.push('\n');
  }
 }
 if !item.dependencies.is_empty(){text.push_str("## Installed dependencies\n\n");for dependency in &item.dependencies{text.push_str(&format!("- {}@{} ({}); use its local reference and recorded import paths.\n",dependency.name,dependency.version,dependency.variant));}text.push('\n');}
 text.push_str("## Editing and verification\n\nUse callable components through JSX with their declared props; call ordinary helpers\nas functions. Omit optional props to use the implementation's defaults. Read local\nsource and primitive documentation for constraints not stated above. Ownership and\ncallback captures follow compiler checks; do not insert implicit clones. Check the\nwhole application, run affected assertions and format the project before finishing.\nExercise the configured native/web target when observable UI behavior changes.\n");
 Ok(text)
}

pub(super) fn plan(root:&Path,config:&Config,directory:&Path,items:&[Item],paths:&BTreeMap<(Identity,String),PathBuf>,plan:&mut Plan)->Result<(),String>{
 let canonical=root.canonicalize().map_err(|e|e.to_string())?;
 let mut sources=BTreeMap::new();
 // Reconstruct immutable catalog inputs at their installed/versioned locations.
 // Existing customized implementations remain untouched and authoritative.
 for item in items {for file in &item.files {if file.path.ends_with(".tsrx") {
  let path=&paths[&(identity(item),file.path.clone())];
  sources.insert(id(&canonical,path),installed_source(item,file,path,items,paths)?);
 }}}
 // Setup may be filling missing configuration/theme slots in this transaction.
 // Analyze that effective configuration without publishing it early.
 for (path,bytes) in &plan.files {if path.extension().is_none_or(|value|value!="tsrx"){
  let source=String::from_utf8(bytes.clone()).map_err(|_|"Invalid generated guidance input")?;
  sources.insert(id(&canonical,path),source);
 }}
 let configuration=Path::new(crate::project_config::FILE);
 let mut effective=read_optional(&no_symlinks(root,configuration)?)?.map(|bytes|serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e|e.to_string())).transpose()?.unwrap_or_else(||serde_json::json!({}));
 effective["ui"]=serde_json::to_value(config).map_err(|e|e.to_string())?;
 sources.insert(id(&canonical,configuration),serde_json::to_string(&effective).map_err(|e|e.to_string())?);
 let root_guide=PathBuf::from("AGENTS.md");
 let existing_root=no_symlinks(root,&root_guide)?;
 if existing_root.exists()&&!existing_root.is_file(){return Err("AGENTS.md must be a file; existing directory preserved".into());}
 if !existing_root.exists(){plan.files.push((root_guide,crate::new_cli::AGENT_GUIDANCE.as_bytes().to_vec()));plan.messages.push("create AGENTS.md (project agent guidance)".into());}
 let index_path=portable_generated_path(&directory.join("AGENTS.md"))?;
 let original=read_optional(&no_symlinks(root,&index_path)?)?;
 let mut index=original.clone().unwrap_or_else(||INTRO.as_bytes().to_vec());
 let index_text=std::str::from_utf8(&index).map_err(|_|format!("{} must be UTF-8; existing contents preserved",index_path.display()))?;
 let mut additions=String::new();
 let mut session=tsrust_compiler::project_session::ProjectSession::default();
 for item in items {
  if !crate::project_config::valid_name(&item.name)||!crate::project_config::valid_name(&item.variant){return Err("Invalid component guidance name".into());}
  version_number(&item.version)?;
  let marker=format!("<!-- tsrust-ui-agent:{}@{}:{} -->",item.name,item.version,item.variant);
  let indexed=index_text.contains(&marker);
  let name=format!("{}-{}-{}.AGENTS.md",item.name,item.version,item.variant);
  let path=portable_generated_path(&directory.join(&name))?;
  let existing=no_symlinks(root,&path)?;
  if existing.exists()&&!existing.is_file(){return Err(format!("{} must be a file; existing directory preserved",path.display()));}
  if indexed&&existing.exists(){continue;}
  if !existing.exists(){
   let text=reference(&canonical,directory,item,paths,&sources,&mut session)?;
   plan.files.push((path.clone(),text.into_bytes()));plan.messages.push(format!("create {} (component agent reference)",path.display()));
  }
  if !indexed{additions.push_str(&format!("\n{marker}\n- [{}@{} ({})]({name})\n",item.name,item.version,item.variant));}
 }
 if !additions.is_empty(){if !index.ends_with(b"\n"){index.push(b'\n');}index.extend_from_slice(additions.as_bytes());}
 if original.as_deref()!=Some(index.as_slice()) {
  plan.messages.push(format!("{} {} (component agent index)",if original.is_some(){"append to"}else{"create"},index_path.display()));
  if original.is_some(){plan.updates.push(Update{path:index_path,previous:original,bytes:index});}else{plan.files.push((index_path,index));}
 }
 Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn every_current_component_has_checked_guidance_without_overwriting_owner_files(){
  let temporary=crate::TempDirectory::new().unwrap();let root=&temporary.0;let config=Config::default();
  fs::write(root.join("tsrust.config.json"),serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"name":"guide-test","build":{"entry":"src/main.tsr","output":"target/app"},"ui":&config})).unwrap()).unwrap();
  let owner=b"# Owner instructions\r\nPreserve this exact text.\r\n";fs::write(root.join("AGENTS.md"),owner).unwrap();
  fs::create_dir_all(root.join(&config.components)).unwrap();fs::write(root.join(&config.components).join("AGENTS.md"),owner).unwrap();
  let items=ordered_catalog().unwrap();let requests=items.iter().map(identity).collect::<Vec<_>>();
  let installation=super::super::plan(root,&config,&requests,None,"test").unwrap();
  assert!(!root.join(&config.lock).exists(),"planning must not install anything");
  for item in &items {
   let name=format!("{}-{}-{}.AGENTS.md",item.name,item.version,item.variant);
   let (_,bytes)=installation.files.iter().find(|(path,_)|path.file_name().is_some_and(|value|value==name.as_str())).expect("every component needs a reference");
   let text=std::str::from_utf8(bytes).unwrap();assert!(text.contains(&format!("{}@{}",item.name,item.version)));
   if item.name=="button" {for witness in ["function Button", "variant", "secondary", "onPress"]{assert!(text.contains(witness),"missing {witness}");}assert!(!text.contains("const variant ="),"references must not snapshot component bodies");}
  }
  apply(root,&installation).unwrap();assert_eq!(fs::read(root.join("AGENTS.md")).unwrap(),owner);
  let index=fs::read(root.join(&config.components).join("AGENTS.md")).unwrap();assert!(index.starts_with(owner));
  let guide=root.join(&config.components).join("button-3.0.0-default.AGENTS.md");fs::write(&guide,"Owner-customized component advice\n").unwrap();
  let repeated=super::super::plan(root,&config,&requests,None,"test").unwrap();assert!(repeated.files.is_empty()&&repeated.updates.is_empty());
  apply(root,&repeated).unwrap();assert_eq!(fs::read(&guide).unwrap(),b"Owner-customized component advice\n");
  assert_eq!(fs::read(root.join(&config.components).join("AGENTS.md")).unwrap(),index);
 }
}
