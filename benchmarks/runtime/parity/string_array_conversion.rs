// Independent standard Rust conversion/nullable reference. Nested Option
// distinguishes null, undefined and payload where the corpus requires all three.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[vec![value2.as_ref(),""],vec!["é","😀"],vec!["a","b","c"]];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_add(value1).unwrap()%3).unwrap();output.push(String::Owned(values[at].join(",")));}
    Array(output)
}
fn main(){}
