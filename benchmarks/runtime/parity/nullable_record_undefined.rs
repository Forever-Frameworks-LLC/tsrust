// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Record{tag:i64,text:std::string::String}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(4).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let value=if mode==0{None}else{Some(Record{tag:value1.checked_add(i).unwrap(),text:value2.to_string()})};match value{None=>output.extend(["undefined","undefined","false","sentinel"].map(String::Borrowed)),Some(_value)=>output.extend(["object","[object Object]","true","payload"].map(String::Borrowed))}}
    Array(output)
}
fn main(){}
