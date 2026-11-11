// Independent std filesystem operations with owned values and fixed observation rows.
use std::io;
type String = std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>([T; 2]);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
 let (n,seed,input)=(value0,value1,value2);
 let source=format!("{input}/source.bin");let target=format!("{input}/copy.bin");let moved=format!("{input}/moved.bin");let mut rows=Vec::new();
 for i in 0..n {
  let copied=std::fs::copy(&source,&target).unwrap();let length=std::fs::metadata(&target).unwrap().len();
  std::fs::rename(&target,&moved).unwrap();std::fs::remove_file(&moved).unwrap();
  rows.push(NumericArray([i64::try_from(copied).unwrap(),i64::try_from(length).unwrap()+(i+seed)%17]));
 }
 Array(rows)
}
fn main(){}
