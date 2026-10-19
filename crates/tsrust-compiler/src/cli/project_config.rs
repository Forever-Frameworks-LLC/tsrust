//! CLI project roots and explicit supported configuration. Never execute config.
use std::{env,fs,path::{Path,PathBuf,Component}};
use serde_json::Value;

pub const FILE:&str="tsrust.config.json";
pub struct Project {pub root:PathBuf,pub entry:PathBuf,pub output:PathBuf,pub ui:Option<tsrust_compiler::ui_project::Config>}
fn object<'a>(value:&'a Value,at:&str,keys:&[&str])->Result<&'a serde_json::Map<String,Value>,String> {
    let object=value.as_object().ok_or_else(||format!("{at} must be an object"))?;
    for key in object.keys() {if !keys.contains(&key.as_str()) {return Err(format!("Unknown {at} setting: {key}"));}}
    Ok(object)
}
pub fn valid_name(name:&str)->bool {
    !name.is_empty() && name.len()<=100 && name.as_bytes()[0].is_ascii_alphabetic()
        && name.bytes().all(|byte|byte.is_ascii_alphanumeric() || matches!(byte,b'-'|b'_'))
}
fn relative(value:Option<&Value>,at:&str)->Result<PathBuf,String> {
    let text=value.and_then(Value::as_str).ok_or_else(||format!("{at} must be a relative path string"))?;
    let path=PathBuf::from(text);
    if text.is_empty() || path.components().any(|part|!matches!(part,Component::Normal(_))) {
        return Err(format!("{at} must be a nonempty relative path without '.' or '..' components"));
    }
    Ok(path)
}
pub fn load(root:&Path)->Result<Project,String> {
    let path=root.join(FILE);
    let text=fs::read_to_string(&path).map_err(|e|format!("{}: {e}",path.display()))?;
    parse(root,&text)
}
pub(crate) fn parse(root:&Path,text:&str)->Result<Project,String> {
    let path=root.join(FILE);
    let value:Value=serde_json::from_str(text).map_err(|e|format!("{}: {e}",path.display()))?;
    let config=object(&value,FILE,&["schemaVersion","name","build","paths","ui"])?;
    if config.get("schemaVersion").and_then(Value::as_u64)!=Some(1) {return Err(format!("{FILE}: schemaVersion must be 1"));}
    if !config.get("name").and_then(Value::as_str).is_some_and(valid_name) {return Err(format!("{FILE}: name must start with a letter and contain only ASCII letters, digits, '_' or '-' (maximum 100 characters)"));}
    tsrust_compiler::project_resolution::Aliases::parse(root,config.get("paths"))?;
    let ui=config.get("ui").map(tsrust_compiler::ui_project::Config::parse).transpose()?;
    let build=object(config.get("build").ok_or("Missing build configuration")?,"build",&["entry","output"])?;
    let entry=relative(build.get("entry"),"build.entry")?;
    let output=relative(build.get("output"),"build.output")?;
    if entry.extension().is_none_or(|extension|extension!="tsr" && extension!="tsrx") {return Err("build.entry must name a .tsr or .tsrx file".into());}
    if output==entry || output.extension().is_some_and(|extension|extension=="tsr" || extension=="tsrx") || output==Path::new(FILE) {
        return Err("build.output must not name a source or configuration file".into());
    }
    Ok(Project {ui,root:root.to_owned(),entry:root.join(entry),output:root.join(output)})
}
pub fn discover()->Result<Option<Project>,String> {
    let cwd=env::current_dir().map_err(|e|e.to_string())?;
    for root in cwd.ancestors() {
        match fs::symlink_metadata(root.join(FILE)) {
            Ok(_)=>return load(root).map(Some),
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error.to_string()),
        }
    }
    Ok(None)
}
pub fn default_root()->Result<PathBuf,String> {
    Ok(discover()?.map(|project|project.root).unwrap_or(env::current_dir().map_err(|e|e.to_string())?))
}
/// Expand only omitted inputs. Explicit paths continue to be relative to cwd.
pub fn defaults(command:&str,mut args:Vec<String>)->Result<(Vec<String>,Option<PathBuf>),String> {
    if !matches!(command,"run"|"build"|"emit") || args.first().is_some_and(|arg|!arg.starts_with('-')) {return Ok((args,None));}
    if let Some(project)=discover()? {
        args.insert(0,project.entry.to_string_lossy().into_owned());
        if command=="build" && !args.iter().any(|arg|matches!(arg.as_str(),"-o"|"--output")) {
            if let Some(parent)=project.output.parent() {fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
            args.extend(["--output".into(),project.output.to_string_lossy().into_owned()]);
        }
        return Ok((args,Some(project.root)));
    }
    Ok((args,None))
}
