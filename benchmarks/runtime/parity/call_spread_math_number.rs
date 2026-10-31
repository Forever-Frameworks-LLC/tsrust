// Independent standard Rust owned packs, borrowed calls and ordered evaluation.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn integers(base:i64,count:i64)->Vec<i64>{(0..count).map(|at|base.checked_add(at).unwrap()).collect()}
fn pack<T>(values:Vec<T>)->Vec<T>{values}
fn text(input:&str,value:i64)->std::string::String{input.repeat(usize::try_from(value%3+1).unwrap())+":"+&value.to_string()}
fn max_pair(a:f64,b:f64)->f64{if a.is_nan()||b.is_nan(){f64::NAN}else if a==0.0 && b==0.0{if a.is_sign_positive()||b.is_sign_positive(){0.0}else{-0.0}}else{a.max(b)}}
fn maximum(values:&[f64])->f64{values.iter().copied().fold(f64::NEG_INFINITY,max_pair)}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>> {let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let count=[0_i64,1,5][usize::try_from(mode).unwrap()];let pool=[0.0,-0.0,f64::from_bits(1),-2.5,0.5,f64::INFINITY,f64::NEG_INFINITY,f64::NAN];let values=(0..count).map(|at|pool[usize::try_from(base.checked_add(at).unwrap()%8).unwrap()]).collect::<Vec<_>>();let selected=maximum(&values);output.push(vec![selected]);output.push(values);}output}
fn main(){}
