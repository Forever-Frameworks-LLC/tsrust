use tsrust_compiler::api::*;
use std::collections::BTreeMap;
use tsrust_compiler::project::{ProjectAnalysis, SourceProvider, SourceId, load_project, check_graph};

struct Sources(BTreeMap<String, String>);
impl SourceProvider for Sources {
    fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.trim_start_matches("./").into())) }
    fn read(&self, id: &SourceId) -> Result<String, String> { self.0.get(&id.0).cloned().ok_or_else(|| format!("missing {}", id.0)) }
}

fn project(source: &str, names: &str) -> ProjectAnalysis {
    let source = tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&source).unwrap(), source);
    let main = tsrust_compiler::format(&format!("import {{{names}}} from './operations.tsr'; function main() {{}} ")).unwrap();
    let provider = Sources(BTreeMap::from([("main.tsr".into(), main), ("operations.tsr".into(), source)]));
    check_graph(load_project(&provider, "main.tsr").unwrap()).unwrap_or_else(|error| panic!("{}", error.message))
}

fn select(name: &str, export: &str) -> Selection { Selection { name: name.into(), export: export.into(), module: "operations.tsr".into(), kind: Kind::Query } }

#[test]
fn checked_contracts_preserve_optional_nullable_literals_and_order() {
    let project = project("type Input={search:string|null;completed?:boolean;};type Item={kind:'task';id:int;};export function list(input:Input):Item[]{return [{kind:'task',id:7}];}export function ping():string{return '__TSRUST_API_ID__';}", "list,ping");
    let manifest = Manifest::from_project(&project, &[select("tasks.list", "list"), select("health", "ping")]).unwrap();
    assert_eq!(manifest.operations[0].name, "health");
    let Schema::Object { properties, required } = &manifest.operations[1].input else { panic!("object input") };
    assert_eq!(required, &["search"]);
    assert_eq!(properties["completed"], Schema::Boolean);
    assert!(matches!(properties["search"], Schema::Union { .. }));
    let Schema::Array { items } = &manifest.operations[1].output else { panic!("array output") };
    let Schema::Object { properties, .. } = items.as_ref() else { panic!("item object") };
    assert_eq!(properties["kind"], Schema::Literal { value: serde_json::json!("task") });
    let reordered = Manifest::from_project(&project, &[select("health", "ping"), select("tasks.list", "list")]).unwrap();
    assert_eq!(manifest.identity(), reordered.identity());
    assert_eq!(manifest.client_package("@example/api", "0.1.0").unwrap(), reordered.client_package("@example/api", "0.1.0").unwrap());
}

#[test]
fn only_explicit_function_exports_are_eligible() {
    let project = project("function hidden():int{return 1;}export function visible():int{return hidden();}", "visible");
    let error = Manifest::from_project(&project, &[select("hidden", "hidden")]).err().unwrap();
    assert!(error.contains("explicit export"), "{error}");
    for name in ["then", "tasks.__proto__", "constructor.list", "tasks..list", "bad-name"] {
        assert!(Manifest::from_project(&project, &[select(name, "visible")]).is_err(), "{name}");
    }
    assert!(Manifest::from_project(&project, &[select("tasks", "visible"), select("tasks.list", "visible")]).err().unwrap().contains("conflicts"));
    assert!(Manifest::from_project(&project, &[select("tasks", "visible"), select("tasks", "visible")]).err().unwrap().contains("duplicate"));
}

#[test]
fn unsupported_wire_contracts_reject_without_emitting_a_package() {
    for (source, names, message) in [
        ("export function add(a:int,b:int):int{return a+b;}", "add", "request record"),
        ("export function add(a:int=1):int{return a;}", "add", "default"),
        ("export function add(a:Map<string,int>):int{return 1;}", "add", "plain wire data"),
    ] {
        let checked = project(source, names);
        let error = Manifest::from_project(&checked, &[select("test", names)]).err().unwrap();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn async_operations_describe_eventual_plain_results() {
    let project = project("export async function count(input:int):Task<int>{return input+1;}", "count");
    let manifest = Manifest::from_project(&project, &[select("count", "count")]).unwrap();
    assert_eq!(manifest.operations[0].output, Schema::Integer);
}

#[test]
fn package_metadata_is_portable_and_has_no_runtime_dependencies() {
    let project = project("export function value():int{return 7;}", "value");
    let manifest = Manifest::from_project(&project, &[select("value", "value")]).unwrap();
    let files = manifest.client_package("@example/api", "0.1.0").unwrap();
    let package: serde_json::Value = serde_json::from_str(&files["package.json"]).unwrap();
    assert!(package.get("dependencies").is_none());
    assert!(package.get("scripts").is_none());
    assert_eq!(package["private"], true);
    assert!(!files["index.js"].contains("operations.tsr"));
    assert!(files["index.d.ts"].contains("Promise<number>"));
    for name in ["../bad", "Name", "@scope/../bad", "@/bad"] { assert!(manifest.client_package(name, "0.1.0").is_err()); }
    assert!(manifest.client_package("api", "01.0.0").is_err());
}
