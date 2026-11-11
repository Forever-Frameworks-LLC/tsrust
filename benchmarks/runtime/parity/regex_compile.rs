type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 
 let mut checksum=value1;
 for _ in 0..value0 {let expression=regex::Regex::new(value2.as_ref()).unwrap();checksum+=i64::from(expression.is_match("a=12"));}
 checksum
}
fn main(){}
