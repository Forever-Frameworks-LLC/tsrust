// Independent standard Rust higher-order scalar callbacks and owned factories.
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}
fn plus(x: i64) -> i64 {
    add(x, 3)
}
fn times(x: i64) -> i64 {
    mul(x, 2)
}
fn apply<F: FnMut(i64) -> i64>(f: &mut F, x: i64) -> i64 {
    f(x)
}

fn function0(value0: i64, value1: i64) -> i64 {
    let consumer = apply;
    let mut left: fn(i64) -> i64 = plus;
    let mut right: fn(i64) -> i64 = times;
    let mut trace = value1;
    for i in 0..value0 {
        let a = consumer(&mut left, add(value1, i));
        let b = consumer(&mut right, add(value1, i));
        trace = digest(trace, add(mul(a, 7), b));
    }
    trace
}
fn main() {}
