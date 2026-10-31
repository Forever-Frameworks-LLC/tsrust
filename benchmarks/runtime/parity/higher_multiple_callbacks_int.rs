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
fn pair<L: FnMut(i64) -> i64, R: FnMut(i64) -> i64>(left: &mut L, right: &mut R, x: i64) -> i64 {
    add(mul(left(x), 7), right(x))
}

fn function0(value0: i64, value1: i64) -> i64 {
    let consumer = pair;
    let mut trace = value1;
    for i in 0..value0 {
        let a = consumer(&mut plus, &mut times, add(value1, i));
        let b = (|left: &mut dyn FnMut(i64) -> i64, right: &mut dyn FnMut(i64) -> i64, x| {
            add(mul(left(x), 7), right(x))
        })(&mut times, &mut plus, add(value1, i));
        trace = digest(trace, add(a, b));
    }
    trace
}
fn main() {}
