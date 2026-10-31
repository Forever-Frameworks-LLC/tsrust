// Hand-written Rust for the matched tagged-object benchmark corpus.
// Parse owned JSON, select a tag, and move into an ordinary Rust enum.
type String = std::borrow::Cow<'static, str>;
#[derive(Debug,PartialEq)]
enum Output { Count(i64), Text(std::string::String) }
#[derive(Debug,PartialEq)]
enum Code { DiscriminatorMismatch }
#[derive(Debug,PartialEq)]
enum Kind { Union, String }
#[derive(Debug)]
struct Issue { code: Code, expected: Kind, observed: Kind, path: Vec<&'static str>, path_truncated: bool }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
fn validate(input:serde_json::Value)->Result<Output,Failure>{
    let serde_json::Value::Object(mut fields)=input else{panic!("Unexpected benchmark input")};
    if fields.len()==2 {
        match fields.get("kind").and_then(serde_json::Value::as_str){
            Some("count")=>if let Some(count)=fields.get("count").and_then(serde_json::Value::as_i64){return Ok(Output::Count(count));},
            Some("text")=>if let Some(serde_json::Value::String(text))=fields.remove("text"){return Ok(Output::Text(text));},
            _=>{}
        }
    }
    Err(Failure{issues:vec![Issue{code:Code::DiscriminatorMismatch,expected:Kind::Union,observed:Kind::String,path:vec!["kind"],path_truncated:false}],truncated:false})
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum=value1;
    for _ in 0..value0 {
        let value=validate(serde_json::from_str(value2).expect("Invalid JSON")).expect("Expected success");
        match value {
            Output::Count(count)=>{assert_eq!(count,4,"Wrong count");checksum=checksum.checked_add(count).expect("integer addition overflow");},
            _=>panic!("Wrong branch")
        }
        let error=validate(serde_json::from_str("{\"kind\":\"unknown\"}").unwrap()).expect_err("Expected rejection");
        assert!(!error.truncated && error.issues.len()==1,"Wrong issues");
        for issue in error.issues {
            assert!(issue.code==Code::DiscriminatorMismatch && issue.expected==Kind::Union
                && issue.observed==Kind::String && !issue.path_truncated && issue.path==["kind"],"Wrong issue");
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
    fn selects_both_branches_and_retains_discriminator_path(){
        assert_eq!(validate(serde_json::from_str("{\"kind\":\"count\",\"count\":4}").unwrap()).unwrap(),Output::Count(4));
        assert_eq!(validate(serde_json::from_str("{\"kind\":\"text\",\"text\":\"hello\"}").unwrap()).unwrap(),Output::Text("hello".into()));
        let failure=validate(serde_json::from_str("{\"kind\":\"unknown\"}").unwrap()).unwrap_err();
        assert_eq!(failure.issues.len(),1);assert_eq!(failure.issues[0].path,["kind"]);
        assert!(validate(serde_json::from_str("{\"kind\":\"count\",\"count\":1.5}").unwrap()).is_err());
    }
}
