use super::*;
struct Sources(BTreeMap<String, String>);
impl SourceProvider for Sources {
    fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        Ok(SourceId(path.trim_start_matches("./").into()))
    }
    fn read(&self, id: &SourceId) -> Result<String, String> { self.0.get(&id.0).cloned().ok_or_else(|| "missing".into()) }
}
fn sources(data: &str, path: &str) -> Sources {
    let text = format!("import settings from './{path}'; function main(): void {{ console.log(settings.port); }}");
    let text = crate::formatting::format(&text).unwrap();
    assert_eq!(crate::formatting::format(&text).unwrap(), text);
    Sources(BTreeMap::from([("main.tsr".into(), text), (path.into(), data.into())]))
}
#[test]
fn data_project_loads_default_export_and_original_diagnostics() {
    let provider = sources("{/*雪*/\"port\":8080,}", "settings.jsonc");
    let graph = load_project(&provider, "main.tsr").unwrap();
    let module = &graph.modules[&SourceId("settings.jsonc".into())];
    assert_eq!(module.source.text, "{/*雪*/\"port\":8080,}");
    assert!(module.exports.contains_key("default"));
    assert!(module.imports.is_empty());
    let text = "{/*雪*/\"port\":nope}";
    let error = load_project(&sources(text,"settings.jsonc"),"main.tsr").unwrap_err();
    assert_eq!(error.location.source, SourceId("settings.jsonc".into()));
    assert_eq!(error.location.span.start, text.find("nope").unwrap()+1);
    let error = load_project(&sources("{\"port\":8080,}","settings.json"),"main.tsr").unwrap_err();
    assert_eq!(error.code, "data-json");
}
#[test]
fn data_project_cache_revalidates_changed_data_and_rejects_named_exports() {
    let mut provider = sources("{\"port\":8080}", "settings.json");
    let mut cache = HeaderCache::default();
    load_project_cached(&provider,"main.tsr",&mut cache).unwrap();
    provider.0.insert("settings.json".into(),"{\"port\":18446744073709551615}".into());
    let error = load_project_cached(&provider,"main.tsr",&mut cache).unwrap_err();
    assert_eq!(error.code,"data-type");
    assert_eq!(error.location.source,SourceId("settings.json".into()));
    provider.0.insert("settings.json".into(),"{\"port\":80}".into());
    load_project_cached(&provider,"main.tsr",&mut cache).unwrap();
    provider.0.insert("main.tsr".into(),crate::formatting::format("import {port} from './settings.json'; function main(): void {}").unwrap());
    assert_eq!(load_project(&provider,"main.tsr").unwrap_err().code,"module-private");
}
#[test]
fn data_project_checks_shared_readonly_storage_and_explicit_clone() {
    let mut provider = sources(r#"{"port":8080,"host":"local","values":[1,2]}"#, "settings.json");
    provider.0.insert("dep.tsr".into(),crate::formatting::format("import shared from './settings.json'; export function otherPort(): int { return shared.port; }").unwrap());
    provider.0.insert("main.tsr".into(),crate::formatting::format("import settings from './settings.json'; import {otherPort} from './dep.tsr'; function port(): int { return settings.port; } function main(): void { console.log(port()); const copy = clone(settings); copy.port = 9000; console.log(copy.port); console.log(settings.port); console.log(settings.host); console.log(otherPort()); const read = () => settings.port; console.log(read()); }").unwrap());
    let checked = check_graph(load_project(&provider,"main.tsr").unwrap()).unwrap();
    let rust = checked.emit_rust();
    assert_eq!(rust.matches("static TSR_DATA").count(),1);
    let directory = std::env::temp_dir().join(format!("tsrust-data-project-{}",std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(directory.clone());
    let input=directory.join("main.rs");let output=directory.join("program");std::fs::write(&input,rust).unwrap();
    let built=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024"]).arg(&input).arg("-o").arg(&output).output().unwrap();
    assert!(built.status.success(),"{}",String::from_utf8_lossy(&built.stderr));
    let run=std::process::Command::new(&output).output().unwrap();assert!(run.status.success());assert_eq!(String::from_utf8(run.stdout).unwrap(),"8080\n9000\n8080\nlocal\n8080\n8080\n");
    for body in ["settings.port = 9;", "const owned = settings;", "settings.values.push(3);", "const owned = settings.host;", "const read = () => settings; const owned = read();"] {
        provider.0.insert("main.tsr".into(),crate::formatting::format(&format!("import settings from './settings.json'; function main(): void {{ {body} }}")).unwrap());
        let error=check_graph(load_project(&provider,"main.tsr").unwrap()).err().expect("readonly rejection");
        assert_eq!(error.code,"data-readonly","{}",error.message);
    }
}
