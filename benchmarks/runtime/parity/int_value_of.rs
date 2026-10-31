// Independent Rust primitive value retention; IEEE bits remain unmodified.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let values:[i64;8]=[0, 1, 7, 31, 1024, 65535, 1048576, 2147483647];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();output.push(values[at]);}NumericArray(output)
}
fn main(){}
