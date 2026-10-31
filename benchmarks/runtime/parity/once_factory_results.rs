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
fn make<T>(value: T) -> impl FnOnce() -> T {
    move || value
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for i in 0..value0 {
        let action = make(add(i, value1));
        checksum = add(checksum, action());
    }
    checksum
}
fn main() {}
