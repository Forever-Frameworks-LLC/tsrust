//! `tsrust ui list|add` and the `tsrust new --ui` template. `add` copies an
//! editable `.tsrx` wrapper per GPUI Kit component (shadcn style) into the
//! project's `src/components/`; the wrappers call the compiler-owned `std:ui`
//! bindings, so projects own styling and defaults without owning the renderer.
use std::{fs,path::Path};

const CATALOG:&str=include_str!("../stdlib/ui/kit.json");
const USAGE:&str="Usage: tsrust ui list | tsrust ui add [Component...] [--all] [--force]";

struct Component {name:String,doc:String,example:String,item:bool,children:Option<String>}

fn catalog()->Vec<Component> {
    let spec:serde_json::Value=serde_json::from_str(CATALOG).expect("bundled UI catalog");
    spec["components"].as_array().into_iter().flatten().map(|entry|Component{
        name:entry["name"].as_str().unwrap_or_default().to_owned(),
        doc:entry["doc"].as_str().unwrap_or_default().to_owned(),
        example:entry["example"].as_str().unwrap_or_default().to_owned(),
        item:entry["item"].as_bool().unwrap_or(false),
        children:entry["children"].as_str().map(str::to_owned),
    }).collect()
}

pub(super) fn run(mut args:impl Iterator<Item=String>)->Result<u8,String> {
    match args.next().as_deref() {
        Some("list")=>{for component in catalog().iter().filter(|component|!component.item){println!("{:<18} {}",component.name,component.doc);}Ok(0)}
        Some("add")=>add(args),
        _=>Err(USAGE.into()),
    }
}

fn add(args:impl Iterator<Item=String>)->Result<u8,String> {
    let catalog=catalog();
    let (mut names,mut all,mut force)=(Vec::new(),false,false);
    for arg in args {match arg.as_str(){"--all"=>all=true,"--force"=>force=true,flag if flag.starts_with('-')=>return Err(format!("Unknown option {flag}. {USAGE}")),_=>names.push(arg)}}
    let top=catalog.iter().filter(|component|!component.item).collect::<Vec<_>>();
    let mut selected=if all {top.clone()} else {
        names.iter().map(|name|top.iter().copied().find(|component|component.name.eq_ignore_ascii_case(name)).ok_or_else(|| format!("Unknown component {name}; run tsrust ui list"))).collect::<Result<Vec<_>,_>>()?
    };
    if selected.is_empty() {
        if !super::interactive_cli::available() {return Err(USAGE.into());}
        let choices=top.iter().map(|component|(component.name.as_str(),component.doc.as_str())).collect::<Vec<_>>();
        let Some(indices)=super::interactive_cli::multiselect("Add components",&choices)? else{return Ok(0);};
        selected=indices.into_iter().map(|index|top[index]).collect();
    }
    let directory=super::project_config::default_root()?.join("src/components");
    fs::create_dir_all(&directory).map_err(|e|format!("Cannot create {}: {e}",directory.display()))?;
    for component in selected {
        let path=directory.join(format!("{}.tsrx",component.name));
        if path.exists() && !force {println!("Kept {} (exists; --force replaces it)",path.display());continue;}
        let source=tsrust_compiler::format(&wrapper(component,&catalog)).map_err(|error|format!("Cannot format {} wrapper: {}",component.name,error.message))?;
        fs::write(&path,source).map_err(|e|e.to_string())?;
        println!("Added {}\n  import {{ {} }} from './components/{}.tsrx';\n  {}",path.display(),exports(component,&catalog).join(", "),component.name,component.example);
    }
    Ok(0)
}

