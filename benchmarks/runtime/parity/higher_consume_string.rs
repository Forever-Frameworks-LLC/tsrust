// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn take(text: std::string::String) -> std::string::String {
    text
}
fn transfer<F: FnOnce(std::string::String) -> std::string::String>(
    f: F,
    text: std::string::String,
) -> std::string::String {
    f(text)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<String> {
    let consumer = transfer;
    let mut output = Vec::new();
    for _ in 0..value0 {
        let value = value2.to_string();
        output.push(String::Owned(consumer(take, value)));
    }
    output
}
fn main() {}
