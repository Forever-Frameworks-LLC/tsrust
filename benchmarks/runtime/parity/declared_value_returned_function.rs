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
fn step(value: i64) -> i64 {
    add(mul(value, 3), 1) % 1_000_000_007
}
struct Probe {
    state: i64,
}
fn produce(probe: &mut Probe) -> fn(i64) -> i64 {
    probe.state = add(mul(probe.state, 10), 7) % 1_000_003;
    step
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut trace = value1;
    for i in 0..value0 {
        let mut probe = Probe {
            state: add(value1, i) % 1_000_003,
        };
        let action = produce(&mut probe);
        let value = action(add(value1, i));
        trace = add(add(mul(trace, 131), value), probe.state) % 1_000_000_007;
    }
    trace
}
fn main() {}
