// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn copy(text: &str) -> std::string::String {
    text.to_owned()
}
fn drive<F: FnMut(&str) -> std::string::String>(f: &mut F, text: &str) -> std::string::String {
    f(text)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<String> {
    let consumer = drive;
    let mut output = Vec::new();
    for _ in 0..value0 {
        output.push(String::Owned(consumer(&mut copy, value2)));
    }
    output
}
fn main() {}
