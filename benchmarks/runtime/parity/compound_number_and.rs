// Independent Rust implementation of the workload's 32-bit number contract.
// Number bitwise operations require modulo word conversion; native Rust int
// operators have separate 64-bit references. Standard Rust conversions only.
fn word(value:f64)->u32{if value.is_finite(){value.trunc().rem_euclid(4294967296.0) as u32}else{0}}
fn operation(value:f64,rhs:f64)->f64{(word(value) & word(rhs)) as i32 as f64}
fn function0(value0: i64, value1: i64) -> Vec<f64> {
    let left:[f64;16]=[-8589934592.0, -4294967297.0, -4294967296.0, -4294967295.0, -2147483649.0, -2147483648.0, -1.0, 0.0, 0.0, 1.0, 2147483647.0, 2147483648.0, 4294967295.0, 4294967296.0, 4294967297.0, 8589934591.0];let right:[f64;16]=[-4294967297.0, -4294967296.0, -4294967295.0, -2147483649.0, -2147483648.0, -1.0, 0.0, 1.0, 2.0, 31.0, 32.0, 33.0, 2147483647.0, 2147483648.0, 4294967295.0, 4294967296.0];
    let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{
        let value=left[usize::try_from((i.checked_mul(13).unwrap().checked_add(value1).unwrap())%16).unwrap()];
        let rhs=right[usize::try_from((i.checked_mul(7).unwrap().checked_add(value1).unwrap())%16).unwrap()];
        output.push(operation(value,rhs));
    }
    output
}
fn main(){}
