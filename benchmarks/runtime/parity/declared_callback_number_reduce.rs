// Independent standard Rust declared functions, callbacks and owned values.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}
fn count(i: i64, seed: i64) -> usize {
    [0, 1, 4, 5][usize::try_from(add(seed, i) % 4).unwrap()]
}
fn sum(a: f64, v: &f64) -> f64 {
    a + v
}
fn function0(value0: i64, value1: i64) -> f64 {
    let base = [0.25, -0.5, 1.5, 2.25, -1.0];
    let mut total = 0.0;
    for i in 0..value0 {
        let values = &base[..count(i, value1)];
        total += values.iter().fold(0.0, sum);
    }
    total
}
fn main() {}
