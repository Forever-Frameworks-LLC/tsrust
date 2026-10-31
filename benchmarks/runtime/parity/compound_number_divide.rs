// Independent Rust floating-point compound arithmetic; preserve IEEE results.
fn operation(mut value:f64,rhs:f64)->f64{value /= rhs;value}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let left:[f64;8]=[-3.0, -0.5, -0.25, -0.0, 0.0, 0.25, 0.75, 2.5];let right:[f64;8]=[-4.0, -2.0, -0.5, 0.5, 1.0, 2.0, 4.0, 8.0];
    let mut output=Vec::with_capacity(usize::try_from(value0).expect("invalid count"));
    for i in 0..value0 {
        let value=left[usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        let rhs=right[usize::try_from((i.checked_mul(7).unwrap().checked_add(value1).unwrap())%8).unwrap()];
        output.push(operation(value,rhs));
    }
    output
}
fn main() {}
