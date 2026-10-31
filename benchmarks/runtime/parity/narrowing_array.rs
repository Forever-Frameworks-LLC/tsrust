// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let value=match mode{0=>None,1=>Some(None),_=>Some(Some([value1,i]))};match value{None=>output.extend(["null",""].map(String::Borrowed)),Some(None)=>output.extend(["undefined",""].map(String::Borrowed)),Some(Some(value))=>output.extend([String::Borrowed("payload"),String::Owned(value.map(|x|x.to_string()).join("é|"))])}}
    Array(output)
}
fn main(){}
