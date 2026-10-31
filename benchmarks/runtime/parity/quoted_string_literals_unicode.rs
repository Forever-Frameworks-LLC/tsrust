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
        "é😀".to_owned(),
        "BOM﻿中".to_owned(),
        "é".to_owned(),
        "é中".to_owned(),
        "😀􏿿".to_owned(),
        "😀".to_owned(),
        "é".to_owned(),
        "qé$_".to_owned(),
        "\u{0000}é😀".to_owned(),
        "é😀".to_owned(),
        "é😀".to_owned(),
        "é😀".to_owned(),
        "é😀".to_owned(),
        "é😀".to_owned(),
        "".to_owned(),
    ];
    let mut output = Vec::new();
    for i in 0..value0 {
        output.push(String::Owned(values[index(i, 13, value1, 16)].clone()));
    }
    Array(output)
}
fn main() {}
