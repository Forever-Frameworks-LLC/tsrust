// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let mut values = Vec::new();
    for i in 0..value0 {
        let even = i.checked_add(value1).unwrap() % 2 == 0;
        let path = if even {
            "reports/../archive/"
        } else {
            "/tmp/report.txt"
        };
        values.push(String::Owned(
            std::path::Path::new(path).is_absolute().to_string(),
        ));
    }
    Array(values)
}
fn main() {}
