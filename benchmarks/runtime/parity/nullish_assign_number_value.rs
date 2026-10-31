// Independent Rust lazy fallback/assignment using Option and retained call counts.
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn make_left(value:i64)->Option<Option<f64>>{match value%5{0=>None,1=>Some(None),_=>Some(Some(match value%8{0=>-0.0,1=>0.0,2=>f64::NAN,3=>f64::INFINITY,4=>f64::NEG_INFINITY,5=>2.5,6=>-2.5,_=>value as f64/4.0}))}}
fn function0(value0: i64, value1: i64) -> Vec<Vec<f64>> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());let mut calls=0_i64;
    for i in 0..value0{let value=i.checked_add(value1).unwrap();let before=calls;
        let mut target=make_left(value).flatten();let selected=*target.get_or_insert_with(||{calls=calls.checked_add(1).unwrap();value.checked_add(100).unwrap() as f64/4.0});
        let delta=calls.checked_sub(before).unwrap();output.push(vec![selected,target.unwrap(),delta as f64]);
    }output.push(vec![calls as f64]);output
}
fn main(){}
