// Independent Rust primitive value retention; IEEE bits remain unmodified.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let values:[f64;8]=[0.0, -0.0, 5e-324, -2.5, 0.5, f64::INFINITY, f64::NEG_INFINITY, f64::NAN];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();output.push(values[at]);}output
}
fn main(){}
