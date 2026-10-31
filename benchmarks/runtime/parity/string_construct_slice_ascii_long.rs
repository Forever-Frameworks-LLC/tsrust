// Independent Rust checked byte slice; every retained result is owned.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let width=i64::try_from(value2.len()).unwrap();let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let start=usize::try_from(i.checked_add(value1).unwrap()%8).unwrap();let end=usize::try_from(width.checked_sub(1).unwrap().checked_sub(i.checked_mul(7).unwrap().checked_add(value1).unwrap()%8).unwrap()).unwrap();output.push(String::Owned(value2.get(start..end).expect("UTF-8 boundary required").to_owned()));}Array(output)
}
fn main(){}
