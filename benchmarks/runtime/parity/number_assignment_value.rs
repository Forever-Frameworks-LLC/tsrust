// Independent Rust assignment expression: retain result and assigned target.
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let initial:[f64;8]=[8.0, -8.0, 0.0, -0.0, 0.5, -0.5, 2.0, -2.0];let rhs:[f64;8]=[-2.25, -0.0, 0.0, 1.5, 9007199254740992.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(2).unwrap()).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%8).unwrap();let mut target=initial[at];let rhs_at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();target=rhs[rhs_at];let observed=target;output.extend([observed,target]);}output
}
fn main(){}
