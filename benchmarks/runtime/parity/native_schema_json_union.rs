// Hand-written Rust choice validator: borrow while testing branches, then move
// the selected owned string. Construct diagnostics only if every branch rejects.
type String = std::borrow::Cow<'static, str>;
#[derive(Debug,PartialEq,Clone,Copy)]
enum Code { TypeMismatch, TooSmall }
#[derive(Debug,PartialEq,Clone,Copy)]
enum Kind { String, Boolean, Other }
#[derive(Debug)]
struct Issue { code: Code, expected: Kind, observed: Kind, path: Vec<std::string::String>, path_truncated: bool }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
fn validate(input:serde_json::Value)->Result<std::string::String,Failure>{
    let mut codes=[Code::TypeMismatch;2];
    for (index,minimum) in [20,2].into_iter().enumerate(){
        if let serde_json::Value::String(text)=&input {
            if text.chars().count()>=minimum {
                let serde_json::Value::String(text)=input else{unreachable!()};
                return Ok(text);
            }
            codes[index]=Code::TooSmall;
        }
    }
    let observed=match input{serde_json::Value::String(_)=>Kind::String,serde_json::Value::Bool(_)=>Kind::Boolean,_=>Kind::Other};
    Err(Failure{issues:codes.into_iter().map(|code|Issue{code,expected:Kind::String,observed,path:Vec::new(),path_truncated:false}).collect(),truncated:false})
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum=value1;
    for _ in 0..value0 {
        let value=validate(serde_json::from_str(value2).expect("Invalid JSON")).expect("Expected success");
        assert_eq!(value,"union-value","Wrong value");
        checksum=checksum.checked_add(i64::try_from(value.len()).unwrap()).expect("integer addition overflow");
        let error=validate(serde_json::from_str("true").unwrap()).expect_err("Expected rejection");
        assert!(!error.truncated && error.issues.len()==2,"Wrong issues");
        for issue in error.issues {
            assert!(issue.code==Code::TypeMismatch && issue.expected==Kind::String
                && issue.observed==Kind::Boolean && issue.path.is_empty()
                && !issue.path_truncated,"Wrong issue");
            checksum=checksum.checked_add(1).expect("integer addition overflow");
        }
    }
    checksum
}
fn main() {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_later_branch_and_counts_unicode_scalars(){
        assert_eq!(validate(serde_json::Value::String("éé".into())).unwrap(),"éé");
        let error=validate(serde_json::Value::String("é".into())).unwrap_err();
        assert_eq!(error.issues.len(),2);
        assert!(error.issues.iter().all(|issue|issue.code==Code::TooSmall));
        let error=validate(serde_json::Value::Bool(true)).unwrap_err();
        assert!(error.issues.iter().all(|issue|issue.code==Code::TypeMismatch));
    }
}
