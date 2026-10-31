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
#[derive(Default)]
struct Probe {
    calls: i64,
    trace: i64,
}
fn mark(probe: &mut Probe, value: bool, tag: i64) -> bool {
    probe.calls = add(probe.calls, 1);
    probe.trace = digest(probe.trace, tag);
    value
}
fn evaluate(probe: &mut Probe, left: bool, right: bool) -> i64 {
    let either = left || mark(probe, right, 1);
    let both = left && mark(probe, right, 2);
    add(i64::from(either), mul(i64::from(both), 2))
}

fn function0(value0: i64, value1: i64) -> i64 {
    let left_values = [false, false, true, true];
    let right_values = [false, true, false, true];
    let mut probe = Probe::default();
    let mut result_digest = value1;
    for i in 0..value0 {
        let at = usize::try_from(add(mul(i, 13), value1) % 4).unwrap();
        let bits = evaluate(&mut probe, left_values[at], right_values[at]);
        result_digest = digest(result_digest, bits);
    }
    add(add(mul(result_digest, 131), probe.trace), probe.calls) % 1_000_000_007
}
fn main() {}
