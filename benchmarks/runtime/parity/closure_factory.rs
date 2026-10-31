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
fn counter(start: i64) -> impl FnMut() -> i64 {
    let mut value = start;
    move || {
        value = add(value, 1);
        value
    }
}

fn function0(value0: i64, value1: i64) -> i64 {
    let mut action = counter(value1);
    let mut sum = 0;
    for _ in 0..value0 {
        sum = add(sum, action());
    }
    sum
}
fn main() {}
