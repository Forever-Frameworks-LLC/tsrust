// Independent standard Rust/serde JSON stream reference.
use std::io::{self, BufRead, Write};
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
#[derive(serde::Deserialize)]
struct Item {
    name: std::string::String,
    values: Vec<i64>,
    active: bool,
}
fn observe(item: Item) -> i64 {
    assert!(item.active, "Inactive item");
    add(
        add(i64::try_from(item.name.len()).unwrap(), item.values[0]),
        item.values[1],
    )
}
fn read_item(reader: &mut impl BufRead) -> io::Result<Option<Item>> {
    let mut line = std::string::String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    serde_json::from_str(&line)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
fn consume(reader: &mut impl BufRead) -> io::Result<i64> {
    let mut total = 0;
    while let Some(item) = read_item(reader)? {
        total = add(total, observe(item));
    }
    Ok(total)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let file = std::fs::File::open(value2.as_ref()).unwrap();
        let mut buffer = io::BufReader::with_capacity(8192, file);
        {
            let reader = &mut buffer;
            checksum = add(checksum, consume(reader).unwrap());
        }
        drop(buffer);
    }
    checksum
}
fn main() {}
