// Independent safe Rust Vec insertion/splicing and retained removed values.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn apply_splice<T>(values:&mut Vec<T>,start:i64,delete:Option<i64>,inserted:Vec<T>)->Vec<T>{let start=bound(start,values.len());let count=delete.map(|n|usize::try_from(n.max(0)).unwrap().min(values.len()-start)).unwrap_or(values.len()-start);values.splice(start..start+count,inserted).collect()}
fn contents(count:i64,seed:i64)->Vec<f64>{let mut values=Vec::new();for j in 0..count{values.push([-0.0,seed as f64/4.0,f64::NAN,f64::INFINITY][usize::try_from(j%4).unwrap()]);}values}
fn inserted(arity:i64,seed:i64)->Vec<f64>{match arity{0=>vec![],1=>vec![-0.0],_=>vec![-0.0,seed as f64/4.0,f64::NAN]}}
fn describe(value:&f64)->String{String::Owned(if *value==0.0{"0".to_owned()}else{value.to_string()})}
fn function0(value0: i64, value1: i64) -> Vec<f64> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(22).unwrap()).unwrap());for i in 0..value0{let count=sizes[usize::try_from(i.checked_add(value1).unwrap()%5).unwrap()];let mut values=contents(count,value1);let arity=i.checked_mul(7).unwrap().checked_add(value1).unwrap()%3;let inserted=inserted(arity,value1);drop(values.splice(0..0,inserted));let new_length=values.len();output.extend([new_length as f64,values.len() as f64]);output.extend(values);}output}
fn main(){}
