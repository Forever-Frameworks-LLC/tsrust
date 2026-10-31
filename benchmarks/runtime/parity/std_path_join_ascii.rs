// Independent standard Rust reference; retain each operation and returned string.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
// POSIX lexical normalization: PathBuf alone does not remove parent segments.
fn normalize(path: &str) -> std::string::String {
    let absolute = path.starts_with('/');
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|p| *p != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    let mut result = if absolute {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    };
    if result.is_empty() {
        result.push('.');
    }
    if path.ends_with('/') && !result.ends_with('/') {
        result.push('/');
    }
    result
}
fn join(parts: &[&str]) -> std::string::String {
    normalize(
        &parts
            .iter()
            .copied()
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("/"),
    )
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let mut values = Vec::new();
    for i in 0..value0 {
        let even = i.checked_add(value1).unwrap() % 2 == 0;
        let part = if even { ".." } else { "reports" };
        values.push(String::Owned(join(&["/tmp", part, "archive.txt"])));
    }
    Array(values)
}
fn main() {}
