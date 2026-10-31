// Hand-written recursive Rust validator for this benchmark's JSON tree contract.
// Builds owned trees and retains ordered field/index paths for validation errors.
type String = std::borrow::Cow<'static, str>;
#[derive(Debug)]
struct Tree { value: i64, children: Vec<Tree> }
#[derive(Debug,Clone,PartialEq)]
enum Segment { Field(&'static str), Index(usize) }
#[derive(Debug,PartialEq)]
enum Code { TypeMismatch }
#[derive(Debug)]
struct Issue { code: Code, path: Vec<Segment>, path_truncated: bool }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
fn reject(path:&[Segment],issues:&mut Vec<Issue>) {
    issues.push(Issue{code:Code::TypeMismatch,path:path.to_vec(),path_truncated:false});
}
fn visit(input:serde_json::Value,path:&mut Vec<Segment>,issues:&mut Vec<Issue>)->Option<Tree> {
    let serde_json::Value::Object(mut fields)=input else {reject(path,issues);return None};
    let start=issues.len();
    let value=fields.remove("value").and_then(|value|value.as_i64());
    if value.is_none(){path.push(Segment::Field("value"));reject(path,issues);path.pop();}
    let mut children=Vec::new();
    path.push(Segment::Field("children"));
    match fields.remove("children") {
        Some(serde_json::Value::Array(items))=>{
            children.reserve(items.len());
            for (index,item) in items.into_iter().enumerate(){
                path.push(Segment::Index(index));
                if let Some(child)=visit(item,path,issues){children.push(child);}
                path.pop();
            }
        },
        _=>reject(path,issues)
    }
    path.pop();
    if !fields.is_empty(){reject(path,issues);}
    if issues.len()==start{Some(Tree{value:value.unwrap(),children})}else{None}
}
fn validate(input:serde_json::Value)->Result<Tree,Failure>{
    let mut issues=Vec::new();let mut path=Vec::new();
    match visit(input,&mut path,&mut issues){Some(tree)=>Ok(tree),None=>Err(Failure{issues,truncated:false})}
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum=value1;
    for _ in 0..value0 {
        let value=validate(serde_json::from_str(value2).expect("Invalid JSON")).expect("Expected tree success");
        let observed=value.value.checked_add(value.children[0].value).expect("integer addition overflow");
        checksum=checksum.checked_add(observed).expect("integer addition overflow");
        let error=validate(serde_json::from_str("{\"value\":1,\"children\":[{\"value\":\"bad\",\"children\":[]}]}").unwrap()).expect_err("Expected nested rejection");
        assert!(!error.truncated && error.issues.len()==1,"Invalid error count");
        for issue in error.issues {
            assert!(issue.code==Code::TypeMismatch && !issue.path_truncated
                && issue.path==[Segment::Field("children"),Segment::Index(0),Segment::Field("value")],"Invalid recursive path");
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
    fn retains_nested_error_indices_and_checks_leaf_types(){
        let error=validate(serde_json::from_str("{\"value\":1,\"children\":[{\"value\":2,\"children\":[]},{\"value\":2.5,\"children\":[]}]}").unwrap()).unwrap_err();
        assert_eq!(error.issues.len(),1);
        assert_eq!(error.issues[0].path,[Segment::Field("children"),Segment::Index(1),Segment::Field("value")]);
        assert!(validate(serde_json::from_str("{\"value\":2,\"children\":[]}").unwrap()).is_ok());
    }
}
