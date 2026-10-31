// Independent Rust conditional evaluation with observable branch counters.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(3).unwrap()).unwrap());
    let(mut predicates,mut left_calls,mut right_calls)=(0_i64,0_i64,0_i64);
    for i in 0..value0{
        let left=(i.checked_mul(13).unwrap().checked_add(value1).unwrap())%3==0;
        let right=(i.checked_mul(7).unwrap().checked_add(value1).unwrap())%3==1;
        predicates=predicates.checked_add(1).unwrap();
        let selected=if i.checked_add(value1).unwrap()%2==0{left_calls=left_calls.checked_add(1).unwrap();left}
            else{right_calls=right_calls.checked_add(1).unwrap();right};
        output.push(i64::from(selected));
    }
    output.extend([predicates,left_calls,right_calls]);NumericArray(output)
}
fn main(){}
