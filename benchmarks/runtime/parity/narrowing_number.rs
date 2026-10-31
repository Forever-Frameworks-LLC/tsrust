// Independent standard Rust conversion/nullable reference. Nested Option
// distinguishes null, undefined and payload where the corpus requires all three.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let values=[value1 as f64/4.0,-0.0,0.5,-2.5,17.0];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%5).unwrap();
        let value=match mode{0=>None,1=>Some(None),_=>Some(Some(values[at]))};
        output.extend(match value{None=>[0.0,0.0],Some(None)=>[1.0,0.0],Some(Some(value))=>[2.0,value]});
    }output
}
fn main(){}
