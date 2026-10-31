// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Record{tag:i64,text:std::string::String}
struct Holder{item:Option<Option<Record>>}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let holder=Holder{item:match mode{0=>None,1=>Some(None),_=>Some(Some(Record{tag:value1.checked_add(i).unwrap(),text:value2.to_string()}))}};match holder.item{None=>output.extend(["null",""].map(String::Borrowed)),Some(None)=>output.extend(["undefined",""].map(String::Borrowed)),Some(Some(mut value))=>{value.text.push('!');output.extend([String::Owned(value.tag.to_string()),String::Owned(value.text)]);}}}
    Array(output)
}
fn main(){}
