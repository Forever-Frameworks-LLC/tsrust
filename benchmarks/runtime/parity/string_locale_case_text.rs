// Independent standard Rust language reference; preserve values and owned data operations.
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
fn owned(text: impl Into<Text>) -> String {
    String::Owned(text.into())
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let words = ["Ab", "Éé", "ßs", "Ωω"];
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let text =
            words[usize::try_from(i % 4).unwrap()].repeat(usize::try_from(base % 3 + 1).unwrap());
        output.push(Array(vec![
            owned(text.to_lowercase()),
            owned(text.to_uppercase()),
        ]));
    }
    output
}
fn main() {}
