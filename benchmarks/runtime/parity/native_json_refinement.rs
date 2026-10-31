// Independent Rust JSON reference using serde_json and ordinary Rust ownership.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn parse(input: &str) -> Value {
    serde_json::from_str(input).expect("Invalid JSON")
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let value = parse(value2);
        let active = value.as_bool().expect("Expected boolean");
        if active {
            checksum = add(checksum, 1);
        }
    }
    checksum
}
fn main() {}
