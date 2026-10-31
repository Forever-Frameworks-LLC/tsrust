// Independent Rust JSON reference using serde_json and ordinary Rust ownership.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
#[derive(serde::Serialize)]
struct Item {
    active: bool,
    name: std::string::String,
    values: Vec<i64>,
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let item = Item {
            active: true,
            name: value2.as_ref().to_owned(),
            values: vec![20, 17],
        };
        let text = serde_json::to_string(&item).unwrap();
        assert_eq!(text, r#"{"active":true,"name":"hello","values":[20,17]}"#);
        checksum = add(checksum, length(&text));
    }
    checksum
}
fn main() {}
