// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),std::string::String::new(),"static".into(),"é😀e\u{301}".into(),"\u{feff}\0".into()];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(4).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%5).unwrap();let value=if mode==0{None}else{Some(values[at].clone())};
match value{None=>output.extend(["undefined","undefined","false","sentinel"].map(String::Borrowed)),Some(value)=>{let truth=if value.is_empty(){"false"}else{"true"};output.extend([String::Borrowed("string"),String::Owned(value),String::Borrowed(truth),String::Borrowed("payload")]);}}}
    Array(output)
}
fn main(){}
