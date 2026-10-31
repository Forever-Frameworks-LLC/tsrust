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
    let mut sum = value1;
    let mut state = value1;
    for i in 0..value0 {
        let value = add(state, i) % 13;
        let code = if value < 3 {
            11
        } else if value < 7 {
            23
        } else if value < 11 {
            37
        } else {
            49
        };
        state = add(add(mul(state, 17), code), value1) % 997;
        sum = digest(sum, state);
    }
    sum
}
fn main() {}
