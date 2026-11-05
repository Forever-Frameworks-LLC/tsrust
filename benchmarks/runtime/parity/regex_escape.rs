type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 
 let mut checksum=value1;
 for _ in 0..value0 {let value=regex::escape(value2.as_ref());checksum+=i64::try_from(value.len()).unwrap();}
 checksum
}
fn main(){}
