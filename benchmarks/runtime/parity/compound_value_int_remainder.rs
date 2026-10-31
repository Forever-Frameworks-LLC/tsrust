// Expression-value variant: retain the assigned value and final target.
// Independent Rust compound arithmetic with checked integer semantics.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn operation(base:i64,rhs:i64)->i64{base.checked_mul(rhs).unwrap().checked_add(base.checked_rem(rhs).unwrap()).unwrap().checked_rem(rhs).expect("invalid integer remainder")}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).expect("invalid count"));
    for i in 0..value0 {
        let base=(i.checked_mul(13).unwrap().checked_add(value1).unwrap())%64;
        let rhs=((i.checked_mul(7).unwrap().checked_add(value1).unwrap())%4).checked_add(1).unwrap();
        let target=operation(base,rhs);let observed=target;output.push(observed);output.push(target);
    }
    NumericArray(output)
}
fn main() {}
