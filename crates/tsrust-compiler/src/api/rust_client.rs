//! Typed Rust models and named calls over the same checked HTTP contract.
use super::{ApiHistory, Manifest, Schema};
use std::collections::{BTreeMap, BTreeSet};

fn quoted(value: &str) -> String { format!("{value:?}") }
fn identifier(value: &str, fallback: &str) -> String {
    if !matches!(value, "self" | "Self" | "super" | "crate" | "_")
        && value.as_bytes().first().is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        format!("r#{value}")
    } else { format!("r#{fallback}") }
}

#[derive(Default)]
struct Models { next: usize, source: String }
impl Models {
    fn ty(&mut self, shape: &Schema) -> Result<String, String> {
        Ok(match shape {
            Schema::String => "String".into(),
            Schema::Integer => "i64".into(),
            Schema::Number => "f64".into(),
            Schema::Boolean => "bool".into(),
            Schema::Void | Schema::Null => "()".into(),
            Schema::Array { items } => format!("Vec<{}>", self.ty(items)?),
            Schema::Tuple { items } => {
                if items.is_empty() { "[(); 0]".into() }
                else {
                    if items.len() > 16 { return Err("Initial Rust SDK supports tuples up to 16 items".into()); }
                    format!("({},)", items.iter().map(|item| self.ty(item)).collect::<Result<Vec<_>, _>>()?.join(","))
                }
            }
            Schema::Object { properties, required } => {
                let name = format!("Model{}", self.next); self.next += 1;
                let mut fields = String::new();
                let mut used = BTreeSet::new();
                for (index, (field, shape)) in properties.iter().enumerate() {
                    let mut field_name = identifier(field, &format!("field_{index}"));
                    while !used.insert(field_name.clone()) { field_name.push('_'); }
                    let ty = self.ty(shape)?;
                    let optional = !required.contains(field);
                    fields.push_str(&format!("#[serde(rename = {})]\n", quoted(field)));
                    if optional { fields.push_str("#[serde(default, deserialize_with = \"present\", skip_serializing_if = \"Option::is_none\")]\n"); }
                    fields.push_str(&format!("pub {field_name}: {},\n", if optional { format!("Option<{ty}>") } else { ty }));
                }
                self.source.push_str(&format!("#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct {name} {{\n{fields}}}\n"));
                name
            }
            Schema::Union { members } => {
                let name = format!("Model{}", self.next); self.next += 1;
                let variants = members.iter().enumerate().map(|(index, shape)| self.ty(shape).map(|ty| format!("V{index}({ty}),"))).collect::<Result<Vec<_>, _>>()?.join("\n");
                self.source.push_str(&format!("#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]\n#[serde(untagged)]\npub enum {name} {{\n{variants}\n}}\n"));
                name
            }
            Schema::Literal { value } => {
                let name = format!("Model{}", self.next); self.next += 1;
                let json = quoted(&value.to_string());
                self.source.push_str(&format!(r#"
#[derive(Clone, Debug, PartialEq)]
pub struct {name};
impl Serialize for {name} {{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {{
        serde_json::from_str::<Value>({json}).expect("generated literal").serialize(serializer)
    }}
}}
impl<'de> Deserialize<'de> for {name} {{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {{
        let value = Value::deserialize(deserializer)?;
        let expected: Value = serde_json::from_str({json}).expect("generated literal");
        if literal_equal(&value, &expected) {{ Ok(Self) }} else {{ Err(serde::de::Error::custom("Unexpected API literal")) }}
    }}
}}
"#));
                name
            }
        })
    }
}

#[derive(Default)]
struct Namespace { children: BTreeMap<String, Namespace>, operation: Option<usize> }
fn namespace_source(node: &Namespace, name: &str, next: &mut usize, source: &mut String, manifest: &Manifest) {
    let mut methods = String::new();
    let mut used = BTreeSet::new();
    for (index, (segment, child)) in node.children.iter().enumerate() {
        let mut method = identifier(segment, &format!("operation_{index}"));
        while !used.insert(method.clone()) { method.push('_'); }
        if let Some(index) = child.operation {
            let operation = &manifest.operations[index];
            let (argument, value) = if operation.input == Schema::Void { (String::new(), "&()") } else { (format!("input: Op{index}Input"), "&input") };
            methods.push_str(&format!("pub async fn {method}(&self, {argument}) -> Result<Op{index}Output, ApiError> {{ self.client.call({index}, {value}).await }}\n"));
        } else {
            let nested = format!("Namespace{}", *next); *next += 1;
            methods.push_str(&format!("pub fn {method}(&self) -> {nested}<'_, T> {{ {nested} {{ client: self.client }} }}\n"));
            namespace_source(child, &nested, next, source, manifest);
        }
    }
    source.push_str(&format!("pub struct {name}<'a, T> {{ client: &'a Client<T> }}\nimpl<T: Transport> {name}<'_, T> {{\n{methods}}}\n"));
}

pub(super) fn package(manifest: &Manifest, history: &ApiHistory) -> Result<BTreeMap<String, String>, String> {
    history.validate()?; manifest.validate()?;
    let current = history.releases.iter().rev().find(|release| release.sdk_version.is_some()).ok_or("Rust SDK needs a current release")?;
    if current.manifest != *manifest { return Err("Rust SDK history does not match the current manifest".into()); }
    let mut models = Models::default();
    let mut aliases = String::new();
    let mut root = Namespace::default();
    let mut profiles = BTreeMap::new();
    let mut operations = BTreeMap::new();
    for (index, operation) in manifest.operations.iter().enumerate() {
        aliases.push_str(&format!("pub type Op{index}Input = {};\npub type Op{index}Output = {};\n", models.ty(&operation.input)?, models.ty(&operation.output)?));
        let mut node = &mut root;
        for segment in operation.name.split('.') { node = node.children.entry(segment.into()).or_default(); }
        node.operation = Some(index);
        operations.insert(operation.name.clone(), operation.wire_identity(current.api_version));
        profiles.insert(operation.name.clone(), history.releases.iter().filter(|release| release.api_version == current.api_version).filter_map(|release| {
            release.manifest.operations.iter().find(|old| operation.accepts_previous_result(old)).map(|old| serde_json::json!({"contractId":release.manifest.wire_identity(current.api_version),"input":old.input}))
        }).collect::<Vec<_>>());
    }
    let protocol = serde_json::json!({"apiVersion":current.api_version,"wireContractId":manifest.wire_identity(current.api_version),"operations":operations,"profiles":profiles});
    let mut namespaces = String::new();
    namespace_source(&root, "Api", &mut 0, &mut namespaces, manifest);
    // Namespaces are always available from api(); avoids a collision with transport constructors.
    let library = format!("// @generated by tsrust api. No backend source required.\n#![allow(non_snake_case)]\nuse serde::{{Serialize, Deserialize}};\nuse serde_json::Value;\nconst MANIFEST: &str = {};\nconst PROTOCOL: &str = {};\n{}\n{}\n{}\n{}\nimpl<T: Transport> Client<T> {{ pub fn api(&self) -> Api<'_, T> {{ Api {{ client:self }} }}\n}}\n",
        quoted(&manifest_string(manifest)?), quoted(&protocol.to_string()), include_str!("rust-runtime.rs.template"), models.source, aliases, namespaces);
    let name = format!("tsrust-{}", history.package_name.trim_start_matches('@').chars().map(|character| if character.is_ascii_alphanumeric() || character == '-' || character == '_' { character } else { '-' }).collect::<String>());
    let cargo = format!("[package]\nname = {}\nversion = {}\nedition = \"2024\"\npublish = false\n\n[workspace]\n\n[features]\ndefault = [\"http\"]\nhttp = [\"dep:reqwest\"]\n\n[dependencies]\nserde = {{ version = \"=1.0.229\", features = [\"derive\"] }}\nserde_json = \"=1.0.151\"\nreqwest = {{ version = \"=0.13.5\", default-features = false, features = [\"rustls\"], optional = true }}\n", quoted(&name), quoted(current.sdk_version.as_deref().unwrap()));
    let description = super::HttpDescription::new(history.clone())?;
    Ok(BTreeMap::from([
        ("Cargo.toml".into(), cargo), ("src/lib.rs".into(), library),
        ("http-api.json".into(), serde_json::to_string_pretty(&description).map_err(|error| error.to_string())?),
        ("openapi.json".into(), serde_json::to_string_pretty(&description.openapi()?).map_err(|error| error.to_string())?),
        ("compatibility.json".into(), serde_json::to_string_pretty(history).map_err(|error| error.to_string())?),
        ("TSRUST-LICENSE".into(), include_str!("../../../../LICENSE").into()),
        ("README.md".into(), "# Generated Rust HTTP SDK\n\nUse Client::new(transport).api().namespace().operation(input). The optional default http feature supplies async HttpTransport::new(url); disable default features for a custom Transport. Await named calls and run HTTP futures in the caller's Tokio runtime; custom transports return Send futures. Use owned generated DTOs; optional fields use Option, nullable unions preserve explicit null separately. Request/result codecs enforce the checked contract and safe integer range. HTTP has bounded bodies, timeouts, no redirects or retries; completion after network failures is unknown. with_header_provider asynchronously resolves credentials per call. Initial tuple support is limited to 16 elements and generation rejects larger tuples. Review generated types and licenses. Cargo dependency preparation is explicit; generation never installs packages.\n".into()),
    ]))
}
fn manifest_string(manifest: &Manifest) -> Result<String, String> { serde_json::to_string(manifest).map_err(|error| error.to_string()) }
