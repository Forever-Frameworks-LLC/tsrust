// Independent standard Rust trim/clone/concatenation reference.
// Data tables are the matched ASCII/Unicode workload corpus; outputs are owned.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values=[value2.to_string(),"".to_owned(),"é".to_owned(),"😀".to_owned(),"é".to_owned(),"a\u{0000}é😀".to_owned(),value2.repeat(2),"Z".to_owned()];let parts=["".to_owned(),"é".to_owned(),"😀".to_owned(),"é".to_owned(),"\u{0000}".to_owned(),value2.to_string(),"中".to_owned(),"Z".to_owned()];
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();
        let arg_at=usize::try_from((i.checked_mul(7).unwrap().checked_add(value1.checked_mul(3).unwrap()).unwrap())%8).unwrap();
        let arity=usize::try_from(i.checked_add(value1).unwrap()%4+1).unwrap();
        let mut capacity=values[at].len();for j in 0..arity{capacity=capacity.checked_add(parts[(arg_at+j)%8].len()).unwrap();}
        let mut selected=std::string::String::with_capacity(capacity);selected.push_str(&values[at]);
        for j in 0..arity{selected.push_str(&parts[(arg_at+j)%8]);}
        output.push(String::Owned(selected));
    }Array(output)
}
fn main(){}
