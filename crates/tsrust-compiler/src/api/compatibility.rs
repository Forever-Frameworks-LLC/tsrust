//! Compatibility concerns accepted values, not source/total-manifest equality.
use super::{Manifest, Operation, Schema, valid_segment};
use std::collections::BTreeSet;

fn valid(shape: &Schema, root: bool, depth: usize) -> bool {
    if depth > 128 { return false; }
    match shape {
        Schema::Void => root,
        Schema::Literal { value } => match value {
            serde_json::Value::String(_) | serde_json::Value::Bool(_) => true,
            serde_json::Value::Number(number) => number.as_i64().map(|value| value.unsigned_abs() <= 9_007_199_254_740_991)
                .or_else(|| number.as_u64().map(|value| value <= 9_007_199_254_740_991))
                .unwrap_or_else(|| number.as_f64().is_some_and(f64::is_finite)),
            _ => false,
        },
        Schema::Array { items } => valid(items, false, depth + 1),
        Schema::Tuple { items } => items.iter().all(|item| valid(item, false, depth + 1)),
        Schema::Object { properties, required } => {
            let names = required.iter().collect::<BTreeSet<_>>();
            names.len() == required.len() && names.iter().all(|name| properties.contains_key(*name))
                && properties.values().all(|child| valid(child, false, depth + 1))
        }
        Schema::Union { members } => !members.is_empty() && members.iter().all(|member| valid(member, false, depth + 1)),
        _ => true,
    }
}

/// Every source value must be accepted by target. Projection can omit object
/// fields, including nested fields, only when producing an older response.
fn accepts(source: &Schema, target: &Schema, project: bool) -> bool {
    if source == target { return true; }
    if let Schema::Union { members } = source { return members.iter().all(|source| accepts(source, target, project)); }
    if let Schema::Union { members } = target { return members.iter().any(|target| accepts(source, target, project)); }
    match (source, target) {
        (Schema::Integer, Schema::Number) => true,
        (Schema::Literal { value }, Schema::String) => value.is_string(),
        (Schema::Literal { value }, Schema::Boolean) => value.is_boolean(),
        (Schema::Literal { value }, Schema::Number) => value.is_number(),
        (Schema::Literal { value }, Schema::Integer) => value.as_f64().is_some_and(|number| number.is_finite() && number.fract() == 0.0 && number.abs() <= 9_007_199_254_740_991.0),
        (Schema::Literal { value: left }, Schema::Literal { value: right }) => left == right || left.is_number() && right.is_number() && left.as_f64() == right.as_f64(),
        (Schema::Array { items: source }, Schema::Array { items: target }) => accepts(source, target, project),
        (Schema::Tuple { items: source }, Schema::Array { items: target }) => source.iter().all(|source| accepts(source, target, project)),
        (Schema::Tuple { items: source }, Schema::Tuple { items: target }) => source.len() == target.len() && source.iter().zip(target).all(|(source, target)| accepts(source, target, project)),
        (Schema::Object { properties: source, required: source_required }, Schema::Object { properties: target, required: target_required }) => {
            target_required.iter().all(|key| source_required.contains(key))
                && (project || source.keys().all(|key| target.contains_key(key)))
                && target.iter().all(|(key, target)| source.get(key).is_none_or(|source| accepts(source, target, project)))
        }
        _ => false,
    }
}

impl Manifest {
    /// Validate a retained public manifest before using it as compatibility policy.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 { return Err("Retained API manifest schemaVersion must be 1".into()); }
        let mut names = BTreeSet::new();
        for operation in &self.operations {
            if operation.name.len() > 200 || !operation.name.split('.').all(valid_segment) || !names.insert(operation.name.clone()) {
                return Err("Retained API manifest has invalid or duplicate operation names".into());
            }
            if !valid(&operation.input, true, 0) || !valid(&operation.output, true, 0) { return Err(format!("Retained API operation {:?} has an invalid wire schema", operation.name)); }
        }
        for name in &names {
            let mut prefix = name.as_str();
            while let Some((parent, _)) = prefix.rsplit_once('.') {
                if names.contains(parent) { return Err("Retained API manifest has conflicting operation namespaces".into()); }
                prefix = parent;
            }
        }
        Ok(())
    }
}

impl Operation {
    /// A new SDK can decode responses from the retained replica without inventing
    /// missing fields or weakening the checked public result type.
    pub fn accepts_previous_result(&self, previous: &Operation) -> bool {
        self.name == previous.name && self.kind == previous.kind
            && accepts(&previous.output, &self.output, false)
    }
    /// An old caller's inputs still work and new results can be safely projected
    /// to that caller's strict output shape. Application semantics remain its own.
    pub fn supports_previous(&self, previous: &Operation) -> bool {
        self.name == previous.name && self.kind == previous.kind
            && accepts(&previous.input, &self.input, false)
            && accepts(&self.output, &previous.output, true)
    }
}
