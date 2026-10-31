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
fn group_by<T, K: PartialEq, F: FnMut(&T, usize) -> K>(
    values: impl IntoIterator<Item = T>,
    mut callback: F,
) -> Vec<(K, Vec<T>)> {
    let mut groups: Vec<(K, Vec<T>)> = Vec::new();
    for (index, value) in values.into_iter().enumerate() {
        let key = callback(&value, index);
        if let Some((_, items)) = groups.iter_mut().find(|(k, _)| k == &key) {
            items.push(value);
        } else {
            groups.push((key, vec![value]));
        }
    }
    groups
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(i, value1);
        let mut source = vec![base, add(base, 1), add(base, 2), add(base, 3)];
        let mut visits = 0;
        let groups = group_by(source.iter().copied(), |value, index| {
            visits = add(visits, add(i64::try_from(index).unwrap(), 1));
            value % 2
        });
        source.push(add(base, 4));
        let first = get(&groups, &(base % 2))
            .map_or_else(std::string::String::new, |items| join(items, ","));
        let second = get(&groups, &(add(base, 1) % 2))
            .map_or_else(std::string::String::new, |items| join(items, ","));
        let characters = group_by(value2.chars().map(|ch| ch.to_string()), |value, _| {
            if value == "a" { 0 } else { 1 }
        });
        let alpha =
            get(&characters, &0).map_or_else(std::string::String::new, |items| items.join("|"));
        let other =
            get(&characters, &1).map_or_else(std::string::String::new, |items| items.join("|"));
        let keys: Vec<_> = groups.iter().map(|(key, _)| *key).collect();
        output.push(row(vec![
            len(&groups).to_string(),
            join(&keys, ","),
            first,
            second,
            visits.to_string(),
            join(&source, ","),
            len(&characters).to_string(),
            alpha,
            other,
        ]));
    }
    output
}
fn main() {}
