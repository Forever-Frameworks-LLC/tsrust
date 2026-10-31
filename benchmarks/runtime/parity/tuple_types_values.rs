// Independent standard Rust language reference; preserve values and owned data operations.
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
fn owned(text: impl Into<Text>) -> String {
    String::Owned(text.into())
}
type Pair = (i64, Text);
fn make_pair(index: i64, value: &str) -> Pair {
    (index, value.to_owned())
}
fn copy_pair(value: &Pair) -> Pair {
    (value.0, value.1.clone())
}
// The two-element tuple shape and source Array.isArray result are static Rust facts.
fn pair_length(_: &Pair) -> i64 {
    2
}
fn pair_is_array(_: &Pair) -> bool {
    true
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let text = value2.repeat(usize::try_from(base % 3 + 1).unwrap());
        let mut pair = make_pair(base, &text);
        let copy = copy_pair(&pair);
        pair.0 = add(base, 1);
        pair.1 = value2.as_ref().to_owned();
        let rows = vec![pair, copy];
        let mapped: Vec<Pair> = rows
            .iter()
            .enumerate()
            .map(|(index, item)| (add(item.0, i64::try_from(index).unwrap()), item.1.clone()))
            .collect();
        output.push(Array(vec![
            owned(rows[0].0.to_string()),
            owned(rows[0].1.clone()),
            owned(rows[1].0.to_string()),
            owned(rows[1].1.clone()),
            owned(mapped[1].0.to_string()),
            owned(mapped[1].1.clone()),
            owned(pair_length(&mapped[1]).to_string()),
            owned(pair_is_array(&mapped[0]).to_string()),
        ]));
    }
    output
}
fn main() {}
