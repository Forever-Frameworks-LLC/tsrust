// Expression-value variant: retain the assigned value and final target.
// Independent standard Rust operator reference. Operand tables are the matched
// benchmark corpus; the implementation uses checked Rust integer semantics.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn operation(value:i64,other:i64)->i64{value.checked_shr(u32::try_from(other).expect("invalid shift")).expect("invalid shift")}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let operands:[i64;8]=[0, 1, 2, 3, 7, 15, 65535, 1073741823];
    let rhs:[i64;8]=[0, 1, 2, 3, 4, 7, 15, 31];
    let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).expect("invalid count"));
    for i in 0..value0 {
        let value=operands[usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        let other=rhs[usize::try_from((i.checked_mul(7).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        let target=operation(value,other);let observed=target;output.push(observed);output.push(target);
    }
    NumericArray(output)
}
fn main() {}
