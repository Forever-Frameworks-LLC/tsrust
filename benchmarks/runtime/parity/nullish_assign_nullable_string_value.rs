// Independent Rust nullable string assignment with explicit sentinels.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn make_left(value:i64,input:&str)->Option<Option<std::string::String>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(if value%3==0{std::string::String::new()}else{format!("{input}:{value}")}))}}
fn fallback(calls:&mut i64,value:i64,input:&str)->Option<Option<std::string::String>>{*calls=calls.checked_add(1).unwrap();if value%2==0{Some(None)}else{Some(Some(format!("{input}|fallback:{value}")))}}
fn display(value:&Option<Option<std::string::String>>)->(String,String){match value{None=>(String::Borrowed("null"),String::Borrowed("object")),Some(None)=>(String::Borrowed("undefined"),String::Borrowed("undefined")),Some(Some(value))=>(String::Owned(value.clone()),String::Borrowed("string"))}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output=Vec::new();let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;
        let mut target=make_left(value,value2.as_ref());if !matches!(target,Some(Some(_))){target=fallback(&mut calls,value,value2.as_ref());}
        let selected=&target;let delta=calls.checked_sub(before).unwrap();let(a,b)=display(selected);let(c,d)=display(&target);
        output.push(Array(vec![a,b,c,d,String::Owned(delta.to_string())]));
    }output.push(Array(vec![String::Owned(calls.to_string())]));output
}
fn main(){}
