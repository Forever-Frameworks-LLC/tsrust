// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let mut rows=vec![vec![i,value1],vec![value1,i.checked_add(1).unwrap()]];rows[1].push(i.checked_add(2).unwrap());rows[0][0]=i.checked_add(3).unwrap();let saved=rows.clone();rows[0][0]=0;for row in saved{for value in row{sum=sum.checked_add(value).unwrap();}}}sum}
fn main(){}
