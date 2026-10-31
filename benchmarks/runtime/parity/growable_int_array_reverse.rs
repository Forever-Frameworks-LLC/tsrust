// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn function0(value0: i64, value1: i64) -> i64 {let mut sum=value1;for i in 0..value0{let mut values=Vec::new();for k in 0..9{values.push(value1.checked_add(i).unwrap().checked_add(k).unwrap());}values.reverse();for value in values{sum=sum.checked_mul(131).unwrap().checked_add(value).unwrap()%1000000007;}}sum}
fn main(){}
