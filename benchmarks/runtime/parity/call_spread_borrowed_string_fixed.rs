// Independent standard Rust owned packs, borrowed calls and ordered evaluation.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn integers(base:i64,count:i64)->Vec<i64>{(0..count).map(|at|base.checked_add(at).unwrap()).collect()}
fn pack<T>(values:Vec<T>)->Vec<T>{values}
fn text(input:&str,value:i64)->std::string::String{input.repeat(usize::try_from(value%3+1).unwrap())+":"+&value.to_string()}
fn borrow_pair(left:&str,right:&str)->String{String::Owned(left.len().checked_add(right.len()).unwrap().to_string())}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let count=[0_i64,1,5][usize::try_from(mode).unwrap()];let values=["ab".repeat(usize::try_from(mode+1).unwrap()),"c".repeat(usize::try_from(base%3+1).unwrap())];let selected=borrow_pair(&values[0],&values[1]);output.push(Array(vec![selected]));output.push(Array(values.into_iter().map(String::Owned).collect()));}output}
fn main(){}
