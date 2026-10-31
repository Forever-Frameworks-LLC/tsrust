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
    let owner = parse(value2);
    let mut checksum = value1;
    for _ in 0..value0 {
        let items = owner
            .as_object()
            .expect("object")
            .get("items")
            .expect("missing array");
        let child = items
            .as_array()
            .expect("array")
            .get(0)
            .expect("missing item");
        checksum = add(checksum, child.as_i64().expect("integer"));
    }
    checksum
}
fn main() {}
