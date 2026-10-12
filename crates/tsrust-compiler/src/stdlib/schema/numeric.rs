//! Reusable numeric validation using the accepted checked JSON extraction rules.

use super::issues::{Code, Issues, Kind, Segment};
use crate::stdlib::json;

#[derive(Debug, Clone, Copy)]
pub(super) struct Bound<T> { pub value: T, pub inclusive: bool }
#[derive(Debug)]
struct Bounds<T> { lower: Option<Bound<T>>, upper: Option<Bound<T>> }
#[derive(Debug, PartialEq, Eq)]
pub(super) struct InvalidBounds;
impl<T: Copy + PartialOrd> Bounds<T> {
    fn new(lower: Option<Bound<T>>, upper: Option<Bound<T>>) -> Result<Self, InvalidBounds> {
        if let (Some(min), Some(max)) = (lower, upper) {
            if min.value > max.value || (min.value == max.value && (!min.inclusive || !max.inclusive)) {
                return Err(InvalidBounds);
            }
        }
        Ok(Self { lower, upper })
    }
    fn rejection(&self, value: T) -> Option<Code> {
        if self.lower.is_some_and(|bound| value < bound.value || (value == bound.value && !bound.inclusive)) { return Some(Code::TooSmall); }
        if self.upper.is_some_and(|bound| value > bound.value || (value == bound.value && !bound.inclusive)) { return Some(Code::TooLarge); }
        None
    }
}
#[derive(Debug)]
enum NumericKind { Int(Bounds<i64>), Number(Bounds<f64>) }
#[derive(Debug)]
pub(super) struct NumericPlan(NumericKind);
#[derive(Debug, PartialEq)]
pub(super) enum Numeric { Int(i64), Number(f64) }
impl NumericPlan {
    pub(super) fn int(lower: Option<Bound<i64>>, upper: Option<Bound<i64>>) -> Result<Self, InvalidBounds> {
        // Integer intervals must contain a representable value. Comparing the
        // written endpoints alone misses adjacent exclusive bounds and overflow.
        let first = match lower {
            Some(bound) if !bound.inclusive => bound.value.checked_add(1).ok_or(InvalidBounds)?,
            Some(bound) => bound.value,
            None => i64::MIN,
        };
        let last = match upper {
            Some(bound) if !bound.inclusive => bound.value.checked_sub(1).ok_or(InvalidBounds)?,
            Some(bound) => bound.value,
            None => i64::MAX,
        };
        if first > last { return Err(InvalidBounds); }
        Bounds::new(lower, upper).map(|bounds| Self(NumericKind::Int(bounds)))
    }
    pub(super) fn number(lower: Option<Bound<f64>>, upper: Option<Bound<f64>>) -> Result<Self, InvalidBounds> {
        if lower.into_iter().chain(upper).any(|bound| !bound.value.is_finite()) { return Err(InvalidBounds); }
        Bounds::new(lower, upper).map(|bounds| Self(NumericKind::Number(bounds)))
    }
    pub(super) fn validate_json(&self, value: &json::Value, path: &[Segment<'_>], errors: &mut Issues) -> Option<Numeric> {
        let (expected, result) = match &self.0 {
            NumericKind::Int(bounds) => (Kind::Integer, json::read_i64(value).map(|number| (Numeric::Int(number), bounds.rejection(number)))),
            NumericKind::Number(bounds) => (Kind::Number, json::read_f64(value).map(|number| (Numeric::Number(number), bounds.rejection(number)))),
        };
        let code = match result {
            Ok((output, None)) => return Some(output),
            Ok((_, Some(code))) => code,
            Err(error) => match error.code {
                "integer-range" => Code::IntegerRange,
                "integer-fractional" => Code::IntegerFractional,
                "number-precision" => Code::NumberPrecision,
                "expected-number" => Code::Type,
                _ => unreachable!("checked numeric extraction changed its error contract"),
            },
        };
        let observed = match value {
            json::Value::Null => Kind::Null, json::Value::Bool(_) => Kind::Boolean,
            json::Value::Number(_) => Kind::Number, json::Value::String(_) => Kind::String,
            json::Value::Array(_) => Kind::Array, json::Value::Object(_) => Kind::Object,
        };
        errors.push(path, code, expected, observed);
        None
    }
}

// Private wrappers fix the output type at plan construction, so collection
// validators never need a dynamic numeric result or a lossy numeric conversion.
pub(super) struct IntPlan(NumericPlan);
pub(super) struct NumberPlan(NumericPlan);
impl IntPlan {
    pub(super) fn new(lower: Option<Bound<i64>>, upper: Option<Bound<i64>>) -> Result<Self, InvalidBounds> {
        NumericPlan::int(lower, upper).map(Self)
    }
}
impl NumberPlan {
    pub(super) fn new(lower: Option<Bound<f64>>, upper: Option<Bound<f64>>) -> Result<Self, InvalidBounds> {
        NumericPlan::number(lower, upper).map(Self)
    }
}
impl super::scalar::Validate for IntPlan {
    type Output = i64;
    fn validate(&self, input: Option<json::Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<i64> {
        let Some(input) = input else {
            super::scalar::reject(None, Kind::Integer, path, issues);
            return None;
        };
        self.0.validate_json(&input, path, issues).map(|output| match output {
            Numeric::Int(value) => value,
            Numeric::Number(_) => unreachable!("integer plan has fixed output type"),
        })
    }
}
impl super::scalar::Validate for NumberPlan {
    type Output = f64;
    fn validate(&self, input: Option<json::Value>, path: &[Segment<'_>], issues: &mut Issues) -> Option<f64> {
        let Some(input) = input else {
            super::scalar::reject(None, Kind::Number, path, issues);
            return None;
        };
        self.0.validate_json(&input, path, issues).map(|output| match output {
            Numeric::Number(value) => value,
            Numeric::Int(_) => unreachable!("number plan has fixed output type"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn value(text: &str) -> serde_json::Value { serde_json::from_str(text).unwrap() }
    fn issues() -> Issues { Issues::new(4, 8, 128) }
    #[test]
    fn consuming_numeric_arrays_keep_exact_typed_outputs() {
        use super::super::{array::ArrayPlan, scalar::Validate};
        let ints = ArrayPlan(IntPlan::new(None, None).unwrap());
        let mut errors = issues();
        let output: Vec<i64> = ints.validate(Some(value("[9223372036854775807,-9223372036854775808]")), &[], &mut errors).unwrap();
        assert_eq!(output, [i64::MAX, i64::MIN]);
        assert!(errors.is_valid());
        let numbers = ArrayPlan(NumberPlan::new(None, None).unwrap());
        let output: Vec<f64> = numbers.validate(Some(value("[-0.0,1.5]")), &[], &mut errors).unwrap();
        assert_eq!(output[0].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(output[1], 1.5);
    }
    #[test]
    fn consuming_numeric_arrays_report_precision_and_original_paths() {
        use super::super::{array::ArrayPlan, scalar::Validate, issues::OwnedSegment};
        let plan = ArrayPlan(NumberPlan::new(None, None).unwrap());
        let mut errors = issues();
        assert!(plan.validate(Some(value("[1,9007199254740993,2]")), &[Segment::Field("values")], &mut errors).is_none());
        assert_eq!(errors.items[0].code, Code::NumberPrecision);
        assert_eq!(errors.items[0].path, [OwnedSegment::Field("values".into()), OwnedSegment::Index(1)]);
        let ints = IntPlan::new(None, None).unwrap();
        let mut errors = issues();
        assert_eq!(ints.validate(None, &[], &mut errors), None);
        assert_eq!(errors.items[0].code, Code::Missing);
        assert_eq!(errors.items[0].expected, Kind::Integer);
        assert_eq!(ints.validate(Some(value("null")), &[], &mut errors), None);
        assert_eq!(errors.items[1].code, Code::Type);
        assert_eq!(errors.items[1].observed, Kind::Null);
        let mut hidden = Issues::new(0, 0, 0);
        assert!(ints.validate(Some(value("1.5")), &[], &mut hidden).is_none());
        assert!(!hidden.is_valid());
    }
    #[test]
    fn integer_bounds_do_not_round_through_floating_point() {
        let plan = NumericPlan::int(Some(Bound { value: i64::MAX, inclusive: true }), None).unwrap();
        let mut errors = issues();
        assert_eq!(plan.validate_json(&value("9223372036854775807"), &[], &mut errors), Some(Numeric::Int(i64::MAX)));
        assert_eq!(plan.validate_json(&value("9223372036854775806"), &[], &mut errors), None);
        assert_eq!(errors.items[0].code, Code::TooSmall);
    }
    #[test]
    fn integer_fraction_and_range_failures_keep_the_checked_contract() {
        let plan = NumericPlan::int(None, None).unwrap();
        for (text, expected) in [("1.5", Code::IntegerFractional), ("18446744073709551615", Code::IntegerRange), ("true", Code::Type)] {
            let mut errors = issues();
            assert_eq!(plan.validate_json(&value(text), &[Segment::Field("count")], &mut errors), None);
            assert_eq!(errors.items[0].code, expected);
            assert!(!errors.is_valid());
        }
    }
    #[test]
    fn floating_extraction_rejects_loss_and_preserves_negative_zero() {
        let plan = NumericPlan::number(None, None).unwrap();
        let mut errors = issues();
        assert_eq!(plan.validate_json(&value("9007199254740993"), &[], &mut errors), None);
        assert_eq!(errors.items[0].code, Code::NumberPrecision);
        let mut errors = issues();
        let Some(Numeric::Number(number)) = plan.validate_json(&value("-0.0"), &[], &mut errors) else { panic!("negative zero rejected") };
        assert_eq!(number.to_bits(), (-0.0_f64).to_bits());
        assert!(errors.is_valid());
    }
    #[test]
    fn reusable_plan_respects_inclusive_and_exclusive_edges() {
        let plan = NumericPlan::int(Some(Bound { value: 2, inclusive: false }), Some(Bound { value: 4, inclusive: true })).unwrap();
        for (n, valid) in [(2, false), (3, true), (4, true), (5, false), (3, true)] {
            let mut errors = issues();
            assert_eq!(plan.validate_json(&serde_json::Value::from(n), &[], &mut errors).is_some(), valid);
            assert_eq!(errors.is_valid(), valid);
        }
    }
    #[test]
    fn invalid_plan_bounds_are_rejected_before_input_validation() {
        assert!(NumericPlan::int(Some(Bound { value: 4, inclusive: true }), Some(Bound { value: 3, inclusive: true })).is_err());
        assert!(NumericPlan::int(Some(Bound { value: 3, inclusive: false }), Some(Bound { value: 3, inclusive: true })).is_err());
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(NumericPlan::number(Some(Bound { value, inclusive: true }), None).is_err());
            assert!(NumericPlan::number(None, Some(Bound { value, inclusive: false })).is_err());
        }
    }
    #[test]
    fn exclusive_integer_bounds_require_a_representable_value() {
        let exclusive = |value| Some(Bound { value, inclusive: false });
        assert!(NumericPlan::int(exclusive(i64::MAX), None).is_err());
        assert!(NumericPlan::int(None, exclusive(i64::MIN)).is_err());
        for lower in [i64::MIN, -1, 0, 9007199254740992, i64::MAX - 1] {
            assert!(NumericPlan::int(exclusive(lower), exclusive(lower + 1)).is_err());
        }
        for lower in [i64::MIN, -2, 0, 9007199254740992, i64::MAX - 2] {
            let plan = NumericPlan::int(exclusive(lower), exclusive(lower + 2)).unwrap();
            let mut errors = issues();
            assert_eq!(plan.validate_json(&json::Value::from(lower + 1), &[], &mut errors), Some(Numeric::Int(lower + 1)));
            assert!(errors.is_valid());
            assert_eq!(plan.validate_json(&json::Value::from(lower), &[], &mut errors), None);
            assert_eq!(errors.items[0].code, Code::TooSmall);
            assert_eq!(plan.validate_json(&json::Value::from(lower + 2), &[], &mut errors), None);
            assert_eq!(errors.items[1].code, Code::TooLarge);
        }
    }
    #[test]
    fn suppressing_retained_issues_never_accepts_invalid_data() {
        let plan = NumericPlan::int(None, Some(Bound { value: 0, inclusive: true })).unwrap();
        let mut errors = Issues::new(0, 0, 0);
        assert_eq!(plan.validate_json(&value("1"), &[], &mut errors), None);
        assert!(!errors.is_valid());
        assert!(errors.items.is_empty() && errors.truncated);
    }
}
