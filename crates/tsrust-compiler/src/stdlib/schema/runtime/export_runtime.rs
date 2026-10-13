//! Shared JSON Schema assembly. Generated visitors borrow plans and own output.
use super::tsrust_native_json::{Map, Value};

pub struct ExportError {
    pub path: std::string::String,
    pub reason: &'static str,
}

pub fn unsupported(path: &str, reason: &'static str) -> ExportError {
    ExportError { path: path.to_owned(), reason }
}

pub fn child_path(parent: &str, segment: &str) -> std::string::String {
    let mut path = parent.to_owned();
    path.push('/');
    for character in segment.chars() {
        match character {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            _ => path.push(character),
        }
    }
    path
}

pub fn object(fields: Vec<(&str, Value)>) -> Value {
    Value::Object(fields.into_iter().map(|(key, value)| (key.to_owned(), value)).collect())
}

pub fn typed(kind: &str) -> Value {
    object(vec![("type", Value::String(kind.to_owned()))])
}

pub fn keyword(schema: &mut Value, name: &str, value: Value) {
    schema.as_object_mut().expect("generated schema object").insert(name.to_owned(), value);
}

pub fn string(min: Option<u64>, max: Option<u64>) -> Value {
    let mut schema = typed("string");
    if let Some(min) = min { keyword(&mut schema, "minLength", Value::from(min)); }
    if let Some(max) = max { keyword(&mut schema, "maxLength", Value::from(max)); }
    schema
}

pub fn integer(min: Option<i64>, max: Option<i64>) -> Value {
    let mut schema = typed("integer");
    keyword(&mut schema, "minimum", Value::from(min.unwrap_or(i64::MIN)));
    keyword(&mut schema, "maximum", Value::from(max.unwrap_or(i64::MAX)));
    schema
}

pub fn number(min: Option<f64>, max: Option<f64>) -> Value {
    let mut schema = typed("number");
    // Construction already rejects nonfinite bounds. Do not let JSON conversion
    // silently replace an invalid bound with null.
    for (name, bound) in [("minimum", min), ("maximum", max)] {
        if let Some(bound) = bound {
            assert!(bound.is_finite(), "validated schema number bound");
            keyword(&mut schema, name, Value::from(bound));
        }
    }
    schema
}

pub fn reference(name: &str) -> Value {
    object(vec![("$ref", Value::String(format!("#/$defs/{name}")))])
}

pub fn array(element: Value) -> Value {
    let mut schema = typed("array");
    keyword(&mut schema, "items", element);
    schema
}

pub fn alternatives(branches: Vec<Value>) -> Value {
    // Ordered native branches may overlap. oneOf would incorrectly reject an
    // input accepted by more than one branch.
    object(vec![("anyOf", Value::Array(branches))])
}

pub fn nullable(schema: Value) -> Value {
    alternatives(vec![schema, typed("null")])
}

pub fn intersection(first: Value, second: Value) -> Value {
    object(vec![("allOf", Value::Array(vec![first, second]))])
}

pub fn record(element: Value) -> Value {
    let mut schema = typed("object");
    keyword(&mut schema, "additionalProperties", element);
    schema
}

pub fn fields(properties: Map<std::string::String, Value>, required: Vec<Value>) -> Value {
    let mut schema = typed("object");
    keyword(&mut schema, "properties", Value::Object(properties));
    keyword(&mut schema, "required", Value::Array(required));
    keyword(&mut schema, "additionalProperties", Value::Bool(false));
    schema
}

pub fn document(mut root: Value, definitions: Map<std::string::String, Value>) -> Value {
    keyword(&mut root, "$schema", Value::String("https://json-schema.org/draft/2020-12/schema".to_owned()));
    if !definitions.is_empty() { keyword(&mut root, "$defs", Value::Object(definitions)); }
    root
}
