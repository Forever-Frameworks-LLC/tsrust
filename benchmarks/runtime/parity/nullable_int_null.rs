// Independent standard Rust nullable transport with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let values=[value1,0,1,17,9001];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(4).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%5).unwrap();let value=if mode==0{None}else{Some(values[at])};
match value{None=>output.extend(["object","null","false","sentinel"].map(String::Borrowed)),Some(value)=>output.extend([String::Borrowed("number"),String::Owned(value.to_string()),String::Borrowed(if value!=0{"true"}else{"false"}),String::Borrowed("payload")])}}
    Array(output)
}
fn main(){}
