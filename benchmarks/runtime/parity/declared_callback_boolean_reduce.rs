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
fn flip(a: bool, v: &bool) -> bool {
    a != *v
}
fn function0(value0: i64, value1: i64) -> bool {
    let base = [true, false, value1 % 2 == 0, false, true];
    let mut total = false;
    for i in 0..value0 {
        let values = &base[..count(i, value1)];
        total = total != values.iter().fold(false, flip);
    }
    total
}
fn main() {}
