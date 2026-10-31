// Independent standard Rust/serde JSON stream reference.
use std::io::{self, BufRead, Write};
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
#[derive(serde::Serialize)]
struct Item {
    active: bool,
    name: std::string::String,
    values: Vec<i64>,
}
fn write_item(writer: &mut impl Write, item: &Item) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, item)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writer.write_all(b"\n")
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let item = Item {
            active: true,
            name: "hello".to_owned(),
            values: vec![20, 17],
        };
        let file = std::fs::File::create(value2.as_ref()).unwrap();
        let mut output = io::BufWriter::with_capacity(8192, file);
        {
            let writer = &mut output;
            write_item(writer, &item).unwrap();
            writer.flush().unwrap();
        }
        drop(output.into_inner().map_err(|e| e.into_error()).unwrap());
        checksum = add(checksum, 1);
    }
    checksum
}
fn main() {}
