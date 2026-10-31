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
fn string_score(text: &str) -> i64 {
    let first = text
        .find("hello")
        .map(|at| i64::try_from(at).unwrap())
        .unwrap_or(-1);
    let last = text
        .rfind("hello")
        .map(|at| i64::try_from(at).unwrap())
        .unwrap_or(-1);
    add(add(length(text), first), last)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let owner = parse(value2);
    let mut checksum = value1;
    for _ in 0..value0 {
        let text = owner.as_str().expect("Expected string");
        checksum = add(checksum, string_score(text));
    }
    checksum
}
fn main() {}
