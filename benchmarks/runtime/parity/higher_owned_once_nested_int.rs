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
fn use_once<C: FnOnce(F) -> R, F: FnOnce(i64) -> i64, R: FnOnce() -> i64>(c: C, f: F) -> R {
    c(f)
}
fn bind<F: FnOnce(i64) -> i64>(f: F, index: i64) -> impl FnOnce() -> i64 {
    move || f(index)
}
fn function0(value0: i64, value1: i64) -> i64 {
    let consumer = use_once;
    let mut trace = value1;
    for i in 0..value0 {
        let callback = |x| add(x, value1 % 13);
        let factory = consumer(move |f| bind(f, i), callback);
        trace = digest(trace, factory());
    }
    trace
}
fn main() {}
