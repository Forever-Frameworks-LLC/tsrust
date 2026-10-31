// Independent Rust lazy fallback/assignment using Option and retained call counts.
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn make_left(value:i64)->Option<Option<i64>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(value%4))}}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;
        let selected=make_left(value).flatten().unwrap_or_else(||{calls=calls.checked_add(1).unwrap();value.checked_add(100).unwrap()});
        let delta=calls.checked_sub(before).unwrap();output.push(NumericArray(vec![selected,delta]));
    }output.push(NumericArray(vec![calls]));Array(output)
}
fn main(){}
