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
fn invert(value: bool) -> bool {
    !value
}
fn function0(value0: i64, value1: i64) -> bool {
    let consumer = |f: &mut dyn FnMut(bool) -> bool, value| f(value);
    let mut parity = false;
    for i in 0..value0 {
        let value = consumer(&mut invert, add(value1, i) % 3 == 0);
        if value {
            parity = !parity;
        }
    }
    parity
}
fn main() {}
