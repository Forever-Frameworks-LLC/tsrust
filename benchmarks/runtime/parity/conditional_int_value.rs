// Independent Rust conditional selection with observable branch counters.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let(mut predicates,mut left_calls,mut right_calls)=(0_i64,0_i64,0_i64);let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(3).unwrap()).unwrap());
    for i in 0..value0{let left=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%64;let right=(i.checked_mul(7).unwrap().checked_add(value1).unwrap()%64).checked_add(64).unwrap();let flag=i.checked_add(value1).unwrap()%2==0;predicates=predicates.checked_add(1).unwrap();let selected=if flag{left_calls=left_calls.checked_add(1).unwrap();left}else{right_calls=right_calls.checked_add(1).unwrap();right};output.push(selected);}output.extend([predicates,left_calls,right_calls]);NumericArray(output)
}
fn main(){}
