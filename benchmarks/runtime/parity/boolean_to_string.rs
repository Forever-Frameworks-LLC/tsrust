// Independent Rust boolean value/conversion reference with owned output.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let values=[false,true,true,false];
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{
        let at=usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%4).unwrap();
        let selected=values[at];output.push(String::Owned(selected.to_string()));
    }
    Array(output)
}
fn main(){}
