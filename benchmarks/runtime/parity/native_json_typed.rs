// Independent Rust JSON reference using serde_json and ordinary Rust ownership.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
#[derive(serde::Deserialize)]
struct Item {
    name: std::string::String,
    values: Vec<i64>,
    active: bool,
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let item: Item = serde_json::from_str(value2).expect("Invalid item");
        assert!(item.active, "Inactive item");
        checksum = add(
            checksum,
            add(add(length(&item.name), item.values[0]), item.values[1]),
        );
    }
    checksum
}
fn main() {}
