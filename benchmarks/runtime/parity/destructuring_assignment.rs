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
struct Cell {
    x: i64,
    y: i64,
    label: Text,
}
fn ends(base: i64, input: &str) -> (i64, Text) {
    (add(base, 2), format!("{input}?"))
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mut left = value2.repeat(1);
        let mut right = base.to_string();
        (left, right) = (right, left);
        let mut f0 = 0;
        let mut f1 = 1;
        for _turn in 0..add(base % 4, 2) {
            (f0, f1) = (f1, add(f0, f1));
        }
        let (count, word) = ends(base, value2);
        let cell = Cell {
            x: base,
            y: count,
            label: format!("{word}!"),
        };
        let Cell {
            x: cx, label: text, ..
        } = cell;
        output.push(Array(vec![
            owned(left),
            owned(right),
            owned(f0.to_string()),
            owned(count.to_string()),
            owned(word),
            owned(add(cx, cell.y).to_string()),
            owned(text),
        ]));
    }
    output
}
fn main() {}
