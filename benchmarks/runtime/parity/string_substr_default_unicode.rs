// Independent Rust substr with owned output and checked UTF-8 slices.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn substr(text:&str,start:i64,count:i64)->&str{if count<=0{return "";}let len=i64::try_from(text.len()).unwrap();let start=if start<0{len.saturating_add(start).max(0)}else{start.min(len)};let start=usize::try_from(start).unwrap();let count=usize::try_from(count).unwrap().min(text.len()-start);if count==0{return "";}text.get(start..start+count).expect("UTF-8 boundary required")}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),"".to_owned(),"é😀Z".to_owned(),"a\u{0000}é😀".to_owned(),"é".to_owned(),"﻿中Z".to_owned(),value2.repeat(2),"﷐￿􏿿".to_owned()];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let selected=values[at].as_str();output.push(String::Owned(selected.to_owned()));}Array(output)
}
fn main(){}
