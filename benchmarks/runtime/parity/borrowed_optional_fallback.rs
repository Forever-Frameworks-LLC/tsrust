// Independent Rust records/unions reference; preserve storage, order and observed operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
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
struct Row {
    label: Text,
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let rows = vec![
            Some(Row {
                label: value2.repeat(usize::try_from(add(i, value1) % 3 + 1).unwrap()),
            }),
            None,
            Some(Row { label: Text::new() }),
        ];
        for item in &rows {
            let label = item
                .as_ref()
                .map(|row| row.label.as_str())
                .unwrap_or("none");
            output.push(Array(vec![
                owned(label.to_uppercase()),
                owned(label.to_owned()),
                owned(i.to_string()),
            ]));
        }
    }
    output
}
fn main() {}
