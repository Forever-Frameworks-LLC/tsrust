// Independent Rust substring: clamped/swapped UTF-8 byte endpoints.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn substring(text:&str,start:i64,end:Option<i64>)->&str{let a=usize::try_from(start.max(0)).unwrap().min(text.len());let b=end.map(|end|usize::try_from(end.max(0)).unwrap().min(text.len())).unwrap_or(text.len());if a==b{return "";}text.get(a.min(b)..a.max(b)).expect("UTF-8 boundary required")}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),"".to_owned(),"Aé".to_owned(),"😀Z".to_owned(),"é".to_owned(),"a\u{0000}é😀".to_owned(),value2.repeat(2),"﻿中Z".to_owned()];let left:[i64;8]=[0, 0, 1, 4, 1, 2, 0, 3];let right:[i64;8]=[i64::try_from(values[0].len()).unwrap(), 0, 3, 5, 3, 4, i64::try_from(values[6].len()).unwrap(), 6];let equal:[i64;8]=[0, 0, 2, 1, 2, 3, 0, 1];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let tag=i.checked_mul(7).unwrap().checked_add(value1).unwrap()%9;let(from,to)=match tag{1=>(left[at],right[at]),2=>(equal[at],equal[at]),3=>(1000000000,1000000000),4=>(-10000,0),5=>(0,right[at]),6=>(left[at],i64::try_from(values[at].len()).unwrap()),7=>(right[at],left[at]),8=>(i64::try_from(values[at].len()).unwrap(),0),_=>(-1,1000000000)};let selected=substring(&values[at],to,Some(from));output.push(String::Owned(selected.to_owned()));}Array(output)
}
fn main(){}
