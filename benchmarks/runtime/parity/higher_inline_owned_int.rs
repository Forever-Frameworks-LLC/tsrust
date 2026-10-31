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
fn apply<F: FnMut(i64) -> i64>(mut f: F, x: i64) -> i64 {
    f(x)
}
fn function0(value0: i64, value1: i64) -> i64 {
    let consumer = apply;
    let mut trace = value1;
    for i in 0..value0 {
        let mut callback = |x| add(mul(x, 3), 1) % 1_000_000_007;
        let value = consumer(callback, add(value1, i));
        trace = digest(trace, value);
    }
    trace
}
fn main() {}
