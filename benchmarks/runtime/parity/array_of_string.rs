// Independent standard Rust construction and one-level iterator flattening.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn item(seed:i64,ordinal:i64,input:&str)->String{String::Owned(input.to_string()+":"+&ordinal.to_string())}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let seed=i.checked_add(value1).unwrap();let flattened=match seed%3{0=>vec![],1=>vec![item(seed,0,value2)],_=>vec![item(seed,0,value2),item(seed,1,value2),item(seed,2,value2),item(seed,3,value2),item(seed,4,value2)]};output.push(Array(flattened));}output}
fn main(){}
