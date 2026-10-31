// Independent standard Rust string conversion/construction reference.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
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

fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values = [
        value2.to_string(),
        "".to_owned(),
        "ABC".to_owned(),
        "a\u{0000}b".to_owned(),
        " x ".to_owned(),
        "quote\"".to_owned(),
        "slash\\".to_owned(),
        value2.repeat(2),
    ];
    let fractions = [0.5, 1.25, 2.5, 12.75];
    let mut output = Vec::new();
    for i in 0..value0 {
        let at = index(i, 13, value1, 8);
        let count = add(mul(i, 7), value1) % 97;
        let fraction = fractions[index(i, 1, value1, 4)];
        let flag = add(i, value1) % 3 == 0;
        output.push(String::Owned(format!(
            "head:{}|{}|{}|{}:tail",
            values[at], count, fraction, flag
        )));
    }
    Array(output)
}
fn main() {}
