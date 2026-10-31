// Independent standard Rust string operations; retained dynamic outputs are independently owned.
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
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn index(i: i64, stride: i64, seed: i64, len: i64) -> usize {
    usize::try_from(add(mul(i, stride), seed) % len).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn pieces(text: &str, limit: usize) -> Vec<std::string::String> {
    text.chars().take(limit).map(|ch| ch.to_string()).collect()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let values = [
        value2.to_string(),
        "".to_owned(),
        "::a::::b::".to_owned(),
        "é😀é😀".to_owned(),
        "a\0b".to_owned(),
        "e\u{301}::😀".to_owned(),
        "\u{feff}::中".to_owned(),
        value2.repeat(2),
    ];
    let limits = [0, 1, 2, 3, 7];
    let mut output = Vec::new();
    for i in 0..value0 {
        let at = index(i, 13, value1, 8);
        let selected = pieces(&values[at], limits[index(i, 3, value1, 5)]);
        output.push(Array(selected.into_iter().map(String::Owned).collect()));
    }
    output
}
fn main() {}
