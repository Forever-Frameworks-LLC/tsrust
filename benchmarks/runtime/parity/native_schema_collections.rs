// Hand-written Rust collection checks using owned strings and standard HashMap/
// HashSet defaults. Key/value transforms consume inputs; collisions retain index.
use std::collections::{HashMap,HashSet};
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
#[derive(Debug,PartialEq)]
enum Code { CollectionCollision }
#[derive(Debug)]
struct Issue { code: Code, path: Vec<usize> }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }
fn collision(index:usize)->Failure {
    Failure{issues:vec![Issue{code:Code::CollectionCollision,path:vec![index]}],truncated:false}
}
fn validate_map(input:HashMap<Text,i64>,lower:bool)->Result<HashMap<Text,i64>,Failure>{
    let mut output=HashMap::with_capacity(input.len());
    for (index,(key,value)) in input.into_iter().enumerate(){
        let key=if lower{key.to_lowercase()}else{key};
        if output.insert(key,value).is_some(){return Err(collision(index));}
    }
    Ok(output)
}
fn validate_set(input:HashSet<Text>,lower:bool)->Result<HashSet<Text>,Failure>{
    let mut output=HashSet::with_capacity(input.len());
    for (index,value) in input.into_iter().enumerate(){
        let value=if lower{value.to_lowercase()}else{value};
        if !output.insert(value){return Err(collision(index));}
    }
    Ok(output)
}
fn observe_collision(error:Failure){
    assert!(!error.truncated && error.issues.len()==1 && error.issues[0].code==Code::CollectionCollision,"Wrong collision");
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum=value1;
    for _ in 0..value0 {
        let record:HashMap<Text,i64>=serde_json::from_str(value2).expect("Record rejected");
        assert!(record.len()==1 && record.get("x")==Some(&2),"Wrong record");
        checksum=checksum.checked_add(2).expect("integer addition overflow");
        let entries=HashMap::from([("x".to_owned(),3)]);
        let value=validate_map(entries,false).expect("Map rejected");
        assert!(value.len()==1 && value.get("x")==Some(&3),"Wrong map");
        checksum=checksum.checked_add(3).expect("integer addition overflow");
        let elements=HashSet::from(["ready".to_owned()]);
        let value=validate_set(elements,false).expect("Set rejected");
        assert!(value.len()==1 && value.contains("ready"),"Wrong set");
        checksum=checksum.checked_add(1).expect("integer addition overflow");
        let keys=HashMap::from([("A".to_owned(),1),("a".to_owned(),2)]);
        observe_collision(validate_map(keys,true).expect_err("Map collision accepted"));
        checksum=checksum.checked_add(1).expect("integer addition overflow");
        let tags=HashSet::from(["A".to_owned(),"a".to_owned()]);
        observe_collision(validate_set(tags,true).expect_err("Set collision accepted"));
        checksum=checksum.checked_add(1).expect("integer addition overflow");
    }
    checksum
}
fn main() {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_transformed_collisions_and_preserves_distinct_values(){
        let error=validate_map(HashMap::from([("A".into(),1),("a".into(),2)]),true).unwrap_err();
        assert_eq!(error.issues[0].path,[1]);
        let error=validate_set(HashSet::from(["A".into(),"a".into()]),true).unwrap_err();
        assert_eq!(error.issues[0].path,[1]);
        let output=validate_map(HashMap::from([("A".into(),1),("B".into(),2)]),true).unwrap();
        assert_eq!(output.get("a"),Some(&1));assert_eq!(output.get("b"),Some(&2));
    }
}
