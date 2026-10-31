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
    let mut result_digest = value1;
    for i in 0..value0 {
        let left = sub(add(mul(i, 37), value1) % 2001, 1000);
        let right = sub(add(mul(i, 17), mul(value1, 3)) % 2001, 1000);
        result_digest = digest(result_digest, i64::from(left <= right));
    }
    result_digest
}
fn main() {}
