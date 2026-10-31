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
struct Point {
    x: i64,
    y: Option<i64>,
}
fn width(point: &Point) -> i64 {
    add(point.x, point.y.unwrap_or(0))
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let k = add(i, value1);
        let flat = Point { x: k, y: None };
        let full = Point {
            x: k,
            y: Some(k % 7),
        };
        let mut moving = Point { x: 1, y: None };
        moving.y = Some(k % 3);
        let taken = flat.y.unwrap_or(9);
        output.push(Array(vec![
            owned(width(&flat).to_string()),
            owned(width(&full).to_string()),
            owned(width(&moving).to_string()),
            owned(taken.to_string()),
            owned(value2.as_ref().to_owned()),
        ]));
    }
    output
}
fn main() {}
