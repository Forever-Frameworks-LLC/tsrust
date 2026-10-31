// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let mut values=Vec::new();for j in 0..32{values.push(value1.checked_add(i).unwrap().checked_add(j).unwrap());}values[3]=i;let saved=values.clone();for value in saved{if value==value1{continue;}if value>20000{break;}sum=sum.checked_add(value).unwrap();}let mut flags=[true,false];flags[1]=i%2==0;if flags[1]{sum=sum.checked_add(i64::try_from(flags.len()).unwrap()).unwrap().checked_sub(1).unwrap();}}sum}
fn main(){}
