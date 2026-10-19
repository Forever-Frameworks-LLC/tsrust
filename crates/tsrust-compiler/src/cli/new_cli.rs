use std::{fs,path::PathBuf};
use super::project_config::{FILE,valid_name};

pub fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
    let mut args=args;let mut ui=false;let mut requested=None;let mut targets=vec![];
    while let Some(arg)=args.next(){match arg.as_str(){"--ui" if !ui=>ui=true,"--target"=>targets.push(args.next().ok_or("--target requires native or web")?),_ if !arg.starts_with('-') && requested.is_none()=>requested=Some(arg),_=>return Err("Usage: tsrust new <name-or-path> [--ui [--target native|web]...]".into())}}
    if !ui && !targets.is_empty(){return Err("--target requires --ui".into());}
    let guided=requested.is_none() && super::interactive_cli::available();
    let root=loop {
        let requested=if guided {let Some(value)=super::interactive_cli::text("Project name or path")? else{return Ok(0);};value}else{requested.take().ok_or("Missing project name. Run tsrust new in a terminal, or pass a name.")?};
        match destination(&requested) {Ok(root)=>break root,Err(error) if guided=>eprintln!("{error}"),Err(error)=>return Err(error)}
    };
    if guided && !ui {
        let Some(kind)=super::interactive_cli::choose("Project type",&["Console application","UI application"])? else{return Ok(0);};ui=kind==1;
    }
    let name=root.file_name().and_then(|value|value.to_str()).unwrap();
    let target=if ui{Some(super::ui_cli::select_targets(None,&targets,false)?)}else{None};
    if ui {super::verify_backend()?;}
    // Exclusive creation protects existing files, directories and symlinks.
    fs::create_dir(&root).map_err(|e|format!("Cannot create {}: {e}. Existing projects are never overwritten.",root.display()))?;
    let mut config=serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":"src/main.tsr","output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)}});
    let mut ui_config=tsrust_compiler::ui_project::Config::default();ui_config.target=target.clone();
    if ui {config["ui"]=serde_json::to_value(&ui_config).unwrap();}
    fs::create_dir(root.join("src")).map_err(|e|e.to_string())?;
    fs::create_dir(root.join("tests")).map_err(|e|e.to_string())?;
    fs::write(root.join(FILE),format!("{}\n",serde_json::to_string_pretty(&config).unwrap())).map_err(|e|e.to_string())?;
    let greeting=format!("export function greeting():string{{return \"Hello from {name}!\";}}");
    let test=format!("import {{greeting}} from '../src/greeting.tsr';import {{test,strictEqual}} from 'std:testing';test('greets by project name',():void=>{{strictEqual(greeting(),\"Hello from {name}!\");}});");
    let sources=if ui {
        fs::create_dir(root.join("src/app")).map_err(|e|e.to_string())?;
        vec![("src/main.tsr", UI_MAIN), ("src/app/page.tsrx", UI_PAGE), ("tests/page.test.tsr", UI_TEST)]
    } else { vec![("src/main.tsr","import {greeting} from './greeting.tsr';function main(){console.log(greeting());}"),("src/greeting.tsr",greeting.as_str()),("tests/greeting.test.tsr",test.as_str())] };
    for (path,source) in sources {
        let formatted=tsrust_compiler::format(source).map_err(|error|format!("Cannot format scaffold: {}",error.message))?;
        fs::write(root.join(path),formatted).map_err(|e|e.to_string())?;
    }
    fs::write(root.join(".gitignore"),"/target/\n").map_err(|e|e.to_string())?;
    fs::write(root.join("README.md"),format!("# {name}\n\nRun `tsrust dev` to rebuild and restart on source changes.\n\n- `tsrust run`: run once.\n- `tsrust build`: build the configured binary.\n- `tsrust check`: check project sources.\n- `tsrust test`: run project tests.\n- `tsrust format`: format project sources.\n\nThe nearest `{FILE}` defines the project root, entry and output.\nA `main` entrypoint is optional; this template includes one to print a greeting.\n")).map_err(|e|e.to_string())?;
    if ui {
        super::ui_cli::install_defaults(&root,&ui_config)?;
        fs::write(root.join("README.md"), format!("# {name}\n\nRun `tsrust dev` to open the native UI and restart after edits.\n\n`tsrust.config.json` declares `src/main.tsr` as the executable entry. That visible bootstrap mounts `Page` from `src/app/page.tsrx`, the root screen convention. Put application state and handlers in components, or import them from ordinary source modules.\n\n`tsrust check`, `tsrust test` and `tsrust format --check` validate this project. `tsrust build` writes the configured executable.\n\nEdit `ui.theme` in `tsrust.config.json` for paired light/dark tokens and spacing, radius and text sizes. Select mode in `src/main.tsr`. Edit `src/components/ui/button.tsrx` and `card.tsrx` directly. The UI paths live in the `ui` field of `tsrust.config.json`. Commit the source, theme and `ui-components.lock.json` together.\n\n`tsrust ui list --bundled` lists offline payloads. `tsrust ui add button@1.0.0 --dry-run` previews the exact installation. Re-adding preserves customized files and lock bytes. `--path` selects an alternate component directory; occupied upgrades are refused. Installation never downloads dependencies.\n\nThis scaffold implements one root page. Nested route discovery, layouts, loaders, navigation, input editing and async UI remain unfinished; naming another file does not wire it into this explicit bootstrap.\n")).map_err(|e|e.to_string())?;
    }
    if target.as_ref().is_some_and(|t|!t.native){
        let readme=root.join("README.md");let text=fs::read_to_string(&readme).map_err(|e|e.to_string())?.replace("Run `tsrust dev` to open the native UI and restart after edits.","This project selects the web target. Use the bounded Wasm web builder; native dev/run is disabled. A general web dev server is not implemented yet.");let text=text.replace("`tsrust build` writes the configured executable.","Use the bounded Wasm builder for browser output; `tsrust emit --package` exports web support without enabling native GPU dependencies.");fs::write(readme,text).map_err(|e|e.to_string())?;
        println!("Created {}. Web UI selected; use the bounded Wasm web builder. Native dev/run is disabled.",root.display());
    }else{println!("Created {}.\nRun `tsrust dev` from that directory.",root.display());}
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

pub(super) const UI_MAIN: &str = "import { run, withTheme } from 'std:ui';import config from '../tsrust.config.json';import { Page } from './app/page';function main(){unwrap(run(withTheme(Page(),config.ui.theme), 'Tsrust UI'));}";
pub(super) const UI_PAGE: &str = r#"import { View, Text, signal, Node } from 'std:ui';
import { Button } from '../components/ui/button';
import { Card } from '../components/ui/card';
export function Page(): Node {
  const count = signal(0);
  const increment = () => { count.set(count.get() + 1); };
  return (
    <View className="gap-4 p-4 bg-background">
      <Card>
        <Text className="text-xl font-bold">Count: {count}</Text>
        <Button onPress={increment}>Increment</Button>
      </Card>
    </View>
  );
}
"#;
const UI_TEST: &str = "import { Page } from '../src/app/page';import { document } from 'std:ui';import { test, strictEqual } from 'std:testing';test('renders root page', (): void => {const html = document(Page(), 'UI');strictEqual(html.includes('Count: 0'), true);});";

pub(super) const UI_EMPTY_PAGE: &str = "import {View,Text,Node} from 'std:ui';export function Page():Node{return (<View className=\"p-4 gap-4 bg-background\"><Text className=\"text-xl\">My application</Text></View>);}";
