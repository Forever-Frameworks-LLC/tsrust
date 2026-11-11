type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]
struct Range {start: i64, end: i64}
fn span(value: regex::Match<'_>)->Range {Range {start:i64::try_from(value.start()).unwrap(),end:i64::try_from(value.end()).unwrap()}}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let expression=regex::Regex::new("([a-z]+)=([0-9]+)").unwrap();
 let mut checksum=value1;
 for _ in 0..value0 {if let Some(value)=expression.find(value2.as_ref()).map(span){checksum+=value.start+value.end+1;}}
 checksum
}
fn main(){}
