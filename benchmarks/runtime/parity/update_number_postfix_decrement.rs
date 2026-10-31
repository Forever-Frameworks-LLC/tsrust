// Independent Rust update: retain expression result and updated target.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn update(mut value:f64)->(f64,f64){let previous=value;value=value-1.0;(previous,value)}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let operands:[f64;8]=[-2.0, -0.5, -0.0, 0.0, 0.5, 2.0, 9007199254740991.0, 9007199254740992.0];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let index=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let(observed,value)=update(operands[index]);output.extend([observed,value]);}output
}
fn main(){}
