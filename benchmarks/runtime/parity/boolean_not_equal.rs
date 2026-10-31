// Independent standard Rust scalar control and operators, with checked arithmetic.
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}

fn function0(value0: i64, value1: i64) -> i64 {
    let left_values = [false, false, true, true];
    let right_values = [false, true, false, true];
    let mut result_digest = value1;
    for i in 0..value0 {
        let at = usize::try_from(add(mul(i, 13), value1) % 4).unwrap();
        let left = left_values[at];
        let right = right_values[at];
        result_digest = digest(result_digest, i64::from(left != right));
    }
    result_digest
}
fn main() {}
