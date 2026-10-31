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
fn tier(code: i64) -> &'static str {
    match code % 7 {
        0 => "zero",
        1 | 2 => "low",
        5 => "five",
        _ => "other",
    }
}
enum Mixed {
    Integer(i64),
    Text(Text),
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mut steps = 0;
        for k in 0..6 {
            match add(base, k) % 3 {
                0 => continue,
                1 => steps = add(steps, 10),
                _ => steps = add(steps, 1),
            }
            steps = add(steps, 100);
        }
        let word = if base % 2 == 0 {
            value2.repeat(1)
        } else {
            base.to_string()
        };
        let text = match word.as_str() {
            "abc" => "short",
            "dynamic é🙂 text" => "long",
            _ => "number",
        };
        let mixed = if base % 3 == 0 {
            Mixed::Text("three".to_owned())
        } else {
            Mixed::Integer(base)
        };
        let seen = match mixed {
            Mixed::Text(text) if text == "three" => text.to_uppercase(),
            Mixed::Integer(13) => "thirteen".to_owned(),
            _ => "none".to_owned(),
        };
        output.push(Array(vec![
            String::Borrowed(tier(base)),
            owned(steps.to_string()),
            String::Borrowed(text),
            owned(seen),
        ]));
    }
    output
}
fn main() {}
