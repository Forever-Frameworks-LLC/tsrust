//! Consuming array validation with typed output and partial-output cleanup.

use super::{issues::{Issues, Kind, Segment}, scalar::{Validate, reject}};
use crate::stdlib::json::Value;

pub(super) struct ArrayPlan<P>(pub P);
impl<P: Validate> Validate for ArrayPlan<P> {
    type Output = Vec<P::Output>;
    fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<Self::Output> {
        let values = match input {
            Some(Value::Array(values)) => values,
            other => { reject(other, Kind::Array, path, issues); return None; }
        };
        let mut output = Vec::new();
        let mut valid = true;
        let mut element_path = path.to_vec();
        element_path.push(Segment::Index(0));
        for (index, value) in values.into_iter().enumerate() {
            *element_path.last_mut().expect("array index slot") = Segment::Index(index);
            match self.0.validate(Some(value), &element_path, issues) {
                Some(value) if valid => output.push(value),
                Some(_) => {}, // Drop later successful outputs once this array cannot succeed.
                None => {
                    valid = false;
                    output.clear(); // Release partial output immediately, exactly once.
                }
            }
        }
        valid.then_some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{scalar::{StringPlan, BooleanPlan}, issues::OwnedSegment};
    use std::{cell::Cell, rc::Rc};
    #[test]
    fn successful_elements_keep_storage_and_order() {
        let text = String::from("first"); let pointer = text.as_ptr();
        let input = Value::Array(vec![Value::String(text), Value::String("second".into())]);
        let mut issues = Issues::new(4, 8, 128);
        let output: Vec<String> = ArrayPlan(StringPlan).validate(Some(input), &[], &mut issues).unwrap();
        assert_eq!(output, ["first", "second"]);
        assert_eq!(output[0].as_ptr(), pointer);
        assert!(issues.is_valid());
    }
    #[test]
    fn nested_errors_have_original_indices_and_bounded_issues() {
        let input = serde_json::from_str("[[true,0],[null,false],42]").unwrap();
        let mut issues = Issues::new(2, 8, 128);
        assert!(ArrayPlan(ArrayPlan(BooleanPlan)).validate(Some(input), &[Segment::Field("items")], &mut issues).is_none());
        assert_eq!(issues.items[0].path, vec![OwnedSegment::Field("items".into()), OwnedSegment::Index(0), OwnedSegment::Index(1)]);
        assert_eq!(issues.items[1].path, vec![OwnedSegment::Field("items".into()), OwnedSegment::Index(1), OwnedSegment::Index(0)]);
        assert!(issues.truncated);
    }
    #[test]
    fn empty_missing_null_and_zero_error_budget_are_distinct() {
        let plan = ArrayPlan(BooleanPlan);
        let mut issues = Issues::new(0, 0, 0);
        assert_eq!(plan.validate(Some(Value::Array(vec![])), &[], &mut issues), Some(vec![]));
        for input in [None, Some(Value::Null), Some(Value::Array(vec![Value::Null]))] {
            assert!(plan.validate(input, &[], &mut issues).is_none());
        }
        assert!(!issues.is_valid()); assert!(issues.truncated);
    }
    struct DropProbe(Rc<Cell<usize>>);
    impl Drop for DropProbe { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
    struct ProbePlan { drops: Rc<Cell<usize>>, visits: Rc<Cell<usize>> }
    impl Validate for ProbePlan {
        type Output = DropProbe;
        fn validate(&self, input: Option<Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<DropProbe> {
            self.visits.set(self.visits.get() + 1);
            BooleanPlan.validate(input, path, issues).map(|_| DropProbe(self.drops.clone()))
        }
    }
    #[test]
    fn failure_drops_partial_and_later_outputs_exactly_once() {
        let drops = Rc::new(Cell::new(0)); let visits = Rc::new(Cell::new(0));
        let plan = ArrayPlan(ProbePlan { drops: drops.clone(), visits: visits.clone() });
        let mut issues = Issues::new(1, 8, 128);
        let input = serde_json::from_str("[true,false,null,true,null,false]").unwrap();
        assert!(plan.validate(Some(input), &[], &mut issues).is_none());
        assert_eq!(visits.get(), 6); assert_eq!(drops.get(), 4);
        assert_eq!(issues.items.len(), 1); assert!(issues.truncated);
    }
}
