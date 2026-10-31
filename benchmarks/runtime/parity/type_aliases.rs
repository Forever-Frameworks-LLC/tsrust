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
enum Id {
    Number(i64),
    Text(Text),
}
type Values<T = Text> = Vec<T>;
type Ids = Values<Id>;
type Count = i64;
type Pair = (Count, Text);
type Row = Vec<String>;
type Rows = Vec<Array<String>>;
fn kind(id: &Id) -> &'static str {
    match id {
        Id::Text(_) => "text",
        Id::Number(_) => "number",
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output: Rows = Vec::new();
    for i in 0..value0 {
        let k: Count = add(i, value1);
        let ids: Ids = vec![Id::Number(k), Id::Text(value2.as_ref().to_owned())];
        let pair: Pair = (mul(k, 2), k.to_string());
        let (doubled, label) = pair;
        let row: Row = vec![
            String::Borrowed(kind(&ids[0])),
            String::Borrowed(kind(&ids[1])),
            owned(doubled.to_string()),
            owned(label),
            owned(ids.len().to_string()),
        ];
        output.push(Array(row));
    }
    output
}
fn main() {}
