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
struct Row {
    value: i64,
}
enum Alternative {
    Integer(i64),
    Text(Text),
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let mut values = Vec::new();
        values.push(value);
        let checked = value;
        let widened = value as f64;
        let alternative = Alternative::Integer(value);
        let row = Row { value: checked };
        let mut rendered = Text::new();
        if let Alternative::Integer(value) = alternative {
            rendered = value.to_string();
        }
        output.push(Array(vec![
            owned(values[0].to_string()),
            owned((widened + 0.5).to_string()),
            owned(row.value.to_string()),
            owned(format!("{value2}{rendered}")),
        ]));
    }
    output
}
fn main() {}
