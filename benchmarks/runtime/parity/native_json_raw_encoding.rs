// Independent Rust JSON reference using serde_json and ordinary Rust ownership.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
// Validate without constructing a JSON tree, retaining the complete original slice.
fn raw(input: &str) -> Result<&str, serde_json::Error> {
    let _: &serde_json::value::RawValue = serde_json::from_str(input)?;
    Ok(input)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let raw = raw(value2).expect("Invalid JSON");
        let encoded = format!("{{\"payload\":{raw}}}");
        assert_eq!(
            encoded,
            r#"{"payload": { "a": [1,true,null], "b": "hello" } }"#
        );
        checksum = add(checksum, length(&encoded));
    }
    checksum
}
fn main() {}
