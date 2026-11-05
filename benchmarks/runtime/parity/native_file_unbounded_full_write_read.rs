// Independent standard Rust file I/O; successful regular-file fixture.
use std::io;
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>([T;2]);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn observe(bytes: &[u8]) -> NumericArray<i64> {
    NumericArray([
        i64::try_from(bytes.len()).unwrap(),
        bytes.iter().fold(0_i64, |total, &byte| {
            total.checked_add(i64::from(byte)).unwrap()
        }),
    ])
}
fn sample(path: &str, count: usize, fill: u8) -> io::Result<NumericArray<i64>> {
    let destination=format!("{path}.out");
    { let owner=vec![fill;count];std::fs::write(&destination,&owner)?; }
    Ok(observe(&std::fs::read(destination)?))
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut rows = Vec::new();
    for i in 0..value0 {
        let count=[65536,1048576,67108865][usize::try_from(i%3).unwrap()];
        let fill=u8::try_from(i.checked_add(value1).unwrap()%256).unwrap();
        rows.push(sample(value2.as_ref(),count,fill).unwrap());
    }
    Array(rows)
}
fn main() {}
