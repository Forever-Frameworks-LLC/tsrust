// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let mut values=[value2.to_string(),"é".to_uppercase(),"OMEGA".to_lowercase(),"😀".to_uppercase(),"alpha".to_uppercase()];let mut sum=value1;for _ in 0..value0{values.reverse();for value in &values{let code=if value==value2.as_ref(){17}else{match value.as_str(){"É"=>23,"omega"=>31,"😀"=>41,"ALPHA"=>53,_=>0}};sum=sum.checked_mul(131).unwrap().checked_add(code).unwrap()%1000000007;}}sum}
fn main(){}
