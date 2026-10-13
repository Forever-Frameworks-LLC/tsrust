//! Missing and null remain separate states in typed composite validation.

use super::{issues::{Issues, Segment}, scalar::Validate};
use crate::stdlib::json::Value;

pub(super) struct OptionalPlan<P>(pub P);
pub(super) struct NullablePlan<P>(pub P);
impl<P: Validate> Validate for OptionalPlan<P> {
    type Output = Option<P::Output>;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<Self::Output> {
        match input {
            None => Some(None),
            present => self.0.validate(present,path,issues).map(Some),
        }
    }
}
impl<P: Validate> Validate for NullablePlan<P> {
    type Output = Option<P::Output>;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<Self::Output> {
        match input {
            Some(Value::Null) => Some(None),
            other => self.0.validate(other,path,issues).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::{json::Value,schema::{scalar::{Validate,StringPlan},issues::{Issues,Code,Kind,Segment,OwnedSegment},array::ArrayPlan}};
    #[test]
    fn optional_accepts_missing_but_not_null() {
        let plan=OptionalPlan(StringPlan);let mut issues=Issues::new(4,8,128);
        assert_eq!(plan.validate(None,&[],&mut issues),Some(None));
        assert_eq!(plan.validate(Some(Value::Null),&[],&mut issues),None);
        assert_eq!(issues.items[0].code,Code::Type);
        assert_eq!(issues.items[0].observed,Kind::Null);
    }
    #[test]
    fn nullable_accepts_null_but_not_missing() {
        let plan=NullablePlan(StringPlan);let mut issues=Issues::new(4,8,128);
        assert_eq!(plan.validate(Some(Value::Null),&[],&mut issues),Some(None));
        assert_eq!(plan.validate(None,&[],&mut issues),None);
        assert_eq!(issues.items[0].code,Code::Missing);
        assert_eq!(issues.items[0].observed,Kind::Missing);
    }
    #[test]
    fn nested_presence_preserves_states_and_owned_storage() {
        let plan=OptionalPlan(NullablePlan(StringPlan));let mut issues=Issues::new(4,8,128);
        assert_eq!(plan.validate(None,&[],&mut issues),Some(None));
        assert_eq!(plan.validate(Some(Value::Null),&[],&mut issues),Some(Some(None)));
        let text=String::from("storage survives");let pointer=text.as_ptr();let capacity=text.capacity();
        let output=plan.validate(Some(Value::String(text)),&[],&mut issues).unwrap().unwrap().unwrap();
        assert_eq!(output.as_ptr(),pointer);assert_eq!(output.capacity(),capacity);assert!(issues.is_valid());
    }
    #[test]
    fn invalid_inner_values_keep_array_paths_with_zero_budget_rejection() {
        let plan=ArrayPlan(NullablePlan(StringPlan));let mut issues=Issues::new(1,8,128);
        let input=Value::Array(vec![Value::Null,Value::String("ok".into()),Value::Bool(false)]);
        assert!(plan.validate(Some(input),&[Segment::Field("values")],&mut issues).is_none());
        assert_eq!(issues.items[0].path,[OwnedSegment::Field("values".into()),OwnedSegment::Index(2)]);
        let mut hidden=Issues::new(0,0,0);
        assert!(OptionalPlan(NullablePlan(StringPlan)).validate(Some(Value::Bool(true)),&[],&mut hidden).is_none());
        assert!(!hidden.is_valid()&&hidden.truncated);
    }
}
