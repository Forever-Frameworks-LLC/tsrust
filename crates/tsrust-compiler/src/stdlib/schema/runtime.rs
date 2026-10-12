//! Scalar schema plan storage shared by generated programs and backend tests.

#[derive(Debug, PartialEq, Eq)]
pub struct ConstructionError<T = i64> { pub code: &'static str, pub operation: &'static str, pub limit: T }
#[derive(Debug, Default)]
pub struct StringBounds { min: Option<u64>, max: Option<u64> }
impl StringBounds {
    pub fn min(mut self, limit: i64) -> Result<Self, ConstructionError> {
        if limit < 0 { return Err(ConstructionError { code: "negative-length", operation: "min", limit }); }
        let next=self.min.unwrap_or(0).max(limit as u64);
        if self.max.is_some_and(|max|next>max) { return Err(ConstructionError { code: "contradictory-bounds", operation: "min", limit }); }
        self.min=Some(next);Ok(self)
    }
    pub fn max(mut self, limit: i64) -> Result<Self, ConstructionError> {
        if limit < 0 { return Err(ConstructionError { code: "negative-length", operation: "max", limit }); }
        let next=self.max.unwrap_or(u64::MAX).min(limit as u64);
        if self.min.is_some_and(|min|min>next) { return Err(ConstructionError { code: "contradictory-bounds", operation: "max", limit }); }
        self.max=Some(next);Ok(self)
    }
    pub fn check(&self, text: &str) -> Option<&'static str> {
        if self.min.is_none() && self.max.is_none() { return None; }
        let mut length=0u64;
        for _ in text.chars() {
            if self.max==Some(length) { return Some("too-large"); }
            length+=1;
        }
        if self.min.is_some_and(|min|length<min) { return Some("too-small"); }
        None
    }
}

#[derive(Debug, Default)]
pub struct IntBounds { min: Option<i64>, max: Option<i64> }
impl IntBounds {
    pub fn gt(self,limit:i64)->Result<Self,ConstructionError>{
        let error=||ConstructionError{code:"contradictory-bounds",operation:"gt",limit};
        let next=limit.checked_add(1).ok_or_else(error)?;
        self.min(next).map_err(|_|error())
    }
    pub fn lt(self,limit:i64)->Result<Self,ConstructionError>{
        let error=||ConstructionError{code:"contradictory-bounds",operation:"lt",limit};
        let next=limit.checked_sub(1).ok_or_else(error)?;
        self.max(next).map_err(|_|error())
    }
    pub fn min(mut self,limit:i64)->Result<Self,ConstructionError>{
        let next=self.min.unwrap_or(i64::MIN).max(limit);
        if self.max.is_some_and(|max|next>max){return Err(ConstructionError{code:"contradictory-bounds",operation:"min",limit});}
        self.min=Some(next);Ok(self)
    }
    pub fn max(mut self,limit:i64)->Result<Self,ConstructionError>{
        let next=self.max.unwrap_or(i64::MAX).min(limit);
        if self.min.is_some_and(|min|min>next){return Err(ConstructionError{code:"contradictory-bounds",operation:"max",limit});}
        self.max=Some(next);Ok(self)
    }
    pub fn check(&self,value:i64)->Option<&'static str>{
        if self.min.is_some_and(|min|value<min){Some("too-small")}
        else if self.max.is_some_and(|max|value>max){Some("too-large")}else{None}
    }
}

