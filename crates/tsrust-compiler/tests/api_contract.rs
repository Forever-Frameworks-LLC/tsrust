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

fn select(name: &str, export: &str) -> Selection { Selection { name: name.into(), export: export.into(), module: "operations.tsr".into(), kind: Kind::Query, context: false } }

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
fn explicitly_selected_native_context_stays_out_of_public_contracts() {
    let checked = project("import {Context} from 'std:http';export async function count(context:Mutable<Context>,input:int):Task<int>{return input+context.req.method().length;}", "count");
    let mut selection = select("count", "count");
    assert!(Manifest::from_project(&checked, &[selection.clone()]).is_err(), "context is never hidden implicitly");
    selection.context = true;
    let api = CheckedApi::from_project(&checked, &[selection]).unwrap();
    assert_eq!(api.manifest.operations[0].input, Schema::Integer);
    assert_eq!(api.manifest.operations[0].output, Schema::Integer);
    let plain = project("export async function count(input:int):Task<int>{return input;}", "count");
    assert_eq!(api.manifest.identity(), Manifest::from_project(&plain, &[select("count", "count")]).unwrap().identity());
    for contents in api.manifest.client_package("api", "0.1.0").unwrap().values() {
        assert!(!contents.contains("std:http"));
        assert!(!contents.contains("Mutable<Context>"));
    }
    let zero = project("import {Context} from 'std:http';export function ping(context:ReadOnly<Context>):string{return context.req.method();}", "ping");
    let mut selection = select("ping", "ping");
    selection.context = true;
    assert_eq!(Manifest::from_project(&zero, &[selection]).unwrap().operations[0].input, Schema::Void);
}

#[test]
fn context_selection_cannot_hide_owned_contexts_or_arbitrary_resources() {
    for source in [
        "import {Context} from 'std:http';export function value(context:Owned<Context>):int{return 1;}",
        "import {Pool} from 'std:sql';export function value(pool:ReadOnly<Pool>):int{return 1;}",
        "export function value(input:int):int{return input;}",
        "type FakeContext={value:int;};export function value(input:ReadOnly<FakeContext>):int{return input.value;}",
        "export function value():int{return 1;}",
    ] {
        let checked = project(source, "value");
        let mut selection = select("value", "value");
        selection.context = true;
        let error = Manifest::from_project(&checked, &[selection]).err().unwrap();
        assert!(error.contains("Context"), "{error}");
    }
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
    assert_eq!(package["exports"]["./http"]["types"], "./http.d.ts");
    assert!(files["http.d.ts"].contains("typeof globalThis.fetch"));
    assert!(!files["index.d.ts"].contains("HeadersInit"));
    assert!(!files["index.js"].contains("from './http.js'"));
    assert_eq!(package["exports"]["./react-query"]["types"], "./react-query.d.ts");
    assert_eq!(package["peerDependenciesMeta"]["@tanstack/react-query"]["optional"], true);
    assert_eq!(package["peerDependenciesMeta"]["react"]["optional"], true);
    assert_eq!(package["peerDependenciesMeta"]["@tanstack/query-core"]["optional"], true);
    assert_eq!(package["exports"]["./query"]["types"], "./query.d.ts");
    assert!(files["query.js"].contains("from '@tanstack/query-core'"));
    assert!(!files["query.js"].contains("@tanstack/react-query"));
    assert!(!files["query.d.ts"].contains("@tanstack/react-query"));
    assert!(files["react-query.js"].contains("from './query.js'"));
    assert!(!files["index.d.ts"].contains("@tanstack"));
    assert!(!files["wire.js"].contains("@tanstack"));
    assert!(files["react-query.d.ts"].contains("MutationOperation<Client[\"value\"]") || files["react-query.d.ts"].contains("QueryOperation<Client[\"value\"]"));
    for name in ["../bad", "Name", "@scope/../bad", "@/bad"] { assert!(manifest.client_package(name, "0.1.0").is_err()); }
    assert!(manifest.client_package("api", "01.0.0").is_err());
}

fn object(fields: &[(&str, Schema)], required: &[&str]) -> Schema {
    Schema::Object { properties: fields.iter().map(|(name, shape)| (name.to_string(), shape.clone())).collect(), required: required.iter().map(|name| name.to_string()).collect() }
}

#[test]
fn rolling_contracts_require_safe_inputs_and_projectable_outputs() {
    let old_input = object(&[("title", Schema::String)], &["title"]);
    let old_output = object(&[("id", Schema::Integer), ("title", Schema::String)], &["id", "title"]);
    let old = Operation { name: "tasks.create".into(), kind: Kind::Mutation, input: old_input.clone(), output: old_output.clone() };
    let mut current = old.clone();
    current.input = object(&[("title", Schema::String), ("completed", Schema::Boolean)], &["title"]);
    current.output = object(&[("id", Schema::Integer), ("title", Schema::String), ("completed", Schema::Boolean)], &["id", "title", "completed"]);
    assert!(current.supports_previous(&old), "optional new inputs and projected new output fields are compatible");
    let compatible = current.clone();
    current.input = object(&[("title", Schema::String), ("completed", Schema::Boolean)], &["title", "completed"]);
    assert!(!current.supports_previous(&old), "new required input is incompatible");
    current = compatible.clone();
    current.output = object(&[("id", Schema::Number), ("title", Schema::String)], &["id", "title"]);
    assert!(!current.supports_previous(&old), "old safe integer outputs cannot accept arbitrary numbers");
    current = compatible.clone();
    current.output = object(&[("id", Schema::Integer), ("title", Schema::String)], &["id"]);
    assert!(!current.supports_previous(&old), "old required output cannot become optional");
    current = compatible.clone(); current.kind = Kind::Query;
    assert!(!current.supports_previous(&old));
    let old_nested = Operation { output: object(&[("item", old_output)], &["item"]), ..old.clone() };
    let current_nested = Operation { output: object(&[("item", compatible.output)], &["item"]), ..old.clone() };
    assert!(current_nested.supports_previous(&old_nested), "projection applies recursively");
    let old_optional = Operation { input: object(&[("title", Schema::String), ("extra", Schema::Boolean)], &["title"]), ..old.clone() };
    assert!(!old.supports_previous(&old_optional), "old optional fields may still be supplied; removing one is breaking");
    let old_literal = Operation { input: Schema::Literal { value: serde_json::json!("a") }, output: Schema::String, ..old.clone() };
    let new_literal = Operation { input: Schema::String, output: Schema::Literal { value: serde_json::json!("b") }, ..old };
    assert!(new_literal.supports_previous(&old_literal), "wider inputs and narrower outputs are safe");
}

