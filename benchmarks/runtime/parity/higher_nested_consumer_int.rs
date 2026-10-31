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
type Consumer = fn(&mut dyn FnMut(i64) -> i64, i64) -> i64;
fn plus(x: i64) -> i64 {
    add(x, 3)
}
fn times(x: i64) -> i64 {
    mul(x, 2)
}
fn apply(f: &mut dyn FnMut(i64) -> i64, x: i64) -> i64 {
    f(x)
}
fn pass(c: &mut Consumer, f: &mut dyn FnMut(i64) -> i64, x: i64) -> i64 {
    c(f, x)
}
fn factory() -> Consumer {
    apply
}
fn factory_factory() -> fn() -> Consumer {
    factory
}

fn function0(value0: i64, value1: i64) -> i64 {
    let maker = factory_factory;
    let factory = maker();
    let mut consumer = factory();
    let mut trace = value1;
    for i in 0..value0 {
        let a = pass(&mut consumer, &mut plus, add(value1, i));
        let b = pass(&mut consumer, &mut times, add(value1, i));
        trace = digest(trace, add(mul(a, 7), b));
    }
    trace
}
fn main() {}
