//! Typed literal validation retains owned successful input and redacts failures.

use super::{scalar::Validate,issues::{Code,Issues,Kind,Segment}};
use crate::stdlib::json::Value;

pub(super) trait LiteralValue: PartialEq {
    const KIND:Kind;
    const OBSERVED:Kind=Self::KIND;
    fn valid(&self)->bool{true}
}
impl LiteralValue for String {const KIND:Kind=Kind::String;}
impl LiteralValue for bool {const KIND:Kind=Kind::Boolean;}
impl LiteralValue for () {const KIND:Kind=Kind::Null;}
impl LiteralValue for i64 {const KIND:Kind=Kind::Integer;const OBSERVED:Kind=Kind::Number;}
impl LiteralValue for f64 {const KIND:Kind=Kind::Number;fn valid(&self)->bool{self.is_finite()}}

#[derive(Debug)]
pub(super) struct InvalidLiteral;
pub(super) struct LiteralPlan<P:Validate> {inner:P,expected:P::Output}
impl<P:Validate> LiteralPlan<P> where P::Output:LiteralValue {
    pub(super) fn new(inner:P,expected:P::Output)->Result<Self,InvalidLiteral>{
        if !expected.valid(){return Err(InvalidLiteral);}
        Ok(Self{inner,expected})
    }
}
impl<P:Validate> Validate for LiteralPlan<P> where P::Output:LiteralValue {
    type Output=P::Output;
    fn validate(&self,input:Option<Value>,path:&[Segment<'_>],issues:&mut Issues)->Option<Self::Output>{
        let output=self.inner.validate(input,path,issues)?;
        if output==self.expected {Some(output)}else{
            issues.push(path,Code::Literal,P::Output::KIND,P::Output::OBSERVED);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{scalar::{Validate,StringPlan,BooleanPlan,NullPlan},numeric::{IntPlan,NumberPlan},array::ArrayPlan,issues::{Issues,Code,OwnedSegment,Segment}};
    use crate::stdlib::json::Value;
    #[test]
    fn literal_string_moves_input_and_reuses_plan() {
        let plan=LiteralPlan::new(StringPlan,"hello".to_owned()).unwrap();
        for _ in 0..2 {
            let input="hello".to_owned();let pointer=input.as_ptr();
            let mut errors=Issues::new(4,8,128);
            let output=plan.validate(Some(Value::String(input)),&[],&mut errors).unwrap();
            assert_eq!(output.as_ptr(),pointer);assert!(errors.is_valid());
        }
        let mut errors=Issues::new(4,8,128);
        assert!(plan.validate(Some(Value::String("secret".into())),&[],&mut errors).is_none());
        assert_eq!(errors.items[0].code,Code::Literal);
        assert!(!format!("{errors:?}").contains("secret"));
        assert!(!format!("{errors:?}").contains("hello"));
    }
    #[test]
    fn numeric_literals_preserve_exactness_and_checked_extraction() {
        let plan=LiteralPlan::new(IntPlan::new(None,None).unwrap(),i64::MAX).unwrap();
        let mut errors=Issues::new(4,8,128);
        assert_eq!(plan.validate(Some(Value::from(i64::MAX)),&[],&mut errors),Some(i64::MAX));
        assert_eq!(plan.validate(Some(Value::from(i64::MAX-1)),&[],&mut errors),None);
        assert_eq!(errors.items[0].code,Code::Literal);
        let number=LiteralPlan::new(NumberPlan::new(None,None).unwrap(),0.0).unwrap();
        let output=number.validate(Some(serde_json::from_str("-0.0").unwrap()),&[],&mut errors).unwrap();
        assert_eq!(output.to_bits(),(-0.0_f64).to_bits());
        assert!(number.validate(Some(Value::from(9007199254740993_i64)),&[],&mut errors).is_none());
        assert_eq!(errors.items[1].code,Code::NumberPrecision);
        for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            assert!(LiteralPlan::new(NumberPlan::new(None,None).unwrap(),value).is_err());
        }
    }
    #[test]
    fn literal_mismatch_keeps_paths_and_zero_budget_failure() {
        let plan=ArrayPlan(LiteralPlan::new(BooleanPlan,false).unwrap());
        let mut errors=Issues::new(1,8,128);
        assert!(plan.validate(Some(Value::Array(vec![Value::Bool(false),Value::Bool(true)])),&[Segment::Field("flags")],&mut errors).is_none());
        assert_eq!(errors.items[0].path,[OwnedSegment::Field("flags".into()),OwnedSegment::Index(1)]);
        let mut hidden=Issues::new(0,0,0);
        assert!(plan.validate(Some(Value::Array(vec![Value::Bool(true)])),&[],&mut hidden).is_none());
        assert!(!hidden.is_valid()&&hidden.truncated);
        let null=LiteralPlan::new(NullPlan,()).unwrap();
        assert_eq!(null.validate(Some(Value::Null),&[],&mut errors),Some(()));
        let mut missing=Issues::new(1,8,128);
        assert_eq!(null.validate(None,&[],&mut missing),None);
        assert_eq!(missing.items[0].code,Code::Missing);
    }
}
