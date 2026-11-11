type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let expression=regex::Regex::new("([a-z]+)=([0-9]+)").unwrap();
 let mut checksum=value1;
 for _ in 0..value0 {let value=expression.replace_all(value2.as_ref(),"$2:$1 $$").into_owned();checksum+=i64::try_from(value.len()).unwrap();}
 checksum
}
fn main(){}
