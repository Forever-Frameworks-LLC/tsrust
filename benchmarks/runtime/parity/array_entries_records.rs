// Independent standard Rust array construction, ownership and iteration.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct Entry{text:std::string::String,values:Vec<i64>}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let source=[Entry{text:value2.repeat(usize::try_from(base%3+1).unwrap()),values:vec![base,base.checked_add(1).unwrap()]},Entry{text:value2.to_string(),values:vec![base.checked_add(2).unwrap()]}];let mut pairs:Vec<_>=source.iter().cloned().enumerate().collect();pairs[0].1.values[0]=base.checked_add(9).unwrap();pairs[0].1.text=value2.to_string();output.push(Array(vec![String::Owned(pairs[0].0.to_string()),String::Owned(pairs[1].0.to_string()),String::Owned(source[0].text.clone()),String::Owned(source[0].values.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),String::Owned(pairs[0].1.text.clone()),String::Owned(pairs[0].1.values.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),String::Owned(pairs.len().to_string())]));}output}
fn main(){}
