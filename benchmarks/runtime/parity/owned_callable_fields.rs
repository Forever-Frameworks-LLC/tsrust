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
struct Handler {
    run: Box<dyn Fn(i64) -> i64>,
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for i in 0..value0 {
        let base = add(i, value1);
        let mut handler = Handler {
            run: Box::new(move |value| add(base, value)),
        };
        checksum = add(checksum, (handler.run)(i));
        handler.run = Box::new(move |value| sub(base, value));
        checksum = add(checksum, (handler.run)(i));
    }
    checksum
}
fn main() {}