/// A component and the item components it contains (Settings → SettingsPage → …).
fn family<'a>(component:&'a Component,catalog:&'a [Component])->Vec<&'a Component> {
    let mut found=vec![component];
    // `children` names one item component, or a list such as "PlotLine, PlotArea and PlotBars".
    let names=|children:&Option<String>|children.iter().flat_map(|text|text.split([',',' ']).filter(|word|!word.is_empty() && *word!="and" && *word!="or").map(str::to_owned).collect::<Vec<_>>()).collect::<Vec<_>>();
    let mut pending=names(&component.children);
    pending.reverse();
    while let Some(name)=pending.pop() {
        if let Some(child)=catalog.iter().find(|entry|entry.name==name && entry.item) && !found.iter().any(|entry|entry.name==child.name) {
            found.push(child);
            let mut nested=names(&child.children);nested.reverse();pending.extend(nested);
        }
    }
    found
}

fn exports(component:&Component,catalog:&[Component])->Vec<String> {family(component,catalog).iter().map(|entry|entry.name.clone()).collect()}

fn wrapper(component:&Component,catalog:&[Component])->String {
    let members=family(component,catalog);
    let imports=members.iter().map(|entry|format!("{0} as Kit{0}, {0}Props",entry.name)).collect::<Vec<_>>().join(", ");
    let mut source=format!("// Added by `tsrust ui add`. This file is yours: change defaults, classes or props.\nimport {{ Node, {imports} }} from 'std:ui';\n");
    for entry in members {
        source.push_str(&format!("\n/** {} */\nexport function {1}(props: {1}Props): Node {{\n  return Kit{1}(props);\n}}\n",entry.doc,entry.name));
    }
    source
}

/// `tsrust new <name> --ui`: a small desktop app (src/cli/ui-template) with live reload.
pub(super) fn create_app(root:&Path)->Result<(),String> {
    let name=root.file_name().and_then(|value|value.to_str()).unwrap_or("app");
    fs::create_dir(root).map_err(|e|format!("Cannot create {}: {e}. Existing projects are never overwritten.",root.display()))?;
    let config=serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":"src/main.tsrx","output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)}});
    let files:[(&str,&[u8]);6]=[
        ("src/main.tsrx",include_bytes!("ui-template/main.tsrx")),
        ("src/locales/en.json",include_bytes!("ui-template/locales/en.json")),
        ("src/locales/fr.json",include_bytes!("ui-template/locales/fr.json")),
        ("icon.png",include_bytes!("ui-template/icon.png")),
        (".gitignore",b"/target/\n/dist/\n"),
        ("AGENTS.md",super::new_cli::AGENT_GUIDANCE.as_bytes()),
    ];
    for (path,bytes) in files {
        let path=root.join(path);
        if let Some(parent)=path.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
        fs::write(path,bytes).map_err(|e|e.to_string())?;
    }
    fs::write(root.join(super::project_config::FILE),format!("{}\n",serde_json::to_string_pretty(&config).unwrap())).map_err(|e|e.to_string())?;
    fs::write(root.join("README.md"),format!("# {name}\n\nA GPUI desktop app written in `.tsrx`.\n\n- `tsrust dev`: run with live reload; saving a file rebuilds and restarts the window.\n- `tsrust ui list`: GPUI Kit components; `tsrust ui add Settings` copies an editable wrapper into `src/components/`.\n- `tsrust package src/main.tsrx --name {name}`: build the app, zip and installer into `dist/`.\n\nTranslations live in `src/locales/`.\n")).map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrappers_cover_item_components_and_format() {
        let catalog=catalog();
        let settings=catalog.iter().find(|component|component.name=="Settings").unwrap();
        assert_eq!(exports(settings,&catalog),["Settings","SettingsPage","SettingsGroup","SettingItem"]);
        let source=tsrust_compiler::format(&wrapper(settings,&catalog)).unwrap();
        assert!(source.contains("export function SettingItem(props: SettingItemProps): Node"));
        let plot=catalog.iter().find(|component|component.name=="Plot").unwrap();
        assert_eq!(exports(plot,&catalog),["Plot","PlotLine","PlotArea","PlotBars"]);
        assert!(catalog.iter().filter(|component|!component.item).count()>60);
    }
}
