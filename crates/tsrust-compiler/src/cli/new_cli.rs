use std::{fs,path::PathBuf};
use super::project_config::{FILE,valid_name};
pub(super) const AGENT_GUIDANCE:&str=include_str!("project-agents.md");

pub fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
    let mut args=args;let mut application=false;let mut requested=None;let mut targets=vec![];
    while let Some(arg)=args.next(){match arg.as_str(){"--app" if !application=>application=true,"--ui"=>return Err(format!("new --ui: {}",super::ui_cli::UNAVAILABLE)),"--target"=>targets.push(args.next().ok_or("--target requires a target name")?),_ if !arg.starts_with('-') && requested.is_none()=>requested=Some(arg),_=>return Err("Usage: tsrust new <name-or-path> [--app [--target backend]]".into())}}
    if !application && !targets.is_empty(){return Err("--target requires --app".into());}
    if application {super::application_scaffold::validate_targets(&targets)?;}
    let guided=requested.is_none() && super::interactive_cli::available();
    let root=loop {
        let requested=if guided {let Some(value)=super::interactive_cli::text("Project name or path")? else{return Ok(0);};value}else{requested.take().ok_or("Missing project name. Run tsrust new in a terminal, or pass a name.")?};
        match destination(&requested) {Ok(root)=>break root,Err(error) if guided=>eprintln!("{error}"),Err(error)=>return Err(error)}
    };
    if guided && !application {
        let Some(kind)=super::interactive_cli::choose("Project type",&["Backend HTTP application","Console application"])? else{return Ok(0);};application=kind==0;
    }
    if application {return super::application_scaffold::create(&root,&targets);}
    let name=root.file_name().and_then(|value|value.to_str()).unwrap();
    // Exclusive creation protects existing files, directories and symlinks.
    fs::create_dir(&root).map_err(|e|format!("Cannot create {}: {e}. Existing projects are never overwritten.",root.display()))?;
    let config=serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":"src/main.tsr","output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)}});
    fs::create_dir(root.join("src")).map_err(|e|e.to_string())?;
    fs::create_dir(root.join("tests")).map_err(|e|e.to_string())?;
    fs::write(root.join(FILE),format!("{}\n",serde_json::to_string_pretty(&config).unwrap())).map_err(|e|e.to_string())?;
    let greeting=format!("export function greeting():string{{return \"Hello from {name}!\";}}");
    let test=format!("import {{greeting}} from '../src/greeting.tsr';import {{test,strictEqual}} from 'std:testing';test('greets by project name',():void=>{{strictEqual(greeting(),\"Hello from {name}!\");}});");
    for (path,source) in [("src/main.tsr","import {greeting} from './greeting.tsr';function main(){console.log(greeting());}"),("src/greeting.tsr",greeting.as_str()),("tests/greeting.test.tsr",test.as_str())] {
        let formatted=tsrust_compiler::format(source).map_err(|error|format!("Cannot format scaffold: {}",error.message))?;
        fs::write(root.join(path),formatted).map_err(|e|e.to_string())?;
    }
    fs::write(root.join(".gitignore"),"/target/\n").map_err(|e|e.to_string())?;
    fs::write(root.join("AGENTS.md"),AGENT_GUIDANCE).map_err(|e|e.to_string())?;
    fs::write(root.join("README.md"),format!("# {name}\n\nRun `tsrust dev` to rebuild and restart on source changes.\n\nRead `AGENTS.md` for project workflow and agent instructions.\n\n- `tsrust run`: run once.\n- `tsrust build`: build the configured binary.\n- `tsrust check`: check project sources.\n- `tsrust test`: run project tests.\n- `tsrust format`: format project sources.\n\nThe nearest `{FILE}` defines the project root, entry and output.\nA `main` entrypoint is optional; this template includes one to print a greeting.\n")).map_err(|e|e.to_string())?;
    println!("Created {}.\nRun `tsrust dev` from that directory.",root.display());
    Ok(0)
}

fn destination(requested:&str)->Result<PathBuf,String> {
    let root=PathBuf::from(requested);
    root.file_name().and_then(|value|value.to_str()).filter(|name|valid_name(name))
        .ok_or("Project name must start with a letter and contain only ASCII letters, digits, '_' or '-' (maximum 100 characters)")?;
    if fs::symlink_metadata(&root).is_ok(){return Err(format!("Cannot create {}. Existing projects are never overwritten.",root.display()));}
    let parent=root.parent().filter(|path|!path.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    if !parent.is_dir(){return Err(format!("Parent directory does not exist: {}",parent.display()));}
    Ok(root)
}
