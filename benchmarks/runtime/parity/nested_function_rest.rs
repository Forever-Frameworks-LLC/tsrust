// Independent standard Rust nested functions and closures; checked integer arithmetic.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.chars().count()).unwrap()
}

fn sum(base: i64, values: Vec<i64>) -> i64 {
    values.into_iter().fold(base, add)
}
fn front(base: i64, first: Option<i64>, tail: Vec<i64>) -> i64 {
    add(first.unwrap_or(base), i64::try_from(tail.len()).unwrap())
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let action = |values| sum(value, values);
        let empty = action(vec![]);
        let supplied = action(vec![2, 3]);
        let pack = vec![4, 5];
        let spread = action(pack);
        let prefix = |first, tail| front(value, first, tail);
        let omitted = prefix(None, vec![]);
        let undefined_first = prefix(None, vec![1, 2]);
        let explicit = prefix(Some(3), vec![4]);
        output.push(NumericArray(vec![
            empty,
            supplied,
            spread,
            omitted,
            undefined_first,
            explicit,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
