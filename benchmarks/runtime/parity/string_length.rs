// Independent standard Rust string operations; retained dynamic outputs are independently owned.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn index(i: i64, stride: i64, seed: i64, len: i64) -> usize {
    usize::try_from(add(mul(i, stride), seed) % len).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}

fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let values = [value2.to_string(), value2.trim().to_owned()];
    let mut result = value1;
    for i in 0..value0 {
        result = add(mul(result, 131), length(&values[index(i, 1, value1, 2)])) % 1_000_000_007;
    }
    result
}
fn main() {}
