// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn char_at(text:&str,index:i64)->std::string::String{text.chars().nth(usize::try_from(index).unwrap()).map(|c|c.to_string()).unwrap_or_default()}
fn function0(value0: i64, value1: i64, value2: &String, value3: i64) -> i64 {let width=value3;let mut values=Vec::new();for j in 0_i64..129{values.push(char_at(value2,j.checked_mul(37).unwrap().checked_add(value1).unwrap()%width));}let mut sum=value1;for i in 0..value0{let from=(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%387).checked_sub(193).unwrap();let start=bound(from,values.len());let needle=match i.checked_add(value1).unwrap()%4{0=>"A".to_owned(),1=>"😀".to_owned(),2=>char_at(value2,i.checked_mul(17).unwrap().checked_add(value1).unwrap()%width.checked_add(2).unwrap()),_=>"missing".to_owned()};let result=values[start..].iter().position(|value|value==&needle).map(|at|i64::try_from(start.checked_add(at).unwrap()).unwrap()).unwrap_or(-1);sum=sum.checked_add(result).unwrap();}sum}
fn main(){}
