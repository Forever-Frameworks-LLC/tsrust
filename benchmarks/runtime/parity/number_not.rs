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

// Measured inputs are finite, in (-2^34,2^34). Truncate via i64 then retain 32 bits.
fn word(value: f64) -> u32 {
    value as i64 as u32
}
struct Probe {
    trace: i64,
    calls: i64,
}
fn mark(probe: &mut Probe, value: f64, tag: i64) -> f64 {
    probe.trace = add(mul(probe.trace, 17), tag) % 1009;
    probe.calls = add(probe.calls, 1);
    value
}
fn operation(left: f64) -> f64 {
    let left = word(left);
    (!left) as i32 as f64
}

fn function0(value0: i64, value1: i64) -> f64 {
    let table0 = [
        -8589934592.0,
        -4294967297.0,
        -4294967296.0,
        -4294967295.0,
        -2147483649.0,
        -2147483648.0,
        -1.0,
        -0.0,
        0.0,
        1.0,
        2147483647.0,
        2147483648.0,
        4294967295.0,
        4294967296.0,
        4294967297.0,
        8589934591.0,
    ];
    let mut probe = Probe {
        trace: value1 % 1009,
        calls: 0,
    };
    let mut sum = value1 as f64;
    for i in 0..value0 {
        let jitter = add(mul(i, 17), value1) % 5;
        let argument0 =
            table0[usize::try_from(add(mul(i, 13), value1) % 16).unwrap()] + jitter as f64 * 0.25;
        let left = mark(&mut probe, argument0, 1);
        let result = operation(left);
        sum += result;
    }
    sum + probe.trace as f64 + probe.calls as f64
}
fn main() {}
