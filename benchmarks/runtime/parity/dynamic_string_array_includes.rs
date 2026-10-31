// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let mut values=Vec::new();for j in 0_i64..129{values.push(if (j.checked_mul(37).unwrap().checked_add(value1).unwrap()%101)<100{value2.trim().to_lowercase()}else{value2.trim().to_owned()});}let mut sum=value1;for i in 0..value0{let from=(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%387).checked_sub(193).unwrap();let start=bound(from,values.len());let needle=match i.checked_add(value1).unwrap()%3{0=>"label å",1=>"Label Å",_=>"label ö"};let result=i64::from(values[start..].iter().any(|value|value==needle));sum=sum.checked_add(result).unwrap();}sum}
fn main(){}
