// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let values=[String::Owned(value2.to_string()),String::Borrowed("Pat"),String::Owned(value2.to_string()),String::Borrowed("a\0😀")];let at=usize::try_from(i.checked_mul(3).unwrap().checked_add(value1).unwrap()%4).unwrap();let selected=values.into_iter().nth(at).unwrap();output.push(selected);}Array(output)}
fn main(){}
