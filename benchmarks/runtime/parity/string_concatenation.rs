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
    let mut output = Vec::new();
    for i in 0..value0 {
        let k = add(i, value1);
        let flag = k % 2 == 0;
        let nullable = if flag { Some(k) } else { None };
        let mut text = value2.as_ref().to_owned();
        text.push_str(&k.to_string());
        text.push_str(&flag.to_string());
        let union = nullable
            .map(|v| v.to_string())
            .unwrap_or_else(|| "undefined".to_owned());
        output.push(Array(vec![
            owned(format!("{value2}{k}")),
            owned(format!("{k}{value2}")),
            owned(format!("fraction:{}", (k % 4) as f64 / 2.0)),
            owned(format!("flag:{flag}")),
            owned(format!("absent:{}{}", "null", "undefined")),
            owned(format!("union:{union}")),
            owned(text),
        ]));
    }
    output
}
fn main() {}
