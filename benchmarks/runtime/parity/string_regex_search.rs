type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let expression=regex::Regex::new("([a-z]+)=([0-9]+)").unwrap();
 let mut checksum=value1;
 for _ in 0..value0 {checksum+=expression.find(value2.as_ref()).map_or(-1,|value|i64::try_from(value.start()).unwrap())+1;}
 checksum
}
fn main(){}
