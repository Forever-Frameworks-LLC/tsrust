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
struct Point {
    x: i64,
    y: i64,
    label: Text,
}
fn pair(base: i64, input: &str) -> (i64, Text) {
    (add(base, 1), format!("{input}!"))
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let (count, word) = pair(base, value2);
        let point = Point {
            x: base,
            y: i % 7,
            label: value2.as_ref().to_owned(),
        };
        let Point { x, y, label } = &point;
        let pairs = vec![("a", base), ("b", count)];
        let mut names = Text::new();
        let mut total = 0;
        for (name, amount) in &pairs {
            names.push_str(name);
            total = add(total, *amount);
        }
        output.push(Array(vec![
            owned(add(add(count, *x), *y).to_string()),
            owned(format!("{word}{label}")),
            owned(names),
            owned(total.to_string()),
        ]));
    }
    output
}
fn main() {}
