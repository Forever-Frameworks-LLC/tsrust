use std::{fs,path::PathBuf};
use super::project_config::{FILE,valid_name};

pub fn run(args:impl Iterator<Item=String>)->Result<u8,String> {
    let args=args.collect::<Vec<_>>();
    let ui=args.iter().filter(|arg|arg.as_str()=="--ui").count()==1;
    let paths=args.iter().filter(|arg|!arg.starts_with('-')).collect::<Vec<_>>();
    if paths.len()!=1 || args.len()!=1+usize::from(ui) {return Err("Usage: tsrust new <name-or-path> [--ui]".into());}
    let root=PathBuf::from(paths[0]);
    let name=root.file_name().and_then(|value|value.to_str()).filter(|name|valid_name(name))
        .ok_or("Project name must start with a letter and contain only ASCII letters, digits, '_' or '-' (maximum 100 characters)")?;
    if ui {super::verify_backend()?;}
    // Exclusive creation protects existing files, directories and symlinks.
    fs::create_dir(&root).map_err(|e|format!("Cannot create {}: {e}. Existing projects are never overwritten.",root.display()))?;
    let mut config=serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":"src/main.tsr","output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)}});
    if ui {config["ui"]=serde_json::to_value(tsrust_compiler::ui_project::Config::default()).unwrap();}
    fs::create_dir(root.join("src")).map_err(|e|e.to_string())?;
    fs::create_dir(root.join("tests")).map_err(|e|e.to_string())?;
    fs::write(root.join(FILE),format!("{}\n",serde_json::to_string_pretty(&config).unwrap())).map_err(|e|e.to_string())?;
    let greeting=format!("export function greeting():string{{return \"Hello from {name}!\";}}");
    let test=format!("import {{greeting}} from '../src/greeting.tsr';import {{test,strictEqual}} from 'std:testing';test('greets by project name',():void=>{{strictEqual(greeting(),\"Hello from {name}!\");}});");
    let sources=if ui {
        fs::create_dir(root.join("app")).map_err(|e|e.to_string())?;
        vec![("src/main.tsr", UI_MAIN), ("app/page.tsrx", UI_PAGE), ("tests/page.test.tsr", UI_TEST)]
    } else { vec![("src/main.tsr","import {greeting} from './greeting.tsr';function main(){console.log(greeting());}"),("src/greeting.tsr",greeting.as_str()),("tests/greeting.test.tsr",test.as_str())] };
    for (path,source) in sources {
        let formatted=tsrust_compiler::format(source).map_err(|error|format!("Cannot format scaffold: {}",error.message))?;
        fs::write(root.join(path),formatted).map_err(|e|e.to_string())?;
    }
    fs::write(root.join(".gitignore"),"/target/\n").map_err(|e|e.to_string())?;
    fs::write(root.join("README.md"),format!("# {name}\n\nRun `tsrust dev` to rebuild and restart on source changes.\n\n- `tsrust run`: run once.\n- `tsrust build`: build the configured binary.\n- `tsrust check`: check project sources.\n- `tsrust test`: run project tests.\n- `tsrust format`: format project sources.\n\nThe nearest `{FILE}` defines the project root, entry and output.\nA `main` entrypoint is optional; this template includes one to print a greeting.\n")).map_err(|e|e.to_string())?;
    if ui {
        super::ui_cli::install_defaults(&root,&tsrust_compiler::ui_project::Config::default())?;
        fs::write(root.join("README.md"), format!("# {name}\n\nRun `tsrust dev` to open the native UI and restart after edits.\n\n`tsrust.config.json` declares `src/main.tsr` as the executable entry. That visible bootstrap mounts `Page` from `app/page.tsrx`, the root screen convention. Put application state and handlers in components, or import them from ordinary source modules.\n\n`tsrust check`, `tsrust test` and `tsrust format --check` validate this project. `tsrust build` writes the configured executable.\n\nEdit `ui.theme.json` for paired light/dark tokens and spacing, radius and text sizes. Select mode in `src/main.tsr`. Edit `components/ui/button.tsrx` and `card.tsrx` directly. The UI paths live in the `ui` field of `tsrust.config.json`. Commit the source, theme and `ui-components.lock.json` together.\n\n`tsrust ui list --bundled` lists offline payloads. `tsrust ui add button@1.0.0 --dry-run` previews the exact installation. Re-adding preserves customized files and lock bytes. `--path` selects an alternate component directory; occupied upgrades are refused. Installation never downloads dependencies.\n\nThis scaffold implements one root page. Nested route discovery, layouts, loaders, navigation, input editing and async UI remain unfinished; naming another file does not wire it into this explicit bootstrap.\n")).map_err(|e|e.to_string())?;
    }
    println!("Created {}.\nRun `tsrust dev` from that directory.",root.display());
    Ok(0)
}

const UI_MAIN: &str = "import { run, withTheme } from 'std:ui';import theme from '../ui.theme.json';import { Page } from '../app/page';function main(){unwrap(run(withTheme(Page(),theme), 'Tsrust UI'));}";
const UI_PAGE: &str = r#"import { View, Text, signal, Node } from 'std:ui';
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
const UI_TEST: &str = "import { Page } from '../app/page';import { document } from 'std:ui';import { test, strictEqual } from 'std:testing';test('renders root page', (): void => {const html = document(Page(), 'UI');strictEqual(html.includes('Count: 0'), true);});";
