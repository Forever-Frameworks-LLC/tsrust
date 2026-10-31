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
fn keep<F: Fn(i64) -> i64>(f: F) -> F {
    f
}
fn relay<C: FnMut(F) -> F, F: Fn(i64) -> i64>(c: &mut C, f: F) -> F {
    c(f)
}
fn plus(offset: i64) -> impl Fn(i64) -> i64 {
    move |x| add(x, offset)
}
fn times(scale: i64) -> impl Fn(i64) -> i64 {
    move |x| mul(x, scale)
}

fn function0(value0: i64, value1: i64) -> i64 {
    let mut trace = value1;
    for i in 0..value0 {
        let left = plus(value1 % 11);
        let right = times(add(value1 % 7, 1));
        let first = relay(&mut keep, left);
        let second = relay(&mut keep, right);
        let a = first(i);
        let b = second(i);
        let c = first(add(i, 1));
        trace = digest(trace, add(add(mul(a, 7), b), c));
    }
    trace
}
fn main() {}
