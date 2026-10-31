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
    let values = [
        "".to_owned(),
        "a".to_owned(),
        "aa".to_owned(),
        "alpha".to_owned(),
        "alphabet".to_owned(),
        "z".to_owned(),
        value2.to_string(),
    ];
    let mut result = value1;
    for i in 0..value0 {
        let left = index(i, 13, value1, 7);
        let right = index(i, 17, mul(value1, 3), 7);
        if values[left] < values[right] {
            result = add(result, 1);
        }
        if values[(left + 1) % 7] <= values[(right + 2) % 7] {
            result = add(result, 2);
        }
        if values[(left + 2) % 7] > values[(right + 3) % 7] {
            result = add(result, 4);
        }
        if values[(left + 3) % 7] >= values[(right + 4) % 7] {
            result = add(result, 8);
        }
    }
    result
}
fn main() {}
