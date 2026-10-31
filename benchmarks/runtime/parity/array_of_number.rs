// Independent standard Rust construction and one-level iterator flattening.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn item(seed:i64,ordinal:i64)->f64{[-0.0,0.0,f64::NAN,f64::INFINITY,f64::NEG_INFINITY,seed as f64/4.0,-2.5][usize::try_from(ordinal.checked_mul(3).unwrap().checked_add(seed).unwrap()%7).unwrap()]}
fn function0(value0: i64, value1: i64) -> Vec<Vec<f64>> {let sizes=[0_i64,1,4,5,17];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let seed=i.checked_add(value1).unwrap();let flattened=match seed%3{0=>vec![],1=>vec![item(seed,0)],_=>vec![item(seed,0),item(seed,1),item(seed,2),item(seed,3),item(seed,4)]};output.push(flattened);}output}
fn main(){}
