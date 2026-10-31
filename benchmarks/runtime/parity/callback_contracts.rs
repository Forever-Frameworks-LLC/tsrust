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
fn forward<F: FnMut(i64, bool) -> i64>(mut action: F, value: i64) -> i64 {
    action(value, true)
}
fn single<F: FnOnce() -> i64>(action: F) -> i64 {
    action()
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = 0;
    for i in 0..value0 {
        let action = |value, flag| if flag { add(value, value1) } else { value };
        sum = add(sum, forward(action, i));
        let once_action = || value1;
        sum = add(sum, single(once_action));
    }
    sum
}
fn main() {}
