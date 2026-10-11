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
