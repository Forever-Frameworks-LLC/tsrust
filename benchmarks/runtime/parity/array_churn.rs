// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let width=i.checked_add(value1).unwrap()%65;let mut values=Vec::new();for j in 0..width{values.push(i.checked_add(value1).unwrap().checked_add(j).unwrap()%1009);}let mut saved=values.clone();if !values.is_empty(){values[0]=i;}let mut removed=0_i64;while saved.len()>usize::try_from(width/2).unwrap(){let index=usize::try_from(i.checked_add(value1).unwrap().checked_add(removed).unwrap()%i64::try_from(saved.len()).unwrap()).unwrap();sum=sum.checked_add(saved.remove(index)).unwrap();removed=removed.checked_add(1).unwrap();}saved.push(value1);saved.push(i);for (at,value) in saved.into_iter().enumerate(){let position=i64::try_from(at).unwrap().checked_add(1).unwrap();sum=sum.checked_add(value.checked_mul(position).unwrap()).unwrap();}for value in values{sum=sum.checked_add(value).unwrap();}}sum}
fn main(){}
