type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]
struct Range {start: i64, end: i64}
fn span(value: regex::Match<'_>)->Range {Range {start:i64::try_from(value.start()).unwrap(),end:i64::try_from(value.end()).unwrap()}}
struct Captures {matched:Range,groups:Vec<Option<Range>>}
fn captured(value:regex::Captures<'_>)->Captures {Captures {matched:span(value.get_match()),groups:value.iter().skip(1).map(|group|group.map(span)).collect()}}
fn weight(value:&Captures)->i64 {value.matched.start+value.matched.end+1+value.groups.iter().flatten().map(|group|group.start+group.end+1).sum::<i64>()}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let expression=regex::Regex::new("([a-z]+)=([0-9]+)").unwrap();
 let mut checksum=value1;
 for _ in 0..value0 {let values=expression.captures_iter(value2.as_ref()).map(captured).collect::<Vec<_>>();for value in values {checksum+=weight(&value);}}
 checksum
}
fn main(){}
