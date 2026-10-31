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
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut selected = 0;
        if selected == 0 {
            selected = value;
        }
        if selected != 0 {
            selected = add(value, 1);
        }
        let mut label = "";
        if label.is_empty() {
            label = "guest";
        }
        if !label.is_empty() {
            label = "member";
        }
        let mut optional: Option<&str> = None;
        if optional.is_none_or(|s| s.is_empty()) {
            optional = Some("present");
        }
        let mut length = 0;
        if let Some(text) = optional {
            if !text.is_empty() {
                length = i64::try_from(text.len()).unwrap();
            }
        }
        let mut remaining = value;
        let mut rounds = 0;
        while remaining != 0 {
            rounds = add(rounds, 1);
            remaining = remaining.checked_sub(1).unwrap();
        }
        output.push(NumericArray(vec![
            selected,
            i64::try_from(label.len()).unwrap(),
            length,
            rounds,
            i64::from(value == 0),
            i64::from(0 == 0),
        ]));
    }
    Array(output)
}
fn main() {}
