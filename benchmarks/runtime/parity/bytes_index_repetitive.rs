// Independent handwritten Rust std/memchr reference; no compiler backend inclusion.
type String = std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {let size:usize=match value2.as_ref(){"adversarial-medium"=>4096,"adversarial-large"=>65536,_=>256};let mut bytes=vec![97u8;size];let mut needle=vec![97u8;size.min(512)/2];let last=needle.len()-1;needle[last]=98;let mut output=Vec::with_capacity(usize::try_from(value0.max(0)).unwrap().checked_mul(3).unwrap());for _ in 0..value0{output.extend([memchr::memmem::find(&bytes,&needle).map_or(0,|value|value as i64+1),size as i64,needle.len() as i64]);}NumericArray(output)}
fn main(){}
