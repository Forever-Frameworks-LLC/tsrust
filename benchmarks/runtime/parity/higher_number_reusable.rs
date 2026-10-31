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
fn identity(value: f64) -> f64 {
    value
}
fn apply<F: FnMut(f64) -> f64>(f: &mut F, value: f64) -> f64 {
    f(value)
}
fn function0(value0: i64, value1: i64) -> f64 {
    let consumer = apply;
    let mut total = 0.0;
    for i in 0..value0 {
        let value = (add(value1, i) % 17) as f64 / 4.0;
        total += consumer(&mut identity, value);
    }
    total
}
fn main() {}
