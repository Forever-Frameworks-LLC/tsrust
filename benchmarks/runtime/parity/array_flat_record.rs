// Independent standard Rust construction and one-level iterator flattening.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Record{id:i64,label:std::string::String}
fn item(seed:i64,ordinal:i64,input:&str)->Record{Record{id:seed.checked_add(ordinal).unwrap(),label:input.to_string()+":"+&ordinal.to_string()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let count=sizes[usize::try_from(i.checked_add(value1).unwrap()%5).unwrap()];let mut source=Vec::new();for j in 0_i64..count{let width=j.checked_add(value1).unwrap()%3;let mut part=Vec::new();for k in 0..width{let ordinal=j.checked_mul(3).unwrap().checked_add(k).unwrap();part.push(item(value1,ordinal,value2));}source.push(part);}let flattened:Vec<_>=source.into_iter().flatten().collect();output.push(Array(flattened.into_iter().map(|value|String::Owned(value.id.to_string()+":"+&value.label)).collect()));}output}
fn main(){}
