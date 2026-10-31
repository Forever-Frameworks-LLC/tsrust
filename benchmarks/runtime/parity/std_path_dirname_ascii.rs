// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn dirname(path: &str) -> std::string::String {
    let end = path.trim_end_matches('/');
    match end.rfind('/') {
        Some(0) => "/".to_owned(),
        Some(1) if end.starts_with("//") => "//".to_owned(),
        Some(at) => end[..at].to_owned(),
        None => {
            if path.starts_with('/') {
                "/".to_owned()
            } else {
                ".".to_owned()
            }
        }
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
        values.push(String::Owned(dirname(path)));
    }
    Array(values)
}
fn main() {}
