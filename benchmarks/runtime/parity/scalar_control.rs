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
    let mut state = value1;
    let mut sum = value1;
    for _ in 0..value0 {
        state = add(mul(state, 17), 13) % 1009;
        if state % 2 == 0 && state > 100 {
            sum = add(sum, state)
        } else {
            sum = sub(sum, 1)
        }
    }
    sum
}
fn main() {}
