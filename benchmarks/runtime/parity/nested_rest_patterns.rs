// Independent standard Rust reference; retain pattern/default/control operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
fn describe(values: &[Text]) -> Text {
    let [_, text, rest @ ..] = values else {
        panic!("Missing text")
    };
    let fallback = rest.first().map(Text::as_str).unwrap_or("tail");
    format!("{text}:{fallback}")
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let words = vec![value2.repeat(1), base.to_string(), value2.repeat(2)];
        let (first, rest) = words.split_first().unwrap();
        let (second, tail) = rest.split_first().unwrap();
        let mut tail = tail.to_vec();
        tail.push("end".to_owned());
        let length = words[1..].len();
        let tuple = (base, value2.repeat(1), add(base, 1));
        let (head, text, amount) = tuple;
        output.push(Array(vec![
            owned(first.clone()),
            owned(second.clone()),
            owned(tail.join("|")),
            owned(length.to_string()),
            owned(head.to_string()),
            owned(text),
            owned(amount.to_string()),
            owned(describe(&[value2.repeat(1), value2.repeat(2)])),
            owned(words.len().to_string()),
        ]));
    }
    output
}
fn main() {}
