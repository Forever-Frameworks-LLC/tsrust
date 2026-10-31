// Independent standard Rust insertion-ordered collections and Unicode iteration.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn len<T>(values: &[T]) -> i64 {
    i64::try_from(values.len()).unwrap()
}
fn row(values: Vec<std::string::String>) -> Array<String> {
    Array(values.into_iter().map(String::Owned).collect())
}
fn join(values: &[i64], separator: &str) -> std::string::String {
    values
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(separator)
}
// ponytail: linear scans suit this <=5-entry corpus; use an indexed ordered map/set if it grows.
fn insert<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}
fn set<T: PartialEq>(values: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut result = Vec::new();
    for value in values {
        insert(&mut result, value);
    }
    result
}
fn delete<T: PartialEq>(values: &mut Vec<T>, key: &T) -> bool {
    if let Some(at) = values.iter().position(|value| value == key) {
        values.remove(at);
        true
    } else {
        false
    }
}
fn put<K: PartialEq, V>(values: &mut Vec<(K, V)>, key: K, value: V) {
    if let Some((_, current)) = values.iter_mut().find(|(k, _)| k == &key) {
        *current = value;
    } else {
        values.push((key, value));
    }
}
fn map<K: PartialEq, V>(values: impl IntoIterator<Item = (K, V)>) -> Vec<(K, V)> {
    let mut result = Vec::new();
    for (key, value) in values {
        put(&mut result, key, value);
    }
    result
}
fn get<'a, K: PartialEq, V>(values: &'a [(K, V)], key: &K) -> Option<&'a V> {
    values
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, value)| value)
}
fn remove<K: PartialEq, V>(values: &mut Vec<(K, V)>, key: &K) -> bool {
    if let Some(at) = values.iter().position(|(k, _)| k == key) {
        values.remove(at);
        true
    } else {
        false
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(i, value1);
        let mut values = map([
            (value2.to_string(), base),
            ("tail".to_owned(), add(base, 1)),
        ]);
        let mut direct = std::string::String::new();
        for entry in &values {
            direct.push_str(&format!("{}:{}|", entry.0, entry.1));
        }
        let mut pattern = std::string::String::new();
        for (key, value) in &values {
            pattern.push_str(&format!("{}:{}|", key, value));
        }
        put(&mut values, "last".to_owned(), add(base, 2));
        let mut count = 0;
        for _entry in &values {
            count = add(count, 1);
        }
        output.push(row(vec![direct, pattern, count.to_string()]));
    }
    output
}
fn main() {}
