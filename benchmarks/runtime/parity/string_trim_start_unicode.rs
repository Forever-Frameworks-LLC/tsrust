// Independent standard Rust trim/clone/concatenation reference.
// Data tables are the matched ASCII/Unicode workload corpus; outputs are owned.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),value2.repeat(2),value2.repeat(3),value2.repeat(1),value2.to_string(),"".to_owned(),"         　".to_owned(),"     literal é😀é    　".to_owned()];
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();
        let selected=values[at].trim_start().to_owned();
        output.push(String::Owned(selected));
    }Array(output)
}
fn main(){}
