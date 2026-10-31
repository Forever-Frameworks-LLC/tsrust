// Independent Rust boolean assignment with retained result and target values.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let left=[false,false,true,true];let right=[false,true,false,true];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{
        let at=usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%4).unwrap();let mut target=left[at];
        target=target && right[at];
        let observed=target;
        output.push(i64::from(observed));output.push(i64::from(target));
    }
    NumericArray(output)
}
fn main(){}
