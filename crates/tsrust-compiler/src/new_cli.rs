use std::{fs,path::PathBuf};
use super::project_config::{FILE,valid_name};

pub fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
    let args=args.collect::<Vec<_>>();
    if args.len()!=1 || args[0].starts_with('-') {return Err("Usage: tsrust new <name-or-path>".into());}
    let root=PathBuf::from(&args[0]);
    let name=root.file_name().and_then(|value|value.to_str()).filter(|name|valid_name(name))
        .ok_or("Project name must start with a letter and contain only ASCII letters, digits, '_' or '-' (maximum 100 characters)")?;
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
    fs::write(root.join("README.md"),format!("# {name}\n\nRun `tsrust dev` to rebuild and restart on source changes.\n\n- `tsrust run`: run once.\n- `tsrust build`: build the configured binary.\n- `tsrust check`: check project sources.\n- `tsrust test`: run project tests.\n- `tsrust format`: format project sources.\n\nThe nearest `{FILE}` defines the project root, entry and output.\nA `main` entrypoint is optional; this template includes one to print a greeting.\n")).map_err(|e|e.to_string())?;
    println!("Created {}.\nRun `tsrust dev` from that directory.",root.display());
    Ok(0)
}
