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
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let label;
        if base >= 64 {
            label = "high";
        } else if base >= 32 {
            label = "mid";
        } else {
            label = "low";
        }
        let size;
        let step = base % 5;
        size = add(mul(step, 2), 1);
        let kind;
        match step {
            0 => kind = "zero",
            1 | 2 => kind = "few",
            _ => kind = "many",
        };
        let text;
        {
            let prefix = base.to_string();
            text = format!("{prefix}{value2}");
        }
        let note: Option<&str> = None;
        let absent = note.is_none();
        output.push(Array(vec![
            String::Borrowed(label),
            owned(size.to_string()),
            String::Borrowed(kind),
            owned(text),
            owned(absent.to_string()),
        ]));
    }
    output
}
fn main() {}
