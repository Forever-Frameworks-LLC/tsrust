//! Explicit source-fallback generation into an existing exact-pinned binding.
use std::{collections::BTreeMap,fs,path::Path};
use sha2::{Digest,Sha256};
use serde::{Deserialize,Serialize};
use tsrust_compiler::project::rust::{dependencies::Bindings,generation};
fn hash(value:&str)->String{Sha256::digest(value.as_bytes()).iter().map(|byte|format!("{byte:02x}")).collect()}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Report{discovery:String,dependency:String,version:String,source_sha256:String,#[serde(default,skip_serializing_if="Option::is_none")]rustdoc_sha256:Option<String>,selected:Vec<String>,skipped:Vec<serde_json::Value>,files:BTreeMap<String,String>}
pub(super) fn run(mut args:impl Iterator<Item=String>)->Result<u8,String>{
 let binding_path=args.next().ok_or("Expected file.bindings.json.")?;if !binding_path.ends_with(".bindings.json"){return Err("Expected file.bindings.json.".into());}
 let(mut dependency,mut source,mut selected,mut rustdoc)=(None,None,None,None);
 while let Some(flag)=args.next(){let slot=match flag.as_str(){"--crate"=>&mut dependency,"--source"=>&mut source,"--select"=>&mut selected,"--rustdoc"=>&mut rustdoc,_=>return Err("Usage: tsrust bindings generate file.bindings.json --crate dependency --source crate-root.rs --select name,name [--rustdoc crate.json]".into())};if slot.is_some(){return Err(format!("Repeated {flag}."));}*slot=Some(args.next().ok_or_else(||format!("Missing value for {flag}."))?);}
 let dependency=dependency.ok_or("Missing --crate.")?;let source=source.ok_or("Missing --source.")?;let selected=selected.ok_or("Missing --select.")?.split(',').map(str::to_owned).collect::<Vec<_>>();
 if selected.iter().any(|name|syn::parse_str::<syn::Ident>(name).is_err()||name.starts_with("r#")){return Err("Selected names must be ordinary Rust identifiers.".into());}
 let binding_path=fs::canonicalize(binding_path).map_err(|error|error.to_string())?;let _lock=crate::package_cli::project_lock(&binding_path)?;
 let binding_text=fs::read_to_string(&binding_path).map_err(|error|error.to_string())?;let binding=Bindings::parse(&binding_text)?;
 let pin=binding.dependencies.get(&dependency).ok_or("--crate must name an exact-pinned binding dependency.")?;
 let source=fs::read_to_string(source).map_err(|error|error.to_string())?;
 let rustdoc=rustdoc.map(fs::read_to_string).transpose().map_err(|error|error.to_string())?;
 if let Some(json)=&rustdoc{let metadata:serde_json::Value=serde_json::from_str(json).map_err(|error|error.to_string())?;if metadata.get("crate_version").and_then(serde_json::Value::as_str)!=Some(pin.version.trim_start_matches('=')){return Err("Rustdoc crate version does not match the exact binding dependency pin.".into());}}
 let generated=if let Some(json)=&rustdoc{generation::rustdoc::generate(&dependency.replace('-',"_"),json,&source,&selected)?}else{generation::source(&dependency.replace('-',"_"),&source,&selected)?};
 let root=binding_path.parent().unwrap();let adapter=root.join(&binding.source);
 if Path::new(&binding.source).components().any(|part|!matches!(part,std::path::Component::Normal(_)|std::path::Component::CurDir)){return Err("Generated adapter source must stay under the binding directory.".into());}
 let declarations=adapter.with_extension("api.tsr");let stubs=adapter.with_extension("stubs.rs");let report_path=binding_path.with_extension("generation.json");
 // Reject symlinks at every component before reading or replacing generated files.
 for path in [&adapter,&declarations,&stubs,&report_path]{let mut parent=root.to_path_buf();for component in path.strip_prefix(root).map_err(|error|error.to_string())?.components(){parent.push(component);match fs::symlink_metadata(&parent){Ok(metadata)if metadata.file_type().is_symlink()=>return Err(format!("Generated path is a symlink: {}",parent.display())),Ok(_)=>{},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(error.to_string())}}}
 let previous=crate::package_validation::read_optional(&report_path)?;let previous=previous.as_deref().map(serde_json::from_str::<Report>).transpose().map_err(|error|format!("Invalid generation report: {error}"))?;
 let key=|path:&Path|path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
 let mut writes=vec![(adapter,generated.adapter),(declarations,generated.declarations)];
 // Stubs are deliberately not compiled and remain user-owned after creation.
 if !stubs.exists(){writes.push((stubs.clone(),generated.stubs));}
 let mut originals=Vec::new();let mut files=BTreeMap::new();
 for(path,contents)in &writes{let existing=crate::package_validation::read_optional(path)?;if let Some(existing)=&existing{if existing!=contents&&previous.as_ref().and_then(|report|report.files.get(&key(path))).is_none_or(|expected|expected!=&hash(existing)){return Err(format!("Customized adapter preserved: {}. Edit it explicitly or choose a new binding source.",path.display()));}}files.insert(key(path),hash(contents));originals.push((path.clone(),existing));}
 if let Some(contents)=crate::package_validation::read_optional(&stubs)?{files.insert(key(&stubs),hash(&contents));originals.push((stubs,Some(contents)));}
 let report=Report{discovery:if rustdoc.is_some(){"rustdoc-json-61"}else{"source-fallback"}.into(),dependency,version:pin.version.clone(),source_sha256:hash(&source),rustdoc_sha256:rustdoc.as_deref().map(hash),selected:generated.selected,skipped:generated.skipped.iter().map(serde_json::to_value).collect::<Result<_,_>>().map_err(|error|error.to_string())?,files};
 writes.push((report_path.clone(),format!("{}\n",serde_json::to_string_pretty(&report).map_err(|error|error.to_string())?)));originals.push((report_path,crate::package_validation::read_optional(&binding_path.with_extension("generation.json"))?));
 if fs::read_to_string(&binding_path).map_err(|error|error.to_string())?!=binding_text{return Err("Binding changed during generation; nothing applied.".into());}
 for(path,original)in &originals{if crate::package_validation::read_optional(path)?!=*original{return Err("Generated file changed during generation; nothing applied.".into());}}
 for(path,contents)in &writes{if let Err(error)=crate::package_cli::publish(path,contents){let mut rollback=Vec::new();for(path,original)in &originals{let result=if let Some(original)=original{crate::package_cli::publish(path,original)}else{match fs::remove_file(path){Ok(())=>Ok(()),Err(error)if error.kind()==std::io::ErrorKind::NotFound=>Ok(()),Err(error)=>Err(error.to_string())}};if let Err(error)=result{rollback.push(error);}}return Err(format!("{error}; rollback errors: {rollback:?}"));}}
 println!("Generated {} concrete exports; {} skipped. No dependencies resolved or downloaded.",report.selected.len(),report.skipped.len());Ok(0)
}
