// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn integers(base:i64,count:i64)->Vec<i64>{(0..count).map(|at|base.checked_add(at).unwrap()).collect()}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let count=[0,1,5][usize::try_from(mode).unwrap()];let values=integers(base,count);let selected=(0..values.len()).map(|index|i64::try_from(index).unwrap()).collect();output.push(NumericArray(selected));}Array(output)}
fn main(){}
