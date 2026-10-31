// Independent standard Rust owned packs, borrowed calls and ordered evaluation.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn integers(base:i64,count:i64)->Vec<i64>{(0..count).map(|at|base.checked_add(at).unwrap()).collect()}
fn pack<T>(values:Vec<T>)->Vec<T>{values}
fn text(input:&str,value:i64)->std::string::String{input.repeat(usize::try_from(value%3+1).unwrap())+":"+&value.to_string()}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let base=i.checked_add(value1).unwrap()%97;let actions:Vec<Box<dyn Fn()->i64>>=vec![Box::new(move||base),Box::new(move||base.checked_add(1).unwrap())];let mut packed=actions;packed.push(Box::new(move||base.checked_add(2).unwrap()));output.push(NumericArray(vec![packed[0](),packed[1](),packed[2]()]));}Array(output)}
fn main(){}
