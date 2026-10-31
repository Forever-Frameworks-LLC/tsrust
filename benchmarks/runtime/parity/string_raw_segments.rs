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
struct Template {
    raw: Vec<Text>,
}
enum Insert<'a> {
    Integer(i64),
    Boolean(bool),
    Text(&'a str),
}
fn raw(template: &Template, inserts: &[Insert<'_>]) -> Text {
    let mut output = Text::new();
    for (index, part) in template.raw.iter().enumerate() {
        if index > 0 {
            if let Some(insert) = inserts.get(index - 1) {
                match insert {
                    Insert::Integer(value) => output.push_str(&value.to_string()),
                    Insert::Boolean(value) => output.push_str(&value.to_string()),
                    Insert::Text(value) => output.push_str(value),
                }
            }
        }
        output.push_str(part);
    }
    output
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let empty = Template { raw: Vec::new() };
    let single = Template {
        raw: vec!["only".to_owned()],
    };
    let inserts = [Insert::Text("X"), Insert::Text("Y")];
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let template = Template {
            raw: vec!["a".to_owned(), "é".to_owned(), format!("{value2}\\n")],
        };
        output.push(Array(vec![
            owned(raw(
                &template,
                &[Insert::Integer(base), Insert::Boolean(true)],
            )),
            owned(raw(&template, &[Insert::Text("X")])),
            owned(raw(&template, &inserts)),
            owned(raw(&single, &inserts)),
            owned(raw(&empty, &[Insert::Integer(base)])),
        ]));
    }
    output
}
fn main() {}
