// Independent standard Rust file I/O; successful regular-file fixture.
use std::io::{self, Read, Write};
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
fn observe(bytes: &[u8]) -> NumericArray<i64> {
    NumericArray(vec![
        i64::try_from(bytes.len()).unwrap(),
        bytes.iter().fold(0_i64, |total, &byte| {
            total.checked_add(i64::from(byte)).unwrap()
        }),
    ])
}
fn read_chunk(reader: &mut impl Read, count: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![0; count];
    let size = reader.read(&mut bytes)?;
    bytes.truncate(size);
    Ok(bytes)
}
// Rust File uses deterministic Drop; std has no fallible explicit close API.
fn sample(path: &str, count: usize) -> io::Result<NumericArray<i64>> {
    let mut file = std::fs::File::open(path)?;
    let bytes = read_chunk(&mut file, count)?;
    let result = observe(&bytes);
    Ok(result)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut rows = Vec::new();
    for i in 0..value0 {
        let count = usize::try_from(i.checked_add(value1).unwrap() % 64 + 1).unwrap();
        rows.push(sample(value2.as_ref(), count).unwrap());
    }
    Array(rows)
}
fn main() {}
