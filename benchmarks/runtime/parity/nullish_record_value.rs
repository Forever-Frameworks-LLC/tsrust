// Independent Rust owned nullish selection. Required duplicate outputs are
// independent owned values; the last use can move its payload safely.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Record{id:i64,label:std::string::String}
fn make_left(value:i64,input:&str)->Option<Option<Record>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(Record{id:value%4,label:if value%3==0{std::string::String::new()}else{format!("{input}:{value}")}}))}}
fn fallback(calls:&mut i64,value:i64,input:&str)->Record{*calls=calls.checked_add(1).unwrap();Record{id:value.checked_add(100).unwrap(),label:format!("{input}|fallback:{value}")}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output=Vec::new();let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;
        let selected=make_left(value,value2.as_ref()).flatten().unwrap_or_else(||fallback(&mut calls,value,value2.as_ref()));
        let delta=calls.checked_sub(before).unwrap();output.push(Array(vec![String::Owned(format!("{}:{}",selected.id,selected.label)),String::Owned(delta.to_string())]));
    }output.push(Array(vec![String::Owned(calls.to_string())]));output
}
fn main(){}
