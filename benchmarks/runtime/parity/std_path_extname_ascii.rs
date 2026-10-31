// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn extname(path: &str) -> std::string::String {
    let name = path.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    if name == ".." {
        return std::string::String::new();
    }
    match name.rfind('.') {
        Some(at) if at > 0 => name[at..].to_owned(),
        _ => std::string::String::new(),
    }
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let mut values = Vec::new();
    for i in 0..value0 {
        let even = i.checked_add(value1).unwrap() % 2 == 0;
        let path = if even {
            "/var/log/archive.tar.gz/"
        } else {
            "/tmp/report.txt"
        };
        values.push(String::Owned(extname(path)));
    }
    Array(values)
}
fn main() {}
