// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn identity() -> &'static str {
    if cfg!(target_endian = "little") {
        "LE"
    } else {
        "BE"
    }
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let _ = value1;
    let mut values = Vec::new();
    for _ in 0..value0 {
        values.push(String::Borrowed(identity()));
    }
    Array(values)
}
fn main() {}
