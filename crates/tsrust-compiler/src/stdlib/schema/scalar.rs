//! Typed, consuming scalar validators. Public builder spelling is separate.

use super::issues::{Code, Issues, Kind, Segment};
use crate::stdlib::json::Value;

/// Validators borrow their immutable plan and consume their input on every path.
/// Missing input is distinct from a present JSON null. Output is statically typed.
pub(super) trait Validate {
    type Output;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<Self::Output>;
}

pub(super) struct StringPlan;
pub(super) struct BooleanPlan;
pub(super) struct NullPlan;

pub(super) fn reject(input: Option<Value>, expected: Kind, path: &[Segment<'_>], issues: &mut Issues) {
    let observed = match input.as_ref() {
        None => Kind::Missing,
        Some(Value::Null) => Kind::Null,
        Some(Value::Bool(_)) => Kind::Boolean,
        Some(Value::Number(_)) => Kind::Number,
        Some(Value::String(_)) => Kind::String,
        Some(Value::Array(_)) => Kind::Array,
        Some(Value::Object(_)) => Kind::Object,
    };
    issues.push(path, if input.is_none() { Code::Missing } else { Code::Type }, expected, observed);
    // Owned input is dropped here, including nested containers on type mismatch.
}
impl Validate for StringPlan {
    type Output = String;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<String> {
        match input {
            Some(Value::String(text)) => Some(text),
            other => { reject(other, Kind::String, path, issues); None }
        }
    }
}
impl Validate for BooleanPlan {
    type Output = bool;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<bool> {
        match input {
            Some(Value::Bool(value)) => Some(value),
            other => { reject(other, Kind::Boolean, path, issues); None }
        }
    }
}
impl Validate for NullPlan {
    type Output = ();
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<()> {
        match input {
            Some(Value::Null) => Some(()),
            other => { reject(other, Kind::Null, path, issues); None }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn errors() -> Issues { Issues::new(4, 8, 128) }
    #[test]
    fn successful_string_validation_moves_existing_storage() {
        let text = String::from("retained allocation");
        let pointer = text.as_ptr();
        let capacity = text.capacity();
        let mut issues = errors();
        let output: String = StringPlan.validate(Some(Value::String(text)), &[], &mut issues).unwrap();
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.capacity(), capacity);
        assert_eq!(output, "retained allocation");
        assert!(issues.is_valid());
    }
    #[test]
    fn missing_and_null_are_distinct() {
        let mut issues = errors();
        assert_eq!(NullPlan.validate(Some(Value::Null), &[], &mut issues), Some(()));
        assert_eq!(NullPlan.validate(None, &[], &mut issues), None);
        assert_eq!(issues.items[0].code, Code::Missing);
        assert_eq!(issues.items[0].observed, Kind::Missing);
        assert_eq!(StringPlan.validate(Some(Value::Null), &[], &mut issues), None);
        assert_eq!(issues.items[1].code, Code::Type);
        assert_eq!(issues.items[1].observed, Kind::Null);
    }
    #[test]
    fn failures_do_not_retain_sensitive_input_values() {
        let mut issues = errors();
        assert_eq!(BooleanPlan.validate(Some(Value::String("secret-token".into())), &[Segment::Field("enabled")], &mut issues), None);
        assert!(!format!("{issues:?}").contains("secret-token"));
        assert_eq!(issues.items[0].expected, Kind::Boolean);
        assert_eq!(issues.items[0].observed, Kind::String);
    }
    #[test]
    fn immutable_plans_reuse_and_zero_budget_still_rejects() {
        let plan = BooleanPlan;
        let mut issues = Issues::new(0, 0, 0);
        assert_eq!(plan.validate(Some(Value::Bool(false)), &[], &mut issues), Some(false));
        assert_eq!(plan.validate(Some(Value::Bool(true)), &[], &mut issues), Some(true));
        assert_eq!(plan.validate(Some(Value::Array(vec![])), &[], &mut issues), None);
        assert!(!issues.is_valid());
        assert!(issues.items.is_empty() && issues.truncated);
    }
}
