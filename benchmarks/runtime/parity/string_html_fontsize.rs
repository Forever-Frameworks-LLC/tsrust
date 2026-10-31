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
fn wrap(value: &str, attribute: &str) -> std::string::String {
    format!(
        "<font size=\"{}\">{}</font>",
        attribute.replace('"', "&quot;"),
        value
    )
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let values = [
        value2.to_string(),
        "".to_owned(),
        "raw &<>\u{0000}".to_owned(),
        "é😀é".to_owned(),
        "﻿quote\"".to_owned(),
        "a'&<>".to_owned(),
        "static".to_owned(),
        value2.repeat(2),
    ];
    let attributes = [
        "".to_owned(),
        value2.to_string(),
        "\"".to_owned(),
        "a\"b".to_owned(),
        "&<>".to_owned(),
        "'\u{0000}".to_owned(),
        "é😀".to_owned(),
        value2.repeat(2),
        "&quot;".to_owned(),
    ];
    let mut output = Vec::new();
    for i in 0..value0 {
        let at = index(i, 13, value1, 8);
        output.push(String::Owned(wrap(
            &values[at],
            &attributes[index(i, 7, value1, 9)],
        )));
    }
    Array(output)
}
fn main() {}
