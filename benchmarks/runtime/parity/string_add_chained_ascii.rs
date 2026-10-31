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

fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let mut values = vec![value2.to_string(), value2.repeat(2), value2.to_string()];
    values.extend([
        "".to_owned(),
        "A\0B".to_owned(),
        " left ".to_owned(),
        "right".to_owned(),
        value2.to_string(),
    ]);
    let mut output = Vec::new();
    for i in 0..value0 {
        let mut selected = values[index(i, 13, value1, 8)].clone();
        selected.push_str(&values[index(i, 7, add(value1, 3), 8)]);
        selected.push_str(&values[index(i, 5, add(value1, 6), 8)]);
        output.push(String::Owned(selected));
    }
    Array(output)
}
fn main() {}
