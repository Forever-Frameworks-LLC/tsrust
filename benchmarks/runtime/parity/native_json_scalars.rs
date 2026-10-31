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
        checksum = add(checksum, value.as_i64().expect("integer"));
        assert_eq!(value.as_f64().expect("number"), 42.0);
        assert!(parse("true").as_bool().expect("boolean"));
        let Value::String(text) = parse("\"hello\"") else {
            panic!("string")
        };
        assert_eq!(text, "hello");
    }
    checksum
}
fn main() {}
