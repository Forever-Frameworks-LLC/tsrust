// Independent Rust owned nullish selection. Required duplicate outputs are
// independent owned values; the last use can move its payload safely.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn make_left(value:i64)->Option<Option<Vec<i64>>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(if value%3==0{Vec::new()}else{vec![value,value.checked_add(1).unwrap()]}))}}
fn fallback(calls:&mut i64,value:i64)->Vec<i64>{*calls=calls.checked_add(1).unwrap();vec![value.checked_add(100).unwrap(),value.checked_add(101).unwrap(),value.checked_add(102).unwrap()]}
fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut output=Vec::new();let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;
        let selected=make_left(value).flatten().unwrap_or_else(||fallback(&mut calls,value));
        let delta=calls.checked_sub(before).unwrap();output.push(NumericArray(selected));output.push(NumericArray(vec![delta]));
    }output.push(NumericArray(vec![calls]));Array(output)
}
fn main(){}
