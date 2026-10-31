// Independent Rust missing-only choice with a lazy counted fallback.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
enum Missing{Null,Undefined}
fn make_left(value:i64)->Missing{if value%2==0{Missing::Null}else{Missing::Undefined}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output=Vec::new();let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let left=make_left(value);let before=calls;
        let selected=match left{Missing::Null|Missing::Undefined=>{calls=calls.checked_add(1).unwrap();format!("{}|fallback:{value}",value2)}};
        output.push(Array(vec![String::Owned(selected),String::Owned(calls.checked_sub(before).unwrap().to_string())]));
    }output.push(Array(vec![String::Owned(calls.to_string())]));output
}
fn main(){}
