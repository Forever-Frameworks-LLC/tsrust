// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
// Inputs in this corpus have no dot or parent segments; compare host path components.
fn relative(from: &str, to: &str) -> std::string::String {
    let cwd = std::env::current_dir().unwrap();
    let from = cwd.join(from);
    let to = cwd.join(to);
    let left: Vec<_> = from.components().collect();
    let right: Vec<_> = to.components().collect();
    let common = left.iter().zip(&right).take_while(|(a, b)| a == b).count();
    let mut result = std::path::PathBuf::new();
    for _ in common..left.len() {
        result.push("..");
    }
    for component in &right[common..] {
        result.push(component.as_os_str());
    }
    result.into_os_string().into_string().unwrap()
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let mut values = Vec::new();
    for i in 0..value0 {
        let even = i.checked_add(value1).unwrap() % 2 == 0;
        let to = if even {
            "reports/archive.txt"
        } else {
            "reports/é😊/2026"
        };
        values.push(String::Owned(relative("reports/é😊", to)));
    }
    Array(values)
}
fn main() {}
