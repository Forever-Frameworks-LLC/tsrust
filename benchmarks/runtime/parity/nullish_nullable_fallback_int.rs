// Independent Rust nullable fallback; null and undefined remain distinct.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn make_left(value:i64)->Option<Option<i64>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(value%4))}}
fn fallback(calls:&mut i64,value:i64)->Option<Option<i64>>{*calls=calls.checked_add(1).unwrap();if value%2==0{None}else{Some(Some(value.checked_add(100).unwrap()))}}
fn display(value:&Option<Option<i64>>)->(String,String){match value{None=>(String::Borrowed("null"),String::Borrowed("object")),Some(None)=>(String::Borrowed("undefined"),String::Borrowed("undefined")),Some(Some(value))=>(String::Owned(value.to_string()),String::Borrowed("number"))}}
fn function0(value0: i64, value1: i64) -> Vec<Array<String>> {
    let mut output=Vec::new();let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;let left=make_left(value);let selected=match left{Some(Some(value))=>Some(Some(value)),_=>fallback(&mut calls,value)};
        let delta=calls.checked_sub(before).unwrap();let(a,b)=display(&selected);output.push(Array(vec![a,b,String::Owned(delta.to_string())]));
    }output.push(Array(vec![String::Owned(calls.to_string())]));output
}
fn main(){}
