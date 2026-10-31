// Independent standard Rust scalar indexing using str::chars().nth.
// Negative/out-of-range coordinates return the matched empty-string result.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn character(text:&str,index:i64)->String{usize::try_from(index).ok().and_then(|index|text.chars().nth(index)).map(|value|String::Owned(value.to_string())).unwrap_or(String::Borrowed(""))}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap();let index=(base%37).checked_sub(7).unwrap();output.push(character(value2.as_ref(),index));}Array(output)
}
fn main(){}
