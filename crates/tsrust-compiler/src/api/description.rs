//! Public, source-independent HTTP contract. Never contains dispatch imports.
use super::{ApiHistory, ApiRelease, Schema};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpDescription {
    pub schema_version: u32,
    pub protocol: String,
    pub history: ApiHistory,
}

impl HttpDescription {
    pub fn new(history: ApiHistory) -> Result<Self, String> {
        let value = Self { schema_version: 1, protocol: "tsrust-http".into(), history };
        value.current()?;
        Ok(value)
    }

    fn current(&self) -> Result<&ApiRelease, String> {
        if self.schema_version != 1 || self.protocol != "tsrust-http" {
            return Err("Unsupported HTTP API description version or protocol".into());
        }
        self.history.validate()?;
        self.history.releases.iter().rev().find(|release| release.sdk_version.is_some())
            .ok_or_else(|| "HTTP API description needs a current SDK release".into())
    }

    /// The caller bounds input bytes before parsing untrusted network/file data.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 1_048_576 { return Err("HTTP API description exceeds 1 MiB".into()); }
        let value: Self = serde_json::from_str(text).map_err(|error| format!("Invalid HTTP API description: {error}"))?;
        value.current()?;
        Ok(value)
    }

    pub fn typescript_package(&self) -> Result<BTreeMap<String, String>, String> {
        let current = self.current()?;
        current.manifest.client_package_versioned(&self.history.package_name,
            current.sdk_version.as_deref().ok_or("Missing SDK version")?, &self.history)
    }

    /// Describe the existing single-endpoint JSON RPC protocol, not invented REST routes.
    /// The application supplies the actual server URL and authentication policy.
    pub fn openapi(&self) -> Result<serde_json::Value, String> {
        use serde_json::json;
        let current = self.current()?;
        let requests = current.manifest.operations.iter().map(|operation| json!({
            "type":"object", "additionalProperties":false,
            "required":["operation","input"],
            "properties":{"operation":{"const":operation.name},"input":schema(&operation.input)}
        })).collect::<Vec<_>>();
        let results = current.manifest.operations.iter().map(|operation| json!({
            "type":"object", "additionalProperties":false, "required":["ok","value"],
            "properties":{"ok":{"const":true},"value":schema(&operation.output)}
        })).collect::<Vec<_>>();
        // An empty allowlist accepts no requests and has no success variants.
        let alternatives = |key: &str, values: Vec<serde_json::Value>| {
            if values.is_empty() { serde_json::Value::Bool(false) }
            else { json!({key:values}) }
        };
        let error = json!({"type":"object","required":["ok","error"],"additionalProperties":false,
            "properties":{"ok":{"const":false},"error":{"type":"object","required":["code","message"],
                "additionalProperties":false,"properties":{"code":{"type":"string"},"message":{"type":"string"}}}}});
        Ok(json!({"openapi":"3.1.1", "info":{"title":self.history.package_name,"version":current.sdk_version},
            "x-tsrust-http":self,
            "paths":{"/":{"post":{"operationId":"invoke",
                "description":"POST to the application-configured API endpoint. Authentication is application-owned. Requests carry x-tsrust-contract plus versioned application/json media parameters; use the included Tsrust description for compatible named clients.",
                "parameters":[{"name":"x-tsrust-contract","in":"header","required":true,"schema":{"type":"string"},"example":current.manifest.wire_identity(current.api_version)}],
                "requestBody":{"required":true,"content":{"application/json":{"schema":alternatives("oneOf",requests)}}},
                "responses":{"200":{"description":"Successful operation","content":{"application/json":{"schema":alternatives("anyOf",results)}}},
                    "default":{"description":"Validation, access, compatibility or server failure","content":{"application/json":{"schema":error}}}}
            }}}}))
    }
}

fn schema(value: &Schema) -> serde_json::Value {
    use serde_json::json;
    match value {
        Schema::String => json!({"type":"string"}),
        Schema::Integer => json!({"type":"integer","minimum":-9_007_199_254_740_991_i64,"maximum":9_007_199_254_740_991_i64}),
        Schema::Number => json!({"type":"number"}),
        Schema::Boolean => json!({"type":"boolean"}),
        Schema::Void | Schema::Null => json!({"type":"null"}),
        Schema::Literal { value } => json!({"const":value}),
        Schema::Array { items } => json!({"type":"array","items":schema(items)}),
        Schema::Tuple { items } => json!({"type":"array","prefixItems":items.iter().map(schema).collect::<Vec<_>>(),"items":false,"minItems":items.len(),"maxItems":items.len()}),
        Schema::Union { members } => json!({"anyOf":members.iter().map(schema).collect::<Vec<_>>()}),
        Schema::Object { properties, required } => json!({"type":"object","properties":properties.iter().map(|(name, value)| (name.clone(),schema(value))).collect::<BTreeMap<_,_>>(),"required":required,"additionalProperties":false}),
    }
}
