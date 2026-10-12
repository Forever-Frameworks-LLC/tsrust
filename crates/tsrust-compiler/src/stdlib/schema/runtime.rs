//! Scalar schema plan storage shared by generated programs and backend tests.

#[derive(Debug, PartialEq, Eq)]
pub struct ConstructionError { pub code: &'static str, pub operation: &'static str, pub limit: i64 }
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

#[cfg(test)]
mod tests {
    use super::*;
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
