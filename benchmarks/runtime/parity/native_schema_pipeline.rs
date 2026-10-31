// Hand-written Rust pipeline: owned coercion followed by borrowed checks and
// consuming transforms. Retry preserves the owned input after readonly rejection.
type String = std::borrow::Cow<'static, str>;
#[derive(Debug,PartialEq)]
enum Code { TooSmall, RefinementFailed }
#[derive(Debug)]
struct Issue { code: Code, message: &'static str, expected: &'static str, observed: &'static str, path: Vec<&'static str>, path_truncated: bool }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
fn pipeline(mut values:Vec<i64>)->Result<i64,Failure>{
    values.push(7);
    let value=i64::try_from(values.len()).expect("integer size overflow");
    let code=if value<3{Some(Code::TooSmall)}else if value>=10{Some(Code::RefinementFailed)}else{None};
    match code {
        Some(code)=>Err(Failure{issues:vec![Issue{code,message:"At least three values required",expected:"integer",observed:"integer",path:Vec::new(),path_truncated:false}],truncated:false}),
        None=>Ok(value.checked_add(1).expect("integer addition overflow"))
    }
}
fn retry(input:serde_json::Value)->i64{
    let serde_json::Value::Array(items)=input else{panic!("Expected array")};
    let values=items.into_iter().map(|item|item.as_i64().expect("Expected integer")).collect::<Vec<_>>();
    // Both branches accept the same integer representation. A failed borrowed
    // predicate leaves it owned here for the second branch's length transform.
    if values.len()>3{0}else{i64::try_from(values.len()).expect("integer size overflow")}
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let _=value2;
    let mut checksum=value1;
    for _ in 0..value0 {
        let value=pipeline(vec![2,3]).expect("Expected success");
        assert_eq!(value,4,"Wrong transformed value");
        checksum=checksum.checked_add(value).expect("integer addition overflow");
        let error=pipeline(Vec::new()).expect_err("Expected rejection");
        assert!(!error.truncated && error.issues.len()==1,"Wrong issue count");
        for issue in error.issues {
            assert!(issue.code==Code::TooSmall && issue.message=="At least three values required"
                && issue.expected=="integer" && issue.observed=="integer"
                && !issue.path_truncated && issue.path.is_empty(),"Wrong issue");
            checksum=checksum.checked_add(1).expect("integer addition overflow");
        }
        let value=retry(serde_json::from_str("[2,3]").unwrap());
        assert_eq!(value,2,"Wrong rollback");
        checksum=checksum.checked_add(value).expect("integer addition overflow");
    }
    checksum
}
fn main() {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_pipeline_bounds_and_both_retry_branches(){
        assert_eq!(pipeline(vec![2,3]).unwrap(),4);
        assert_eq!(pipeline(Vec::new()).unwrap_err().issues[0].code,Code::TooSmall);
        assert_eq!(pipeline(vec![0;9]).unwrap_err().issues[0].code,Code::RefinementFailed);
        assert_eq!(retry(serde_json::from_str("[2,3]").unwrap()),2);
        assert_eq!(retry(serde_json::from_str("[1,2,3,4]").unwrap()),0);
    }
}
