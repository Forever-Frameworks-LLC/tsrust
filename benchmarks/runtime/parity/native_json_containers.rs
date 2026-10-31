// Independent Rust JSON reference using serde_json and ordinary Rust ownership.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let values = vec![Value::from(42_i64), Value::Null];
        let entries = vec![
            ("items".to_owned(), Value::Array(values)),
            ("name".to_owned(), Value::String("old".to_owned())),
            ("name".to_owned(), Value::String(value2.as_ref().to_owned())),
        ];
        let mut object = serde_json::Map::new();
        for (key, value) in entries {
            object.insert(key, value);
        }
        let text = serde_json::to_string(&Value::Object(object)).unwrap();
        checksum = add(checksum, length(&text));
    }
    checksum
}
fn main() {}
