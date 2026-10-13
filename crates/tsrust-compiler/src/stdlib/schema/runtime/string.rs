//! Reusable Unicode code-point length constraints for owned strings.

use super::{issues::{Code, Issues, Kind, Segment}, scalar::{StringPlan, Validate}};
use crate::stdlib::json::Value;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InvalidLengthBounds;
pub(super) struct StringLengthPlan { min: Option<usize>, max: Option<usize> }
impl StringLengthPlan {
    pub(super) fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, InvalidLengthBounds> {
        if matches!((min,max),(Some(min),Some(max)) if min>max) { return Err(InvalidLengthBounds); }
        Ok(Self { min, max })
    }
}
impl Validate for StringLengthPlan {
    type Output = String;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<String> {
        let text=StringPlan.validate(input,path,issues)?;
        if self.min.is_none() && self.max.is_none() { return Some(text); }
        // Once the maximum is exceeded, the remaining text cannot change failure.
        // No decoded character buffer or replacement string is allocated.
        let mut length=0usize;
        for _ in text.chars() {
            if self.max==Some(length) {
                issues.push(path,Code::TooLarge,Kind::String,Kind::String);
                return None;
            }
            length+=1;
        }
        if self.min.is_some_and(|min|length<min) {
            issues.push(path,Code::TooSmall,Kind::String,Kind::String);
            return None;
        }
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::{json::Value, schema::{array::ArrayPlan, scalar::Validate, issues::{Issues, Code, Segment, OwnedSegment}}};
    #[test]
    fn length_counts_code_points_not_bytes_units_or_graphemes() {
        let plan=StringLengthPlan::new(Some(1),Some(1)).unwrap();
        for (text,valid) in [("",false),("a",true),("😀",true),("é",true),("e\u{301}",false),("👩‍💻",false)] {
            let mut issues=Issues::new(4,8,128);
            assert_eq!(plan.validate(Some(Value::String(text.into())),&[],&mut issues).is_some(),valid,"{text:?}");
            assert_eq!(issues.is_valid(),valid);
        }
    }
    #[test]
    fn success_moves_storage_and_bounds_are_inclusive() {
        let plan=StringLengthPlan::new(Some(2),Some(3)).unwrap();
        for text in ["ab","😀ab"] {
            let input=text.to_owned();let pointer=input.as_ptr();let capacity=input.capacity();
            let mut issues=Issues::new(4,8,128);
            let output=plan.validate(Some(Value::String(input)),&[],&mut issues).unwrap();
            assert_eq!(output,text);assert_eq!(output.as_ptr(),pointer);assert_eq!(output.capacity(),capacity);
        }
        assert!(StringLengthPlan::new(Some(3),Some(2)).is_err());
        assert!(StringLengthPlan::new(Some(0),Some(0)).is_ok());
    }
    #[test]
    fn nested_failures_preserve_paths_and_never_retain_input() {
        let plan=ArrayPlan(StringLengthPlan::new(Some(2),Some(3)).unwrap());
        let mut issues=Issues::new(4,8,128);
        let input=Value::Array(vec![Value::String("a".into()),Value::String("private-value".into()),Value::String("ok".into())]);
        assert!(plan.validate(Some(input),&[Segment::Field("names")],&mut issues).is_none());
        assert_eq!(issues.items[0].code,Code::TooSmall);
        assert_eq!(issues.items[1].code,Code::TooLarge);
        assert_eq!(issues.items[1].path,[OwnedSegment::Field("names".into()),OwnedSegment::Index(1)]);
        assert!(!format!("{issues:?}").contains("private-value"));
    }
    #[test]
    fn missing_wrong_type_and_zero_issue_budget_still_fail() {
        let plan=StringLengthPlan::new(None,Some(0)).unwrap();
        let mut issues=Issues::new(4,8,128);
        assert!(plan.validate(None,&[],&mut issues).is_none());
        assert_eq!(issues.items[0].code,Code::Missing);
        assert!(plan.validate(Some(Value::Null),&[],&mut issues).is_none());
        assert_eq!(issues.items[1].code,Code::Type);
        let mut hidden=Issues::new(0,0,0);
        assert!(plan.validate(Some(Value::String("x".into())),&[],&mut hidden).is_none());
        assert!(!hidden.is_valid()&&hidden.truncated);
        assert_eq!(plan.validate(Some(Value::String(String::new())),&[],&mut issues),Some(String::new()));
    }
}
