// Independent Rust assignment expression: retain result and assigned target.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let initial:[i64;8]=[257, 129, 64, 16, 8, 3, 2, 1];let rhs:[i64;8]=[0, 1, 2, 3, 7, 31, 129, 1024];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%8).unwrap();let mut target=initial[at];let rhs_at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();target=rhs[rhs_at];let observed=target;output.extend([observed,target]);}NumericArray(output)
}
fn main(){}
