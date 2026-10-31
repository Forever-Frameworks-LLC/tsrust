// Independent standard Rust trim/clone/concatenation reference.
// Data tables are the matched ASCII/Unicode workload corpus; outputs are owned.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),"".to_owned(),"A".to_owned(),"a\u{0000}b".to_owned(),"XY".to_owned()," x ".to_owned(),value2.repeat(2),"Z".to_owned()];
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();
        let selected=values[at].clone();
        output.push(String::Owned(selected));
    }Array(output)
}
fn main(){}
