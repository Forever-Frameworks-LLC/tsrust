// Independent standard Rust optional callbacks and function expressions.
type String = std::borrow::Cow<'static, str>;
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
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.chars().count()).unwrap()
}
fn apply<F: Fn(i64) -> i64>(action: F, value: i64) -> i64 {
    action(value)
}
fn join(first: Option<std::string::String>, rest: Vec<std::string::String>) -> std::string::String {
    first.unwrap_or_else(|| "a".to_owned()) + &rest.len().to_string()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let scale = |v| add(mul(v, 3), value);
        let tags = vec![value.to_string(), "z".to_owned(), "tag".to_owned()];
        let sizes: Vec<_> = tags.iter().map(|tag| length(tag)).collect();
        let mut words = vec![value2.to_string(), value.to_string(), "z".to_owned()];
        words.sort();
        let mut total = 0;
        sizes.iter().for_each(|size| total = add(total, *size));
        output.push(Array(
            vec![
                apply(scale, i % 5).to_string(),
                total.to_string(),
                words.remove(0),
                join(None, vec![]),
                join(Some("b".to_owned()), vec!["c".to_owned(), "d".to_owned()]),
            ]
            .into_iter()
            .map(String::Owned)
            .collect(),
        ));
    }
    output
}
fn main() {}
