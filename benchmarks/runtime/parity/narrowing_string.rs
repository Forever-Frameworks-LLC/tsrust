// Independent standard Rust conversion/nullable reference. Nested Option
// distinguishes null, undefined and payload where the corpus requires all three.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),std::string::String::new(),"é😀e\u{301}".into(),"\u{feff}\0".into(),value2.repeat(2)];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%5).unwrap();
        let value=match mode{0=>None,1=>Some(None),_=>Some(Some(values[at].clone()))};
        match value{None=>output.extend([String::Borrowed("null"),String::Borrowed("")]),Some(None)=>output.extend([String::Borrowed("undefined"),String::Borrowed("")]),Some(Some(mut value))=>{value.push('!');output.extend([String::Borrowed("payload"),String::Owned(value)]);}}
    }Array(output)
}
fn main(){}
