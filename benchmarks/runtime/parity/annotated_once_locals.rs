// Independent standard Rust callback contracts and owned callable storage.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn single<F: FnOnce(i64) -> i64>(action: F, value: i64) -> i64 {
    action(value)
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for i in 0..value0 {
        let base = add(i, value1);
        let action = move |value| add(base, value);
        checksum = add(checksum, single(action, i));
    }
    checksum
}
fn main() {}
