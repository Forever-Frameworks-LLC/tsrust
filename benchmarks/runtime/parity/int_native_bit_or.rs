// Independent standard Rust operator reference. Operand tables are the matched
// benchmark corpus; the implementation uses checked Rust integer semantics.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn operation(value:i64,other:i64)->i64{value | other}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let operands:[i64;8]=[0, 1, 3, 7, 15, 63, 255, 65535];
    let rhs:[i64;8]=[0, 2, 5, 8, 17, 42, 170, 43690];
    let mut output=Vec::with_capacity(usize::try_from(value0).expect("invalid count"));
    for i in 0..value0 {
        let value=operands[usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        let other=rhs[usize::try_from((i.checked_mul(7).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        output.push(operation(value,other));
    }
    NumericArray(output)
}
fn main() {}
