// Independent owned Rust argument vectors and retained packed results.
type String=std::borrow::Cow<'static,str>;
fn item(base:i64,ordinal:i64)->f64{let pool=[0.0,-0.0,f64::from_bits(1),-2.5,0.5,f64::INFINITY,f64::NEG_INFINITY,f64::NAN];pool[usize::try_from(base.checked_add(ordinal).unwrap()%8).unwrap()]}
fn pack(values:Vec<f64>)->Vec<f64>{values}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>>{let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let values=match mode{0=>pack(vec![]),1=>pack(vec![item(base,0)]),_=>pack(vec![item(base,0),item(base,1),item(base,2),item(base,3),item(base,4)])};output.push(values);}output}
fn main(){}
