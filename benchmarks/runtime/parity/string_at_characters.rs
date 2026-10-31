// Independent Rust relative scalar access using forward/reverse chars iterators.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn at(text:&str,index:i64)->Option<char>{if index>=0{usize::try_from(index).ok().and_then(|index|text.chars().nth(index))}else{usize::try_from(index.unsigned_abs()-1).ok().and_then(|index|text.chars().rev().nth(index))}}
fn observed(value:Option<char>)->String{value.map(|value|String::Owned(value.to_string())).unwrap_or(String::Borrowed("none"))}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let text=value2.repeat(usize::try_from(mode).unwrap());output.push(Array(vec![observed(at(&text,i%7)),observed(at(&text,(-1_i64).checked_sub(mode).unwrap()))]));}output
}
fn main(){}
