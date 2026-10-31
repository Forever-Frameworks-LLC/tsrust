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
    label: String,
}
fn size(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut text = String::Borrowed("a");
        let first = {
            text = owned("x".repeat(usize::try_from(value).unwrap()));
            size(&text)
        };
        let lent = {
            text = owned(format!("four{value}"));
            size(&text)
        };
        let mut values = vec![0];
        let count = {
            values = vec![value, add(value, 1), add(value, 2)];
            i64::try_from(values.len()).unwrap()
        };
        let mut point = Point {
            x: 0,
            label: String::Borrowed("p"),
        };
        let coordinate = {
            point = Point {
                x: mul(value, 2),
                label: String::Borrowed("point"),
            };
            point.x
        };
        let label_size = {
            point.label = owned("q".repeat(usize::try_from(add(value, 1)).unwrap()));
            size(&point.label)
        };
        let mut current = "";
        let mut rounds = 0;
        loop {
            current = if rounds < value { "go" } else { "" };
            if current.is_empty() {
                break;
            }
            rounds = add(rounds, 1);
        }
        output.push(NumericArray(vec![
            first,
            lent,
            count,
            values[2],
            coordinate,
            label_size,
            size(&point.label),
            rounds,
            size(&text),
        ]));
    }
    Array(output)
}
fn main() {}