#[test]
fn retained_manifests_reject_malformed_shapes_and_namespace_conflicts() {
    let operation = Operation { name: "value".into(), kind: Kind::Query, input: Schema::Void, output: Schema::String };
    let mut manifest = Manifest { schema_version: 1, operations: vec![operation.clone()] };
    manifest.validate().unwrap();
    let round_trip: Manifest = serde_json::from_str(&serde_json::to_string(&manifest).unwrap()).unwrap();
    assert_eq!(manifest, round_trip);
    for shape in [
        object(&[("a", Schema::String)], &["missing"]),
        object(&[("a", Schema::String)], &["a", "a"]),
        Schema::Union { members: vec![] },
        Schema::Array { items: Box::new(Schema::Void) },
        Schema::Literal { value: serde_json::json!({"not":"literal"}) },
        Schema::Literal { value: serde_json::json!(9007199254740993_u64) },
    ] {
        manifest.operations[0].output = shape;
        assert!(manifest.validate().is_err());
    }
    manifest.operations = vec![operation.clone(), Operation { name: "value.child".into(), ..operation }];
    assert!(manifest.validate().is_err());
    assert!(serde_json::from_str::<Manifest>(r#"{"schemaVersion":1,"operations":[],"extra":true}"#).is_err());
}

#[test]
fn versioned_sdk_baselines_protect_both_sides_of_a_replica_rollout() {
    let old = Manifest { schema_version: 1, operations: vec![Operation {
        name: "tasks.list".into(), kind: Kind::Query,
        input: object(&[("after", Schema::Integer)], &["after"]),
        output: object(&[("id", Schema::Integer)], &["id"]),
    }] };
    let history = ApiHistory::new("@example/api").advance("@example/api", "1.0.0", 1, &old).unwrap();
    assert_eq!(history.advance("@example/api", "1.0.0", 1, &old).unwrap(), history);
    let mut added = old.clone();
    added.operations.push(Operation { name: "health".into(), kind: Kind::Query, input: Schema::Void, output: Schema::Boolean });
    assert!(history.advance("@example/api", "1.1.0", 1, &added).is_ok());
    assert!(history.advance("@example/api", "1.0.0", 1, &added).unwrap_err().contains("SDK version"));
    let mut optional = old.clone();
    optional.operations[0].input = object(&[("after", Schema::Integer), ("search", Schema::String)], &["after"]);
    optional.operations[0].output = object(&[("id", Schema::Integer), ("revision", Schema::Integer)], &["id"]);
    assert!(history.advance("@example/api", "1.1.0", 1, &optional).is_ok());
    let mut required = optional.clone();
    required.operations[0].output = object(&[("id", Schema::Integer), ("revision", Schema::Integer)], &["id", "revision"]);
    assert!(required.operations[0].supports_previous(&old.operations[0]));
    assert!(!required.operations[0].accepts_previous_result(&old.operations[0]));
    assert!(history.advance("@example/api", "1.1.0", 1, &required).unwrap_err().contains("older replicas"));
    assert!(history.advance("@example/api", "1.1.0", 2, &required).unwrap_err().contains("SDK major"));
    assert!(history.advance("@example/api", "2.0.0", 2, &required).is_ok());
    let mut removed = old.clone(); removed.operations.clear();
    assert!(history.advance("@example/api", "1.1.0", 1, &removed).unwrap_err().contains("remove operation"));
    let unchanged = history.advance("@example/api", "1.0.1", 1, &old).unwrap();
    assert_eq!(unchanged.releases.len(), 1, "unchanged wire schemas are not accumulated as snapshots");
    assert_eq!(unchanged.sdk_releases.len(), 2, "SDK version references remain immutable without duplicating schemas");
    assert!(unchanged.advance("@example/api", "1.0.0", 1, &old).is_err());
    assert!(history.advance("@other/api", "1.1.0", 1, &old).is_err());
    for invalid in ["1.0", "01.0.0", "1.0.0-extra", "4294967296.0.0"] { assert!(history.advance("@example/api", invalid, 1, &old).is_err()); }
    let round_trip: ApiHistory = serde_json::from_str(&serde_json::to_string(&history).unwrap()).unwrap();
    assert_eq!(round_trip, history);
    assert_ne!(old.wire_identity(1), old.wire_identity(2), "identical schemas in different API majors are distinct");
    assert_eq!(old.wire_identity(1), old.identity(), "legacy API v1 identities remain usable");
    let mut merged = history.clone(); merged.merge(&unchanged).unwrap();
    assert_eq!(merged, unchanged);
    let conflicting = ApiHistory::new("@example/api").advance("@example/api", "1.0.0", 1, &added).unwrap();
    assert!(merged.merge(&conflicting).is_err());
    assert_eq!(merged, unchanged, "failed merges do not replace history");
}
