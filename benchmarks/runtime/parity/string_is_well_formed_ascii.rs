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
// Rust str guarantees valid UTF-8; no additional validation pass is needed.
fn well_formed(_text: &str) -> bool {
    true
}

fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<bool> {
    let values = [
        value2.to_string(),
        "".to_owned(),
        "ABC".to_owned(),
        "a\u{0000}b".to_owned(),
        " e ".to_owned(),
        "Z".to_owned(),
        value2.repeat(2),
        "tail".to_owned(),
    ];
    let mut output = Vec::new();
    for i in 0..value0 {
        let at = index(i, 13, value1, 8);
        output.push(well_formed(&values[at]));
    }
    NumericArray(output)
}
fn main() {}
