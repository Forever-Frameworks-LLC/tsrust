// Independent standard Rust byte-length padding with owned output.
// Unicode filler truncation checks the same required UTF-8 boundary.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn pad(text:&str,target:i64,filler:&str,start:bool)->std::string::String{
    let target=usize::try_from(target.max(0)).unwrap();if target<=text.len()||filler.is_empty(){return text.to_owned();}
    let missing=target-text.len();let cycles=missing/filler.len();let remainder=missing%filler.len();
    let prefix=filler.get(..remainder).expect("UTF-8 filler boundary required");let mut output=std::string::String::with_capacity(target);
    if !start{output.push_str(text);}for _ in 0..cycles{output.push_str(filler);}output.push_str(prefix);if start{output.push_str(text);}output
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),"".to_owned(),"A".to_owned(),"a\u{0000}b".to_owned(),"XY".to_owned()," x ".to_owned(),value2.repeat(2),"Z".to_owned()];let extras:[i64;8]=[-10000,-1,0,1,2,5,8,17];let fillers=[""," ","ab","a\u{0000}b"];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let tag=usize::try_from(i.checked_mul(7).unwrap().checked_add(value1).unwrap()%8).unwrap();let fill_at=usize::try_from(i.checked_add(value1).unwrap()%4).unwrap();let filler=fillers[fill_at];let extra=extras[tag];let target=i64::try_from(values[at].len()).unwrap().checked_add(extra).unwrap();output.push(String::Owned(pad(&values[at],target,filler,false)));}Array(output)
}
fn main(){}
