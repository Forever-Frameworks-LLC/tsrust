// Independent handwritten Rust std/memchr reference; no compiler backend inclusion.
type String = std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {let size:usize=match value2.as_ref(){"4k"=>4096,"1m"=>1048576,_=>32};let mut bytes=vec![97u8;size];let mut other=vec![97u8;size];other[size-1]=98;let mut output=Vec::with_capacity(usize::try_from(value0.max(0)).unwrap().checked_mul(3).unwrap());for _ in 0..value0{output.extend([match bytes.cmp(&other){std::cmp::Ordering::Less=>0,std::cmp::Ordering::Equal=>1,std::cmp::Ordering::Greater=>2},size as i64,other.len() as i64]);}NumericArray(output)}
fn main(){}
