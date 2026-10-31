// Independent standard Rust conversion/nullable reference. Nested Option
// distinguishes null, undefined and payload where the corpus requires all three.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let values=[true,false];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(4).unwrap()).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%2).unwrap();
        let value=if mode==0{None}else{Some(values[at])};
        let labels=match value{None=>["undefined","undefined","false","sentinel"],Some(true)=>["boolean","true","true","payload"],Some(false)=>["boolean","false","false","payload"]};
        output.extend(labels.into_iter().map(String::Borrowed));
    }Array(output)
}
fn main(){}
