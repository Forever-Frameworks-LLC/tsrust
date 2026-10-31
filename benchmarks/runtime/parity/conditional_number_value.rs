// Independent Rust conditional selection with observable branch counters.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let values=[0.0,-0.0,0.5,-0.5,f64::INFINITY,f64::NEG_INFINITY,f64::NAN,4.0];let(mut predicates,mut left_calls,mut right_calls)=(0_i64,0_i64,0_i64);let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(3).unwrap()).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1.checked_mul(2).unwrap()).unwrap()%8).unwrap();let flag=i.checked_add(value1).unwrap()%3==0;predicates=predicates.checked_add(1).unwrap();let selected=if flag{left_calls=left_calls.checked_add(1).unwrap();values[at]}else{right_calls=right_calls.checked_add(1).unwrap();values[(at+3)%8]};output.push(selected);}output.extend([predicates as f64,left_calls as f64,right_calls as f64]);output
}
fn main(){}