#[derive(Debug, Default)]
pub struct NumberBounds { min: Option<f64>, max: Option<f64> }
impl NumberBounds {
    // Every finite exclusive endpoint has a next representable inclusive value,
    // except the largest/smallest finite value. No epsilon or decimal rounding.
    fn bound(mut self, limit:f64, operation:&'static str, lower:bool, exclusive:bool)->Result<Self,ConstructionError<f64>> {
        let error=|code|ConstructionError{code,operation,limit};
        if !limit.is_finite(){return Err(error("nonfinite-limit"));}
        let endpoint=if !exclusive {limit}else if lower {limit.next_up()}else{limit.next_down()};
        if !endpoint.is_finite(){return Err(error("contradictory-bounds"));}
        if lower {
            let next=self.min.unwrap_or(-f64::MAX).max(endpoint);
            if self.max.is_some_and(|max|next>max){return Err(error("contradictory-bounds"));}
            self.min=Some(next);
        }else{
            let next=self.max.unwrap_or(f64::MAX).min(endpoint);
            if self.min.is_some_and(|min|min>next){return Err(error("contradictory-bounds"));}
            self.max=Some(next);
        }
        Ok(self)
    }
    pub fn min(self,limit:f64)->Result<Self,ConstructionError<f64>>{self.bound(limit,"min",true,false)}
    pub fn max(self,limit:f64)->Result<Self,ConstructionError<f64>>{self.bound(limit,"max",false,false)}
    pub fn gt(self,limit:f64)->Result<Self,ConstructionError<f64>>{self.bound(limit,"gt",true,true)}
    pub fn lt(self,limit:f64)->Result<Self,ConstructionError<f64>>{self.bound(limit,"lt",false,true)}
    pub fn check(&self,value:f64)->Option<&'static str>{
        if !value.is_finite(){Some("number-nonfinite")}
        else if self.min.is_some_and(|min|value<min){Some("too-small")}
        else if self.max.is_some_and(|max|value>max){Some("too-large")}else{None}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn number_bounds_preserve_exclusive_edges_and_negative_zero() {
        let plan=NumberBounds::default().gt(1.0).unwrap().lt(2.0).unwrap().min(0.0).unwrap().max(3.0).unwrap();
        assert_eq!(plan.check(1.0),Some("too-small"));
        assert_eq!(plan.check(1.0_f64.next_up()),None);
        assert_eq!(plan.check(1.5),None);
        assert_eq!(plan.check(2.0),Some("too-large"));
        let zero=NumberBounds::default().min(-0.0).unwrap().max(0.0).unwrap();
        assert_eq!(zero.check(-0.0),None);assert_eq!(zero.check(0.0),None);
        assert_eq!(NumberBounds::default().gt(-0.0).unwrap().check(0.0),Some("too-small"));
        assert_eq!(NumberBounds::default().lt(0.0).unwrap().check(-0.0),Some("too-large"));
    }
    #[test]
    fn number_bounds_reject_empty_and_nonfinite_construction() {
        for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            for (operation,error) in [
                ("min",NumberBounds::default().min(value).unwrap_err()),
                ("max",NumberBounds::default().max(value).unwrap_err()),
                ("gt",NumberBounds::default().gt(value).unwrap_err()),
                ("lt",NumberBounds::default().lt(value).unwrap_err()),
            ] {assert_eq!(error.code,"nonfinite-limit");assert_eq!(error.operation,operation);assert_eq!(error.limit.to_bits(),value.to_bits());}
            assert_eq!(NumberBounds::default().check(value),Some("number-nonfinite"));
        }
        assert!(NumberBounds::default().gt(f64::MAX).is_err());
        assert!(NumberBounds::default().lt(-f64::MAX).is_err());
        for lower in [-1.0,0.0,1.0,9007199254740992.0] {
            assert!(NumberBounds::default().gt(lower).unwrap().lt(lower.next_up()).is_err());
            let plan=NumberBounds::default().gt(lower).unwrap().max(lower.next_up()).unwrap();
            assert_eq!(plan.check(lower.next_up()),None);
        }
        let error=NumberBounds::default().min(2.0).unwrap().lt(2.0).unwrap_err();
        assert_eq!((error.code,error.operation,error.limit),("contradictory-bounds","lt",2.0));
    }
    #[test]
    fn exclusive_integer_runtime_bounds_intersect_without_overflow() {
        let plan=IntBounds::default().gt(2).unwrap().lt(5).unwrap().min(1).unwrap().max(9).unwrap();
        for (input,expected) in [(2,Some("too-small")),(3,None),(4,None),(5,Some("too-large"))] {
            assert_eq!(plan.check(input),expected);
        }
        let high=IntBounds::default().gt(i64::MAX-1).unwrap();
        assert_eq!(high.check(i64::MAX),None);
        assert_eq!(high.check(i64::MAX-1),Some("too-small"));
        let low=IntBounds::default().lt(i64::MIN+1).unwrap();
        assert_eq!(low.check(i64::MIN),None);
        assert_eq!(low.check(i64::MIN+1),Some("too-large"));
        for (error,operation,limit) in [
            (IntBounds::default().gt(i64::MAX).unwrap_err(),"gt",i64::MAX),
            (IntBounds::default().lt(i64::MIN).unwrap_err(),"lt",i64::MIN),
            (IntBounds::default().gt(2).unwrap().lt(3).unwrap_err(),"lt",3),
            (IntBounds::default().lt(3).unwrap().gt(2).unwrap_err(),"gt",2),
        ] {
            assert_eq!((error.code,error.operation,error.limit),("contradictory-bounds",operation,limit));
        }
    }
    #[test]
    fn consuming_bounds_reject_invalid_construction_and_intersect() {
        let plan=StringBounds::default().min(2).unwrap().max(4).unwrap().min(1).unwrap().max(9).unwrap();
        assert_eq!(plan.check("a"),Some("too-small"));
        assert_eq!(plan.check("abcd"),None);
        assert_eq!(plan.check("abcde"),Some("too-large"));
        let negative=StringBounds::default().min(-1).unwrap_err();
        assert_eq!((negative.code,negative.operation,negative.limit),("negative-length","min",-1));
        let reversed=StringBounds::default().min(3).unwrap().max(2).unwrap_err();
        assert_eq!((reversed.code,reversed.operation,reversed.limit),("contradictory-bounds","max",2));
    }
    #[test]
    fn runtime_unicode_lengths_are_code_points() {
        let plan=StringBounds::default().min(1).unwrap().max(1).unwrap();
        for text in ["a","é","😀"] {assert_eq!(plan.check(text),None);}
        assert_eq!(plan.check("e\u{301}"),Some("too-large"));
        assert_eq!(plan.check(""),Some("too-small"));
        assert_eq!(StringBounds::default().max(0).unwrap().check(""),None);
        assert_eq!(StringBounds::default().min(i64::MAX).unwrap().check("short"),Some("too-small"));
    }
}
