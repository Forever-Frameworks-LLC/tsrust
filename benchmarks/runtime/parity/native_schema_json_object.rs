// Hand-written Rust: owned object decoding, checked count, and detailed field errors.
type String = std::borrow::Cow<'static, str>;
#[derive(Debug,PartialEq)]
enum Code { Missing, Unknown, Type, TooSmall }
#[derive(Debug,PartialEq)]
enum Kind { Integer, Missing, Boolean, Number, Other }
#[derive(Debug)]
struct Issue { code: Code, expected: Kind, observed: Kind, path: Vec<std::string::String> }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
#[derive(Debug)]
struct Document { count: i64 }
fn kind(value:&serde_json::Value)->Kind {
    match value { serde_json::Value::Bool(_)=>Kind::Boolean, serde_json::Value::Number(_)=>Kind::Number,_=>Kind::Other }
}
fn validate(input:serde_json::Value)->Result<Document,Failure> {
    let serde_json::Value::Object(mut fields)=input else { panic!("Expected object input") };
    let mut issues=Vec::new();
    let count=match fields.remove("count") {
        None=>{issues.push(Issue{code:Code::Missing,expected:Kind::Integer,observed:Kind::Missing,path:vec!["count".into()]});None},
        Some(value)=>match value.as_i64() {
            Some(count) if count>=2=>Some(count),
            count=>{issues.push(Issue{code:if count.is_some(){Code::TooSmall}else{Code::Type},expected:Kind::Integer,observed:kind(&value),path:vec!["count".into()]});None}
        }
    };
    for (key,value) in fields { issues.push(Issue{code:Code::Unknown,expected:Kind::Missing,observed:kind(&value),path:vec![key]}); }
    if issues.is_empty(){Ok(Document{count:count.unwrap()})}else{Err(Failure{issues,truncated:false})}
}
fn decode(text:&str)->Result<Document,Failure> {
    validate(serde_json::from_str(text).expect("Invalid JSON"))
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum=value1;
    for _ in 0..value0 {
        let value=decode(value2).expect("Expected success");
        checksum=checksum.checked_add(value.count).expect("integer addition overflow");
        let error=match decode("{\"extra\":true}"){Err(error)=>error,Ok(_)=>panic!("Expected rejection")};
        assert!(!error.truncated && error.issues.len()==2,"Invalid count");
        for issue in error.issues {
            assert_eq!(issue.path.len(),1,"Invalid path");
            match issue.code {
                Code::Missing=>assert!(issue.path[0]=="count" && issue.expected==Kind::Integer && issue.observed==Kind::Missing,"Invalid missing field"),
                Code::Unknown=>assert!(issue.path[0]=="extra" && issue.expected==Kind::Missing && issue.observed==Kind::Boolean,"Invalid unknown field"),
                _=>panic!("Unexpected issue")
            }
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
    fn validates_boundaries_and_retains_both_field_errors() {
        assert_eq!(decode("{\"count\":2}").unwrap().count,2);
        for text in ["{\"count\":1}","{\"count\":2.5}","{\"count\":true}"] { assert!(decode(text).is_err()); }
        let failure=decode("{\"extra\":true}").err().unwrap();
        assert_eq!(failure.issues.len(),2);
        assert_eq!(failure.issues[0].code,Code::Missing);
        assert_eq!(failure.issues[1].code,Code::Unknown);
    }
}
